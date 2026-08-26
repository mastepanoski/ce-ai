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
use crate::state::backups::backup_file;
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
pub(crate) struct FoundSkill {
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
pub(crate) struct SurfaceReport {
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
pub(crate) fn detect_surface(
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

/// Adoptable-but-untracked surfaces for the matrix's `pending-adoption`
/// state (R17): canonical-verified `ce-*` content under a harness skills
/// root that the ledger does not track. Best-effort — detection failures
/// yield an empty list rather than failing sync.
pub(crate) fn pending_adoptions(
    ctx: &Context,
    home_dir: &Path,
    ledger_roots: &[(String, PathBuf)],
) -> Vec<(String, PathBuf)> {
    let Ok(tree_path) = canonical_source_tree(ctx) else {
        return Vec::new();
    };
    let Ok(tree) = managed_tree(&tree_path) else {
        return Vec::new();
    };
    let canonical = canonical_skills(&tree);
    if canonical.is_empty() {
        return Vec::new();
    }
    let mut pending = Vec::new();
    for kind in HarnessKind::all() {
        let root = sync_skills_root(kind, home_dir);
        if ledger_roots
            .iter()
            .any(|(h, r)| h == kind.as_str() && r == &root)
        {
            continue;
        }
        if let Some(report) = detect_surface(kind.as_str(), &root, &canonical) {
            if report.adoptable_count() > 0 {
                pending.push((report.harness.clone(), report.root.clone()));
            }
        }
    }
    pending
}

/// CE content sitting under known plugin-cache/marketplace roots (R18):
/// reported as `external-duplicate`, never adopted or modified.
pub(crate) fn external_duplicates(home_dir: &Path) -> Vec<String> {
    let mut hits = Vec::new();
    let cache = home_dir.join(".claude/plugins/cache");
    let Ok(entries) = fs::read_dir(&cache) else {
        return hits;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains("compound-engineering") {
            hits.push(entry.path().to_string_lossy().to_string());
        }
    }
    hits.sort();
    hits
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
            execute_adoption(ctx, report)?;
            println!("  → adopted under ce-ai management");
        } else if interactive {
            if prompt_decline(ctx, report)? {
                continue;
            }
            execute_adoption(ctx, report)?;
            println!("  → adopted under ce-ai management");
        } else {
            println!("  pending-adoption: re-run with --yes to confirm adoption");
        }
    }

    Ok(())
}

/// One filesystem mutation recorded for auto-restore on failure (R15).
struct Mutation {
    path: PathBuf,
    /// Prior bytes; `None` when this adoption created the file.
    prior: Option<Vec<u8>>,
}

/// Executes the transactional adoption of one confirmed surface (R15):
/// per-file backup → journal arm → atomic writes (completing the set) →
/// retire prior managed surfaces (ledger roots and the manifest-tracked
/// managed-dir `skills/` tree) → ledger saved atomically last. Any failure
/// auto-restores every mutation in reverse order and leaves the surface
/// unadopted.
fn execute_adoption(ctx: &Context, report: &SurfaceReport) -> Result<(), CeError> {
    let source_tree = canonical_source_tree(ctx)?;
    let tree = managed_tree(&source_tree)?;
    let canonical = canonical_skills(&tree);
    let backups = ctx.config_dir.join("backups");
    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;
    let mut journal = Some(crate::state::journal::Journal::begin(
        &ctx.config_dir,
        "adopt",
    )?);
    let mut mutations: Vec<Mutation> = Vec::new();

    let outcome = adopt_surface(&mut AdoptionTx {
        ctx,
        report,
        source_tree: &source_tree,
        canonical: &canonical,
        state: &mut state,
        journal: &mut journal,
        mutations: &mut mutations,
        backups: &backups,
    });

    match outcome {
        Ok(()) => {
            state.save(&state_path)?;
            if let Some(j) = journal.take() {
                j.complete()?;
            }
            Ok(())
        }
        Err(e) => {
            for m in mutations.iter().rev() {
                match &m.prior {
                    Some(bytes) => {
                        let _ = fs::write(&m.path, bytes);
                    }
                    None => {
                        let _ = fs::remove_file(&m.path);
                    }
                }
            }
            if let Some(j) = journal.take() {
                let _ = j.complete();
            }
            Err(e)
        }
    }
}

/// Borrowed transaction context shared by the adoption steps.
struct AdoptionTx<'a> {
    ctx: &'a Context,
    report: &'a SurfaceReport,
    source_tree: &'a Path,
    canonical: &'a BTreeMap<String, BTreeMap<String, String>>,
    state: &'a mut State,
    journal: &'a mut Option<crate::state::journal::Journal>,
    mutations: &'a mut Vec<Mutation>,
    backups: &'a Path,
}

