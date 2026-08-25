//! `ce-ai skills adopt`: put pre-existing `ce-*` skill copies under ce-ai
//! management (canonical-skills-adoption, R2/R3/R9/R14/R17).
//!
//! Detection classifies every `ce-*` directory under a harness skills root as
//! adoptable (frontmatter-verified against the canonical harvest), an
//! unrecognized user-authored skill (never touched, R9), or symlinked
//! (rejected — the rewrite engine only touches regular files). Execution is
//! gated until the transactional engine ships (plan U3).

use std::collections::BTreeMap;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use clap::Args as ClapArgs;

use crate::commands::sync::sync_skills_root;
use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
use crate::opencode::manifest::InstallManifest;
use crate::source::cache::managed_tree;
use crate::source::registry::parse_skill_frontmatter;
use crate::state::state::{SkillSurface, State};

#[derive(ClapArgs, Debug, Clone)]
pub struct Args {
    /// Harness whose skills root is scanned (name or `all`).
    #[arg(long, default_value = "all")]
    pub harness: String,

    /// Confirm every adoptable surface without an interactive prompt.
    #[arg(long, default_value_t = false)]
    pub yes: bool,
}

/// Classification of one `ce-*` directory found under a skills root.
struct FoundSkill {
    dir_name: String,
    /// Frontmatter name matches the canonical harvested set.
    canonical: bool,
    symlinked: bool,
    stale: usize,
    current: usize,
    missing: usize,
}

impl FoundSkill {
    fn adoptable(&self) -> bool {
        self.canonical && !self.symlinked
    }
}

/// A harness skills root holding at least one `ce-*` directory.
struct SurfaceReport {
    harness: String,
    root: PathBuf,
    found: Vec<FoundSkill>,
}

impl SurfaceReport {
    fn adoptable_count(&self) -> usize {
        self.found.iter().filter(|s| s.adoptable()).count()
    }
}

/// Canonical skills of the installed release: dir name -> {rel path: sha256}.
fn canonical_skills(
    tree: &BTreeMap<String, (String, String)>,
) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut skills: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for (managed_rel, (_, hash)) in tree {
        let Some(rest) = managed_rel.strip_prefix("skills/") else {
            continue;
        };
        let Some((dir, file)) = rest.split_once('/') else {
            continue;
        };
        skills
            .entry(dir.to_string())
            .or_default()
            .insert(file.to_string(), hash.clone());
    }
    skills
}

/// Resolves the installed release tree from the install manifest (GitHub
/// `tree` key or local `path` key).
fn canonical_source_tree(ctx: &Context) -> Result<PathBuf, CeError> {
    let manifest = InstallManifest::load(&ctx.opencode_config_dir)?;
    let tree = manifest
        .source
        .get("tree")
        .and_then(|t| t.as_str())
        .map(PathBuf::from)
        .or_else(|| {
            manifest
                .source
                .get("path")
                .and_then(|p| p.as_str())
                .map(PathBuf::from)
        });
    match tree {
        Some(t) if t.is_dir() => Ok(t),
        _ => Err(CeError::Usage(
            "no installed release tree found — run `ce-ai install` or `ce-ai sync` first"
                .to_string(),
        )),
    }
}

/// Classifies one `ce-*` directory against the canonical set (pure).
fn classify_found(
    dir_name: &str,
    dir: &Path,
    canonical: &BTreeMap<String, BTreeMap<String, String>>,
) -> FoundSkill {
    let mut found = FoundSkill {
        dir_name: dir_name.to_string(),
        canonical: false,
        symlinked: false,
        stale: 0,
        current: 0,
        missing: 0,
    };

    let Ok(entries) = fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let Ok(ty) = entry.file_type() else { continue };
        if ty.is_symlink() {
            found.symlinked = true;
        }
    }

    let canonical_files = canonical.get(dir_name);
    let skill_md = dir.join("SKILL.md");
    let frontmatter = fs::read_to_string(&skill_md)
        .map(|c| parse_skill_frontmatter(&c))
        .unwrap_or_default();
    found.canonical = canonical_files.is_some() && frontmatter.name == dir_name && !found.symlinked;

    let Some(expected) = canonical_files else {
        return found;
    };
    for (file, hash) in expected {
        match fs::read(dir.join(file)).map(|bytes| crate::state::diff::sha256_hex(&bytes)) {
            Ok(disk) if disk == *hash => found.current += 1,
            Ok(_) => found.stale += 1,
            Err(_) => found.missing += 1,
        }
    }
    found
}

