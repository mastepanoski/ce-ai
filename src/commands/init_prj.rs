//! Project adoption init subcommand: `ce-ai init-prj`.

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::{AdoptionTier, ProjectAdoptionEntry, State};
use crate::state::{report_best_effort_remove, report_best_effort_write};

use serde::{Deserialize, Serialize};

pub const BLOCK_BEGIN_MARKER: &str = "<!-- ce-ai:block begin";
pub const BLOCK_END_MARKER: &str = "<!-- ce-ai:block end -->";
pub const GITIGNORE_BEGIN_MARKER: &str = "# BEGIN CE-AI MANAGED BLOCK";
pub const GITIGNORE_END_MARKER: &str = "# END CE-AI MANAGED BLOCK";

/// Managed block schema version, shared by the on-disk header and the
/// `state.json` adoption entry so the two cannot drift apart.
pub const BLOCK_VERSION: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

### ⚡ Turn-0 Session Directives (Zero-Step Drift Recovery)
At the start of EVERY new session or after context compaction, before running any task or reading historical chat assumptions, the AI agent MUST run:
`ce-ai workflow resume`
to synchronize live Git working tree state, active branch, manifest SHA256 integrity, and active OpenSpec progress.

### Stage 2 OpenSpec Enforcement Requirements
Before creating PRs or writing feature code, agents MUST verify `openspec/changes/<feature_name>/` contains:
- `proposal.md`: Problem statement, in-scope/out-of-scope boundaries, and success criteria.
- `exploration.md`: Technical investigation and architectural tradeoffs.
- `design.md`: Technical design, system architecture, structs, and API/CLI contracts.
- `spec.md`: Formal requirements using `WHEN ... THEN ...` format and explicit acceptance criteria.
- `tasks.md`: Atomic, executable task checklist with TDD verification steps.

