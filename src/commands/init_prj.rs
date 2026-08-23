//! Project adoption init subcommand: `ce-ai init-prj`.

use std::fs;
use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::{AdoptionTier, ProjectAdoptionEntry, State};

pub const BLOCK_BEGIN_MARKER: &str = "<!-- ce-ai:block begin";
pub const BLOCK_END_MARKER: &str = "<!-- ce-ai:block end -->";
pub const GITIGNORE_BEGIN_MARKER: &str = "# BEGIN CE-AI MANAGED BLOCK";
pub const GITIGNORE_END_MARKER: &str = "# END CE-AI MANAGED BLOCK";

/// Managed block schema version, shared by the on-disk header and the
/// `state.json` adoption entry so the two cannot drift apart.
pub const BLOCK_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdoptionBlockStatus {
    Ok,
    StaleVersion { version: u32 },
    DriftDetected,
    MalformedBlock,
    BlockMissing,
    FileMissing,
    ReadError,
}

/// Single source of truth for classifying an on-disk adoption block status across doctor and status diagnostics.
pub fn check_adoption_block_status(
    agents_file: &std::path::Path,
    tier: AdoptionTier,
) -> AdoptionBlockStatus {
    if !agents_file.exists() {
        return AdoptionBlockStatus::FileMissing;
    }
    let Ok(text) = fs::read_to_string(agents_file) else {
        return AdoptionBlockStatus::ReadError;
    };
    let Some(start_idx) = text.find(BLOCK_BEGIN_MARKER) else {
        return AdoptionBlockStatus::BlockMissing;
    };
    if text[start_idx..].find(BLOCK_END_MARKER).is_none() {
        return AdoptionBlockStatus::MalformedBlock;
    }

    let inner_body = render_block_content(tier);
    let expected_sha = compute_sha256(inner_body);
    if text.contains(&expected_sha) {
        return AdoptionBlockStatus::Ok;
    }

    let declared_version = text.lines().find_map(|line| {
        let rest = line.trim().strip_prefix(BLOCK_BEGIN_MARKER)?;
        let v = rest.trim_start().strip_prefix("v=")?;
        v.split([' ', '-']).next()?.parse::<u32>().ok()
    });

    match declared_version {
        Some(v) if v < BLOCK_VERSION => AdoptionBlockStatus::StaleVersion { version: v },
        _ => AdoptionBlockStatus::DriftDetected,
    }
}

/// Renders the managed block content based on tier.
pub fn render_block_content(tier: AdoptionTier) -> &'static str {
    match tier {
        AdoptionTier::Full => {
            r#"## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement

All AI agents MUST follow the 7-stage Compound Engineering development cycle:
`[Stage 1: Ideation]` ➔ `[Stage 2: OpenSpec Definition]` ➔ `[Stage 3: Execution Plan]`
➔ `[Stage 4: TDD & Implementation]` ➔ `[Stage 5: Verification]` ➔ `[Stage 6: Knowledge Capture]`
➔ `[Stage 7: Git Shipping]`

### Stage 2 OpenSpec Enforcement Requirements
Before creating PRs or writing feature code, agents MUST verify `openspec/changes/<feature_name>/` contains:
- `proposal.md`: Problem statement, in-scope/out-of-scope boundaries, and success criteria.
- `exploration.md`: Technical investigation and architectural tradeoffs.
- `design.md`: Technical design, system architecture, structs, and API/CLI contracts.
- `spec.md`: Formal requirements using `WHEN ... THEN ...` format and explicit acceptance criteria.
- `tasks.md`: Atomic, executable task checklist with TDD verification steps.

### Single Source of Truth Rule
Ideation artifacts (`docs/brainstorms/*.md`, `docs/ideation/*.md`) are disposable inputs, NOT parallel specifications. Distill their conclusions into the OpenSpec files above (`proposal.md`, `exploration.md`) and reference the source doc instead of copying content. Never maintain brainstorm/ideation documents in sync with OpenSpec. Skip ideation skills entirely when requirements and approach are already clear."#
        }
        AdoptionTier::Minimal => {
            r#"## 🔄 Compound Engineering Workflow Guidelines

AI agents operating on this codebase should follow structured planning and verification:
- Validate scope boundaries before making changes.
- Ensure all unit, integration, and linter tests pass before committing.
- Document key technical learnings and post-mortem fixes."#
        }
        AdoptionTier::Orchestrator => {
            r#"## 🔄 Orchestrator Agent Governance & Delegation Directives

Orchestrator agents MUST delegate domain tasks to specialized subagents:
- Use `ce-brainstorm` for scope exploration.
- Use `ce-plan` for implementation unit breakdown.
- Use `ce-code-review` before opening Pull Requests.
- Enforce strict PR CI status check gates before merging.
- Ideation outputs (`docs/brainstorms/`, `docs/ideation/`) are disposable inputs: distill them into the specs before delegation; never maintain them in parallel."#
        }
    }
}