/// The adoption transaction body. `state` is saved by the caller only on
/// success — the ledger write is the last durable action (R15).
fn adopt_surface(tx: &mut AdoptionTx) -> Result<(), CeError> {
    let AdoptionTx {
        ctx,
        report,
        source_tree,
        canonical,
        state,
        journal,
        mutations,
        backups,
    } = tx;
    // 1. Rewrite stale + create missing canonical skills (completion, R2).
    for (skill_dir, files) in canonical.iter() {
        for (file, hash) in files {
            let dest = report.root.join(skill_dir).join(file);
            let prior = fs::read(&dest).ok();
            if let Some(bytes) = &prior {
                if crate::state::diff::sha256_hex(bytes) == *hash {
                    continue;
                }
                backup_file(backups, &dest)?;
            }
            mutations.push(Mutation {
                path: dest.clone(),
                prior,
            });
            if let Some(j) = journal.as_mut() {
                j.arm(&dest)?;
            }
            let content = fs::read(source_tree.join("skills").join(skill_dir).join(file))?;
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            crate::state::write_atomic(&dest, &content)?;
        }
    }

    // 2. Retire prior managed surfaces for this harness (R13).
    retire_prior_surfaces(&mut AdoptionTx {
        ctx,
        report,
        source_tree,
        canonical,
        state,
        journal,
        mutations,
        backups,
    })?;

    // 3. Ledger adopted entry — the last durable action. Re-adoption
    //    preserves the original `adopted_at` provenance.
    let previous_adopted_at = state
        .skill_surfaces
        .iter()
        .find(|s| s.harness == report.harness && s.root == report.root)
        .and_then(|s| s.adopted_at.clone());
    state
        .skill_surfaces
        .retain(|s| !(s.harness == report.harness && s.root == report.root));
    let mut files: Vec<crate::state::state::SkillSurfaceFile> = Vec::new();
    for (skill_dir, expected) in canonical.iter() {
        for (file, hash) in expected {
            files.push(crate::state::state::SkillSurfaceFile {
                path: format!("{skill_dir}/{file}"),
                sha256: hash.clone(),
            });
        }
    }
    state.skill_surfaces.push(SkillSurface {
        harness: report.harness.clone(),
        root: report.root.clone(),
        status: "adopted".to_string(),
        files,
        adopted_at: Some(previous_adopted_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339())),
    });
    Ok(())
}

/// Removes every prior managed skills surface for this harness except the
/// one being adopted: ledger-tracked adopted roots and the manifest-tracked
/// managed-dir `skills/` tree (R13). Each removed file is backed up first.
fn retire_prior_surfaces(tx: &mut AdoptionTx) -> Result<(), CeError> {
    let AdoptionTx {
        ctx,
        report,
        state,
        journal,
        mutations,
        backups,
        ..
    } = tx;
    let prior_roots: Vec<PathBuf> = state
        .skill_surfaces
        .iter()
        .filter(|s| s.harness == report.harness && s.root != report.root && s.status == "adopted")
        .map(|s| s.root.clone())
        .collect();
    for root in &prior_roots {
        let tracked: Vec<String> = state
            .skill_surfaces
            .iter()
            .filter(|s| s.root == *root)
            .flat_map(|s| s.files.iter().map(|f| f.path.clone()))
            .collect();
        for rel in &tracked {
            let p = root.join(rel);
            if !p.exists() {
                continue;
            }
            backup_file(backups, &p)?;
            mutations.push(Mutation {
                path: p.clone(),
                prior: fs::read(&p).ok(),
            });
            if let Some(j) = journal.as_mut() {
                j.arm(&p)?;
            }
            fs::remove_file(&p)?;
        }
        for rel in &tracked {
            prune_empty_parents(root, &root.join(rel));
        }
        state.skill_surfaces.retain(|s| s.root != *root);
    }

    // Manifest-tracked managed-dir skills tree: the fresh-machine canonical
    // copy written by install/sync (retired with backup, manifest updated).
    let Ok(manifest) = InstallManifest::load(&ctx.opencode_config_dir) else {
        return Ok(());
    };
    let retired: Vec<crate::opencode::manifest::ManifestFile> = manifest
        .files
        .iter()
        .filter(|f| f.path.starts_with("skills/"))
        .cloned()
        .collect();
    if retired.is_empty() {
        return Ok(());
    }
    let managed_dir = ctx
        .opencode_config_dir
        .join(crate::opencode::plugins::MANAGED_DIR);
    for f in &retired {
        let p = managed_dir.join(&f.path);
        if !p.exists() {
            continue;
        }
        backup_file(backups, &p)?;
        mutations.push(Mutation {
            path: p.clone(),
            prior: fs::read(&p).ok(),
        });
        if let Some(j) = journal.as_mut() {
            j.arm(&p)?;
        }
        fs::remove_file(&p)?;
        prune_empty_parents(&managed_dir, &p);
    }
    let remaining: Vec<crate::opencode::manifest::ManifestFile> = manifest
        .files
        .into_iter()
        .filter(|f| !f.path.starts_with("skills/"))
        .collect();
    InstallManifest {
        files: remaining,
        ..manifest
    }
    .write(&ctx.opencode_config_dir)?;
    Ok(())
}

/// Best-effort removal of directories left empty between `start` (a removed
/// file's parent) and `root` (the surface root, never removed). Shared with
/// uninstall's ledger-scoped removal.
pub(crate) fn prune_empty_parents(root: &Path, start: &Path) {
    let mut dir = start.to_path_buf();
    while dir.starts_with(root) && dir != root {
        match fs::read_dir(&dir) {
            Ok(mut entries) => {
                if entries.next().is_some() {
                    break;
                }
                if fs::remove_dir(&dir).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
        if !dir.pop() {
            break;
        }
    }
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