### Single Source of Truth Rule
Ideation artifacts (`docs/brainstorms/*.md`, `docs/ideation/*.md`) are disposable inputs, NOT parallel specifications. Distill their conclusions into the OpenSpec files above (`proposal.md`, `exploration.md`) and reference the source doc instead of copying content. Never maintain brainstorm/ideation documents in sync with OpenSpec. Ideation artifacts are retained by default as the permanent raw-history record OpenSpec intentionally does not duplicate; "disposable" never means deleting them. Skip ideation skills entirely when requirements and approach are already clear."#
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
- Ideation outputs (`docs/brainstorms/`, `docs/ideation/`) are disposable inputs: distill them into the specs before delegation; never maintain them in parallel; retain them as raw history instead of deleting them."#
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
    skip_rtk: bool,
    skip_companions: bool,
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
    let (new_content, is_already_up_to_date) =
        if let Some(start_idx) = existing_content.find(BLOCK_BEGIN_MARKER) {
            if let Some(end_rel_idx) = existing_content[start_idx..].find(BLOCK_END_MARKER) {
                let end_idx = start_idx + end_rel_idx + BLOCK_END_MARKER.len();
                let existing_block = &existing_content[start_idx..end_idx];

                let up_to_date = existing_block == full_block && !force;

                let mut updated = String::new();
                updated.push_str(&existing_content[..start_idx]);
                updated.push_str(&full_block);
                updated.push_str(&existing_content[end_idx..]);
                (updated, up_to_date)
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
            (appended, false)
        };

    if !ctx.dry_run {
        if !is_already_up_to_date {
            crate::state::write_atomic(&agents_file, new_content.as_bytes())?;
        }

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
        ensure_gitignore_block(&target_dir)?;

        // Reconcile harness project rules and hooks across all supported harnesses
        reconcile_project_harness_hooks(&target_dir, inner_body)?;

        if let Err(e) = crate::source::registry::SkillRegistry::sync_registry(ctx) {
            if !ctx.quiet {
                eprintln!("warning: skill registry sync failed: {e}");
            }
        }

        init_codegraph_if_available(&target_dir, ctx.quiet);

        // Auto-configure RTK hook injection for detected supported harnesses unless opted out
        if !crate::harness::rtk::is_rtk_opted_out(skip_rtk, skip_companions) {
            reconcile_rtk_hooks_if_supported(&target_dir, &state, ctx)?;
        } else if !ctx.quiet {
            println!("rtk: hook injection skipped (opted out)");
        }

        state.save(&global_state_path)?;
    }

    if is_already_up_to_date {
        if !ctx.quiet {
            println!(
                "Project at '{}' is already adopted with up-to-date block (SHA: {}).",
                target_dir.display(),
                &body_sha256[..8]
            );
        }
        return Ok(());
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

/// Ensures the sentinel-bounded .gitignore block exists and contains both
/// `.ce-ai/skills-registry.json` and `compound-engineering/`.
pub fn ensure_gitignore_block(target_dir: &Path) -> Result<(), CeError> {
    let gitignore_file = target_dir.join(".gitignore");
    let gitignore_block = format!(
        "{}\n.ce-ai/skills-registry.json\ncompound-engineering/\n{}\n",
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
    } else if !gitignore_text.contains("compound-engineering/") {
        if let Some(start_idx) = gitignore_text.find(GITIGNORE_BEGIN_MARKER) {
            if let Some(end_rel) = gitignore_text[start_idx..].find(GITIGNORE_END_MARKER) {
                let end_idx = start_idx + end_rel + GITIGNORE_END_MARKER.len();
                let mut updated_gi = String::new();
                updated_gi.push_str(&gitignore_text[..start_idx]);
                updated_gi.push_str(gitignore_block.trim_end());
                updated_gi.push_str(&gitignore_text[end_idx..]);
                crate::state::write_atomic(&gitignore_file, updated_gi.as_bytes())?;
            }
        }
    }
    Ok(())
}

fn init_codegraph_if_available(target_dir: &Path, quiet: bool) {
    let codegraph_dir = target_dir.join(".codegraph");
    if codegraph_dir.exists() {
        return;
    }

    let is_codegraph_available = std::process::Command::new("codegraph")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);

    if !is_codegraph_available {
        return;
    }

    if !quiet {
        println!(
            "init-prj: initializing CodeGraph index in '{}'...",
            target_dir.display()
        );
    }

    match std::process::Command::new("codegraph")
        .arg("init")
        .arg(target_dir)
        .status()
    {
        Ok(status) if status.success() => {
            if !quiet {
                println!("✓ Initialized CodeGraph index (.codegraph/)");
            }
        }
        Ok(status) => {
            if !quiet {
                eprintln!("warning: 'codegraph init' exited with status {status}");
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("warning: failed to run 'codegraph init': {e}");
            }
        }
    }
}

fn reconcile_rtk_hooks_if_supported(
    target_dir: &Path,
    state: &State,
    ctx: &Context,
) -> Result<(), CeError> {
    use crate::harness::HarnessKind;
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| ctx.config_dir.clone());

    let has_installed = |name: &str| {
        state
            .installed_harnesses
            .iter()
            .any(|h| h["name"].as_str() == Some(name))
    };

    if target_dir.join(".cursor").exists() || has_installed("cursor") {
        crate::harness::rtk::configure_rtk_hook(
            &home,
            HarnessKind::Cursor,
            ctx.dry_run,
            ctx.quiet,
        )?;
    }
    if target_dir.join(".claude").exists()
        || target_dir.join("CLAUDE.md").exists()
        || has_installed("claude")
    {
        crate::harness::rtk::configure_rtk_hook(
            &home,
            HarnessKind::Claude,
            ctx.dry_run,
            ctx.quiet,
        )?;
    }
    if target_dir.join(".codex").exists() || has_installed("codex") {
        crate::harness::rtk::configure_rtk_hook(&home, HarnessKind::Codex, ctx.dry_run, ctx.quiet)?;
    }
    if target_dir.join(".github").exists() || has_installed("copilot") {
        crate::harness::rtk::configure_rtk_hook(
            &home,
            HarnessKind::Copilot,
            ctx.dry_run,
            ctx.quiet,
        )?;
    }

    Ok(())
}