/// Scans one harness skills root for `ce-*` directories (pure filesystem).
fn detect_surface(
    harness: &str,
    root: &Path,
    canonical: &BTreeMap<String, BTreeMap<String, String>>,
) -> Option<SurfaceReport> {
    let entries = fs::read_dir(root).ok()?;
    let mut found: Vec<FoundSkill> = Vec::new();
    for entry in entries.flatten() {
        let Ok(ty) = entry.file_type() else { continue };
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("ce-") {
            continue;
        }
        if !ty.is_dir() {
            continue;
        }
        found.push(classify_found(&name, &entry.path(), canonical));
    }
    if found.is_empty() {
        None
    } else {
        Some(SurfaceReport {
            harness: harness.to_string(),
            root: root.to_path_buf(),
            found,
        })
    }
}

fn record_decline(ctx: &Context, surface: &SurfaceReport) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;
    state
        .skill_surfaces
        .retain(|s| !(s.harness == surface.harness && s.root == surface.root));
    state.skill_surfaces.push(SkillSurface {
        harness: surface.harness.clone(),
        root: surface.root.clone(),
        status: "declined".to_string(),
        files: vec![],
        adopted_at: None,
    });
    state.save(&state_path)
}

/// Runs detection across the target harnesses and prints the report.
pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let tree = managed_tree(&canonical_source_tree(ctx)?)?;
    let canonical = canonical_skills(&tree);
    if canonical.is_empty() {
        return Err(CeError::Usage(
            "the installed release ships no skills/ tree — nothing to adopt".to_string(),
        ));
    }

    let home_dir = crate::harness::home_dir_from_ctx(ctx);
    let targets: Vec<HarnessKind> = if args.harness == "all" {
        HarnessKind::all().to_vec()
    } else {
        vec![args.harness.parse::<HarnessKind>()?]
    };

    let mut reports: Vec<SurfaceReport> = Vec::new();
    for kind in &targets {
        let root = sync_skills_root(*kind, &home_dir);
        if let Some(report) = detect_surface(kind.as_str(), &root, &canonical) {
            reports.push(report);
        }
    }

    if reports.is_empty() {
        println!("skills adopt: no ce-* skill directories found under any harness skills root");
        return Ok(());
    }

    let interactive = std::io::stdin().is_terminal();
    let mut confirmed_any = false;
    for report in &reports {
        println!(
            "== surface {} ({}) ==",
            report.harness,
            report.root.display()
        );
        for found in &report.found {
            if found.symlinked {
                println!(
                    "  ○ {} — symlinked; ce-ai never rewrites links (skipped)",
                    found.dir_name
                );
            } else if !found.canonical {
                println!(
                    "  ○ {} — unrecognized ce-* skill (frontmatter not in the canonical set); user-authored, never touched",
                    found.dir_name
                );
            } else {
                println!(
                    "  ✓ {} — adoptable ({} current, {} stale, {} missing)",
                    found.dir_name, found.current, found.stale, found.missing
                );
            }
        }

        if report.adoptable_count() == 0 {
            continue;
        }
        if args.yes {
            confirmed_any = true;
        } else if interactive {
            if prompt_decline(ctx, report)? {
                continue;
            }
            confirmed_any = true;
        } else {
            println!("  pending-adoption: re-run with --yes to confirm adoption");
        }
    }

    if !confirmed_any {
        return Ok(());
    }

    // Transactional rewrite engine lands with the uninstall scoping rework
    // (plan U3/U6 — no released adoption before uninstall is safe). The gate
    // leaves every surface untouched: no writes, no ledger changes.
    Err(CeError::Runtime(
        "adoption engine ships in the next release (canonical-skills-adoption PR 2); surfaces left untouched".to_string(),
    ))
}

/// Interactive per-surface confirmation. Returns `true` when the user
/// declined (recorded in the ledger, R3); `false` when they confirmed.
fn prompt_decline(ctx: &Context, report: &SurfaceReport) -> Result<bool, CeError> {
    println!(
        "  Adopt surface {} under ce-ai management? [y/N]",
        report.root.display()
    );
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    if answer.trim().eq_ignore_ascii_case("y") {
        return Ok(false);
    }
    record_decline(ctx, report)?;
    println!("  → declined; recorded. Re-run `skills adopt` to reconsider.");
    Ok(true)
}