/// Compute SHA256 of string content.
pub fn compute_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Executes `ce-ai init-prj`.
pub fn run(
    ctx: &Context,
    target_path_opt: Option<PathBuf>,
    tier_str: &str,
    force: bool,
) -> Result<(), CeError> {
    let raw_target = match target_path_opt {
        Some(p) => p,
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let target_dir = match raw_target.canonicalize() {
        Ok(canonical) => canonical,
        Err(_) => raw_target,
    };

    if !target_dir.exists() || !target_dir.is_dir() {
        return Err(CeError::Usage(format!(
            "target project path '{}' does not exist or is not a directory",
            target_dir.display()
        )));
    }

    let tier = match tier_str.to_lowercase().as_str() {
        "full" => AdoptionTier::Full,
        "minimal" => AdoptionTier::Minimal,
        "orchestrator" => AdoptionTier::Orchestrator,
        unknown => {
            return Err(CeError::Usage(format!(
                "unknown adoption tier '{}'. Supported tiers: full, minimal, orchestrator",
                unknown
            )))
        }
    };

    let agents_file = target_dir.join("AGENTS.md");
    let file_existed = agents_file.exists();

    let (existing_content, is_crlf) = if file_existed {
        let text = fs::read_to_string(&agents_file)?;
        let crlf = text.contains("\r\n");
        (text, crlf)
    } else {
        (String::new(), false)
    };

    let inner_body = render_block_content(tier);
    let body_sha256 = compute_sha256(inner_body);

    let newline = if is_crlf { "\r\n" } else { "\n" };

    let block_header = format!(
        "<!-- ce-ai:block begin v={} tier={} sha256={} -->",
        BLOCK_VERSION,
        tier_str.to_lowercase(),
        body_sha256
    );
    let full_block = format!(
        "{}{}{}{}{}",
        block_header, newline, inner_body, newline, BLOCK_END_MARKER
    );

    // Check if block already exists
    let new_content = if let Some(start_idx) = existing_content.find(BLOCK_BEGIN_MARKER) {
        if let Some(end_rel_idx) = existing_content[start_idx..].find(BLOCK_END_MARKER) {
            let end_idx = start_idx + end_rel_idx + BLOCK_END_MARKER.len();
            let existing_block = &existing_content[start_idx..end_idx];

            if existing_block == full_block && !force {
                if !ctx.quiet {
                    println!(
                        "Project at '{}' is already adopted with up-to-date block (SHA: {}).",
                        target_dir.display(),
                        &body_sha256[..8]
                    );
                }
                return Ok(());
            }

            let mut updated = String::new();
            updated.push_str(&existing_content[..start_idx]);
            updated.push_str(&full_block);
            updated.push_str(&existing_content[end_idx..]);
            updated
        } else {
            return Err(CeError::Runtime(format!(
                "malformed managed block in '{}': found begin marker without end marker",
                agents_file.display()
            )));
        }
    } else {
        let mut appended = existing_content.clone();
        if !appended.is_empty() && !appended.ends_with('\n') && !appended.ends_with("\r\n") {
            appended.push_str(newline);
        }
        if !appended.is_empty() {
            appended.push_str(newline);
        }
        appended.push_str(&full_block);
        appended.push_str(newline);
        appended
    };

    if !ctx.dry_run {
        crate::state::write_atomic(&agents_file, new_content.as_bytes())?;

        // Create derived stub (e.g. CLAUDE.md) if missing
        let claude_stub = target_dir.join("CLAUDE.md");
        if !claude_stub.exists() {
            let stub_content = format!("@AGENTS.md{}", newline);
            crate::state::write_atomic(&claude_stub, stub_content.as_bytes())?;
        }

        // Update state.json registry
        let global_state_path = ctx.config_dir.join("state.json");
        let mut state = State::load(&global_state_path)?;

        let now = chrono::Utc::now().to_rfc3339();
        let mut entry = ProjectAdoptionEntry {
            path: target_dir.clone(),
            file: "AGENTS.md".into(),
            tier,
            block_version: BLOCK_VERSION,
            block_sha256: body_sha256.clone(),
            created_file: !file_existed,
            adopted_at: now,
        };

        match state.projects.iter().position(|p| p.path == target_dir) {
            Some(pos) => {
                // Preserve who originally created the file: an upgrade
                // re-run replaces the entry, and deinit-prj relies on this
                // flag to clean up agent-created AGENTS.md/CLAUDE.md.
                entry.created_file = state.projects[pos].created_file;
                state.projects[pos] = entry;
            }
            None => {
                state.projects.push(entry);
            }
        }
        // Inject sentinel-bounded .gitignore block (DEC-06)
        let gitignore_file = target_dir.join(".gitignore");
        let gitignore_block = format!(
            "{}\n.ce-ai/skills-registry.json\n{}\n",
            GITIGNORE_BEGIN_MARKER, GITIGNORE_END_MARKER
        );
        let gitignore_text = if gitignore_file.exists() {
            fs::read_to_string(&gitignore_file)?
        } else {
            String::new()
        };
        if !gitignore_text.contains(GITIGNORE_BEGIN_MARKER) {
            let mut updated_gi = gitignore_text;
            if !updated_gi.is_empty() && !updated_gi.ends_with('\n') {
                updated_gi.push('\n');
            }
            updated_gi.push_str(&gitignore_block);
            crate::state::write_atomic(&gitignore_file, updated_gi.as_bytes())?;
        }

        if let Err(e) = crate::source::registry::SkillRegistry::sync_registry(ctx) {
            if !ctx.quiet {
                eprintln!("warning: skill registry sync failed: {e}");
            }
        }

        state.save(&global_state_path)?;
    }

    if !ctx.quiet {
        println!(
            "✓ Adopted project at '{}' (tier: {}, block SHA: {})",
            target_dir.display(),
            tier_str.to_lowercase(),
            &body_sha256[..8]
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_check_adoption_block_status_file_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("AGENTS.md");
        assert_eq!(
            check_adoption_block_status(&path, AdoptionTier::Full),
            AdoptionBlockStatus::FileMissing
        );
    }

    #[test]
    fn test_check_adoption_block_status_block_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("AGENTS.md");
        fs::write(&path, "# Hello World\n").unwrap();
        assert_eq!(
            check_adoption_block_status(&path, AdoptionTier::Full),
            AdoptionBlockStatus::BlockMissing
        );
    }

    #[test]
    fn test_check_adoption_block_status_stale_version() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("AGENTS.md");
        let content = format!(
            "{} v=1\nOld Content\n{}\n",
            BLOCK_BEGIN_MARKER, BLOCK_END_MARKER
        );
        fs::write(&path, content).unwrap();
        assert_eq!(
            check_adoption_block_status(&path, AdoptionTier::Full),
            AdoptionBlockStatus::StaleVersion { version: 1 }
        );
    }
}