/// Reconciles harness project rules and hooks across all supported harnesses for an adopted project.
pub fn reconcile_project_harness_hooks(target_dir: &Path, inner_body: &str) -> Result<(), CeError> {
    // 1. Cursor
    let cursor_dir = target_dir.join(".cursor");
    if cursor_dir.exists() {
        let cursor_rules_dir = cursor_dir.join("rules");
        fs::create_dir_all(&cursor_rules_dir)?;
        let rule_path = cursor_rules_dir.join("compound-engineering.mdc");
        let frontmatter = crate::harness::cursor::CursorRuleFrontmatter::default();
        crate::harness::cursor::update_cursor_rule_mdc(&rule_path, &frontmatter, inner_body)?;

        // Ensure Cursor hooks (sessionStart, stop)
        let cursor_hooks = cursor_dir.join("hooks.json");
        crate::harness::cursor::ensure_session_start_hook(&cursor_hooks)?;
    }

    // 2. Claude
    let claude_dir = target_dir.join(".claude");
    let claude_md_root = target_dir.join("CLAUDE.md");
    let has_user_claude_md = claude_md_root.exists() && {
        let text = fs::read_to_string(&claude_md_root).unwrap_or_default();
        text.trim() != "@AGENTS.md"
    };
    if claude_dir.exists() || has_user_claude_md {
        let claude_rule_path = if claude_md_root.exists() {
            claude_md_root
        } else {
            claude_dir.join("CLAUDE.md")
        };
        crate::harness::claude::update_claude_md(&claude_rule_path, inner_body)?;

        let settings_path = target_dir.join(".claude").join("settings.json");
        let _ = crate::harness::claude::ensure_session_start_hook(&settings_path);
    }

    // 3. Codex
    let codex_dir = target_dir.join(".codex");
    if codex_dir.exists() {
        let codex_rule_path = codex_dir.join("AGENTS.md");
        crate::harness::codex::update_codex_agents_md(&codex_rule_path, inner_body)?;

        let codex_config_path = codex_dir.join("config.toml");
        crate::harness::codex::ensure_session_start_hook(&codex_config_path)?;
    }

    // 4. Copilot
    let github_dir = target_dir.join(".github");
    let copilot_md_path = github_dir.join("copilot-instructions.md");
    if github_dir.exists() || copilot_md_path.exists() {
        fs::create_dir_all(&github_dir)?;
        crate::harness::copilot::update_copilot_instructions_md(&copilot_md_path, inner_body)?;

        let copilot_hooks_dir = github_dir.join("hooks");
        let copilot_hooks_file = copilot_hooks_dir.join("hooks.json");
        crate::harness::copilot::ensure_session_start_hook(&copilot_hooks_file)?;
    }

    // 5. Grok
    let grok_dir = target_dir.join(".grok");
    if grok_dir.exists() {
        let grok_rules_dir = grok_dir.join("rules");
        fs::create_dir_all(&grok_rules_dir)?;
        let grok_rule_path = grok_rules_dir.join("compound-engineering.md");
        crate::harness::update_managed_rule_md(&grok_rule_path, inner_body)?;
    }

    // 6. Kimi
    let kimi_dir = target_dir.join(".kimi-code");
    if kimi_dir.exists() {
        let kimi_agents = kimi_dir.join("AGENTS.md");
        crate::harness::update_managed_rule_md(&kimi_agents, inner_body)?;

        // Clean up legacy .kimi-code/rules/compound-engineering.md if present
        let legacy_rule = kimi_dir.join("rules").join("compound-engineering.md");
        if legacy_rule.exists() {
            if let Ok(text) = fs::read_to_string(&legacy_rule) {
                if text.contains(crate::harness::CE_MANAGED_BEGIN) {
                    let stripped = crate::harness::strip_managed_rule_block(&text);
                    if stripped.trim().is_empty() {
                        report_best_effort_remove(&legacy_rule, fs::remove_file(&legacy_rule));
                        report_best_effort_remove(
                            kimi_dir.join("rules"),
                            fs::remove_dir(kimi_dir.join("rules")),
                        );
                    } else {
                        report_best_effort_write(
                            &legacy_rule,
                            crate::state::write_atomic(&legacy_rule, stripped.as_bytes()),
                        );
                    }
                }
            }
        }
    }

    // 7. Pi
    let pi_dir = target_dir.join(".pi");
    if pi_dir.exists() {
        let pi_agents = pi_dir.join("AGENTS.md");
        crate::harness::update_managed_rule_md(&pi_agents, inner_body)?;

        let ext_path = pi_dir
            .join("extensions")
            .join(crate::harness::pi::PI_EXTENSION_FILENAME);
        crate::harness::pi::ensure_session_start_hook(&ext_path)?;
    }

    // 8. fx
    let fx_dir = target_dir.join(".fx");
    if fx_dir.exists() {
        let fx_agents = fx_dir.join("AGENTS.md");
        crate::harness::update_managed_rule_md(&fx_agents, inner_body)?;
    }

    // 9. Antigravity & GEMINI.md
    let agents_dir = target_dir.join(".agents");
    if agents_dir.exists() {
        let agents_rules_dir = agents_dir.join("rules");
        fs::create_dir_all(&agents_rules_dir)?;
        let agy_rule_path = agents_rules_dir.join("compound-engineering.md");
        crate::harness::update_managed_rule_md(&agy_rule_path, inner_body)?;
        let agy_hooks = agents_dir.join("hooks.json");
        crate::harness::agy::ensure_pre_invocation_hook(&agy_hooks)?;
    }
    let gemini_md = target_dir.join("GEMINI.md");
    if gemini_md.exists() || (agents_dir.exists() && !gemini_md.exists()) {
        crate::harness::update_managed_rule_md(&gemini_md, inner_body)?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/init_prj.rs"]
mod tests;
