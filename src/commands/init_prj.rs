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
pub const BLOCK_VERSION: u32 = 9;

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
            r#"## 🔄 Mandatory 7-Stage Development Cycle & Architecture Guidelines

All AI agents MUST follow the 7-stage Compound Engineering development cycle:
`[Stage 1: Ideation]` ➔ `[Stage 2: Architecture & Deltas (Optional)]` ➔ `[Stage 3: Execution Plan]`
➔ `[Stage 4: TDD & Implementation]` ➔ `[Stage 5: Verification]` ➔ `[Stage 6: Knowledge Capture]`
➔ `[Stage 7: Git Shipping]`

### ⚡ Turn-0 Session Directives (Zero-Step Drift Recovery)
`ce-ai` auto-installs a SessionStart hook on supported harnesses that runs `ce-ai workflow resume --json` and injects live FSM state (Git working tree, active branch, manifest SHA256 integrity, plan progress) into context before your first turn. If that state is already present at session start, treat it as current — do NOT re-run the command. Only run `ce-ai workflow resume` yourself when no such context was injected (harness without hook support, or the hook is disabled) or when you suspect the injected state has gone stale mid-session (e.g. after a `git` operation you didn't make through your own tool calls).

### Stage 2 Living Architecture & Deltas (Optional)
Living architecture specifications reside under `docs/arch/`:
- `docs/arch/system/`: Living domain specifications.
- `docs/arch/adr/`: Architecture Decision Records (`NNNN-title.md`).
- `docs/arch/contracts/`: Formal data contracts and API interfaces.
- `docs/arch/deltas/`: Temporary feature changes during development.
- `docs/arch/archive/`: Archived changes and historical milestones.

CE-AI enforces zero semantic authority: architecture deltas are decoupled from the core workflow as an optional tool for architectural changes (> 200 LOC), schema changes, or cross-cutting features, rather than a mandatory write blocker. For bug fixes, minor tweaks, or tactical tasks (< 200 LOC), proceed directly with `/ce-plan` or `ce-work`.

When formal specifications are activated for complex features, OpenSpec is authored progressively in `docs/arch/deltas/<feature>/` (or legacy `openspec/changes/<feature>/`) to establish the frozen Stage 2 contract.
The execution plan and task checklist reside in `docs/plans/` (`- [x]` checkboxes) as the single source of truth for progress.

### Single Source of Truth Rule
Ideation artifacts (`docs/brainstorms/*.md`, `docs/ideation/*.md`) are disposable inputs, NOT parallel specifications. Distill their conclusions into architecture or plan docs and reference the source doc instead of copying content. Never maintain brainstorm/ideation documents in sync with architecture deltas. Ideation artifacts are retained by default as the permanent raw-history record that specifications intentionally do not duplicate; "disposable" never means deleting them. Skip ideation skills entirely when requirements and approach are already clear.

### 🚀 Self-Explaining Pull Requests & Review Readiness
When creating Pull Requests (Stage 7), agents MUST ensure the change explains itself:
- **What It Ruled Out**: State rejected alternatives and discarded trade-offs.
- **Which Rule Decided It**: Cite the specific invariant, rule, architectural boundary, or compliance constraint (from `AGENTS.md`) that determined the solution.
- **Upfront Evidence**: Attach empirical validation proof (test run summaries, CLI logs, reproduction traces, or browser check results) directly to the PR body using collapsible `<details><summary>` blocks (`not asked for later`).
- **Pre-Review Automated Checks**: Run and verify 100% green status across all local linters, unit tests, and browser/E2E checks BEFORE requesting human review or marking the PR ready.
- **Reviewer Routing**: Identify domain owners or `CODEOWNERS` and assign appropriate reviewers.

### 🧹 Post-Merge Lifecycle & Clean State
Immediately after merging a PR on `main`, agents MUST run `ce-ai workflow status` and archive completed changes via `ce-ai archive <feature>` to seal change packages and maintain zero workflow drift."#
        }
        AdoptionTier::Minimal => {
            r#"## 🔄 Compound Engineering Workflow Guidelines

AI agents operating on this codebase should follow structured planning and verification:
- Validate scope boundaries before making changes.
- Ensure all unit, integration, and linter tests pass before committing.
- Ensure PRs explain what was ruled out, which rule decided it, and attach verification evidence upfront in collapsible blocks.
- Document key technical learnings and post-mortem fixes."#
        }
        AdoptionTier::Orchestrator => {
            r#"## 🔄 Orchestrator Agent Governance & Delegation Directives

Orchestrator agents MUST delegate domain tasks to specialized subagents:
- Use `ce-brainstorm` for scope exploration.
- Use `ce-plan` for implementation unit breakdown.
- Use `ce-code-review` before opening Pull Requests.
- Enforce self-explaining PRs: mandate that subagents document rejected alternatives, governing rules, and attach empirical verification evidence in collapsible blocks before requesting review.
- Enforce strict PR CI status check gates before merging.
- Post-merge, ensure completed changes are sealed via `ce-ai archive <feature>`.
- Ideation outputs (`docs/brainstorms/`, `docs/ideation/`) are disposable inputs: distill them into specs or plans before delegation; never maintain them in parallel; retain them as raw history instead of deleting them."#
        }
    }
}

/// Compute SHA256 of string content.
pub fn compute_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Represents a legacy directive or FSM clause discovered outside a managed block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyPatternFinding {
    pub pattern_id: &'static str,
    pub description: &'static str,
    pub line: usize,
}

/// Extracts content outside of the managed block delimiters (or the entire content if no block exists).
pub fn extract_unmanaged_content(content: &str) -> String {
    if let Some(start_idx) = content.find(BLOCK_BEGIN_MARKER) {
        if let Some(end_rel_idx) = content[start_idx..].find(BLOCK_END_MARKER) {
            let end_idx = start_idx + end_rel_idx + BLOCK_END_MARKER.len();
            let mut unmanaged = String::with_capacity(content.len());
            unmanaged.push_str(&content[..start_idx]);
            unmanaged.push_str(&content[end_idx..]);
            return unmanaged;
        }
    }
    content.to_string()
}

/// Scans unmanaged markdown content for legacy Compound Engineering directives and rigid FSM clauses.
pub fn scan_legacy_directives(content: &str) -> Vec<LegacyPatternFinding> {
    let unmanaged = extract_unmanaged_content(content);
    let mut findings = Vec::new();

    for (line_idx, line) in unmanaged.lines().enumerate() {
        let line_num = line_idx + 1;
        if (line.contains("OpenSpec Required") && line.contains("NO code changes without"))
            || (line.trim_start().starts_with("6.") && line.contains("OpenSpec Required"))
        {
            findings.push(LegacyPatternFinding {
                pattern_id: "PAT-INV-6",
                description: "Legacy Invariant 6 enforcing mandatory OpenSpec",
                line: line_num,
            });
        } else if line.contains("Mandatory OpenSpec Before Code Changes")
            || (line.contains("NO code changes or feature implementations shall be made without creating or updating a formal spec"))
        {
            findings.push(LegacyPatternFinding {
                pattern_id: "PAT-CONSTRAINT-OPENSPEC",
                description: "Mandatory OpenSpec constraint before code changes",
                line: line_num,
            });
        } else if line.contains("[Stage 2: OpenSpec Definition (MANDATORY)]") {
            findings.push(LegacyPatternFinding {
                pattern_id: "PAT-FSM-7",
                description: "Mandatory Stage 2 in 7-stage development cycle diagram",
                line: line_num,
            });
        } else if line.contains("Stage 2 OpenSpec Enforcement Requirements") {
            findings.push(LegacyPatternFinding {
                pattern_id: "PAT-STAGE-2",
                description: "Stage 2 OpenSpec Enforcement heading",
                line: line_num,
            });
        } else if line.contains("models.rs") && line.contains("ce-ai models") {
            findings.push(LegacyPatternFinding {
                pattern_id: "PAT-CLI-MODELS",
                description: "Deprecated 'ce-ai models' CLI reference",
                line: line_num,
            });
        } else if line.contains("decisions.rs") && line.contains("model advisor") {
            findings.push(LegacyPatternFinding {
                pattern_id: "PAT-CLI-DECISIONS",
                description: "Deprecated 'ce-ai decisions' model advisor reference",
                line: line_num,
            });
        } else if line.contains("Ready (100%) with zero unarchived change warnings") {
            findings.push(LegacyPatternFinding {
                pattern_id: "PAT-FSM-READY",
                description: "Rigid FSM 100% Ready gate in post-merge lifecycle",
                line: line_num,
            });
        }
    }

    findings
}

/// Surgically modernizes legacy directives in the unmanaged portion of an AGENTS.md file.
pub fn migrate_legacy_directives(content: &str) -> (String, Vec<LegacyPatternFinding>) {
    let findings = scan_legacy_directives(content);
    if findings.is_empty() {
        return (content.to_string(), findings);
    }

    // Split around managed block if present so we only modify unmanaged portions
    let (prefix, managed_opt, suffix) = if let Some(start_idx) = content.find(BLOCK_BEGIN_MARKER) {
        if let Some(end_rel_idx) = content[start_idx..].find(BLOCK_END_MARKER) {
            let end_idx = start_idx + end_rel_idx + BLOCK_END_MARKER.len();
            (
                &content[..start_idx],
                Some(&content[start_idx..end_idx]),
                &content[end_idx..],
            )
        } else {
            (content, None, "")
        }
    } else {
        (content, None, "")
    };

    let transform_slice = |text: &str| -> String {
        let mut out = String::with_capacity(text.len());
        let mut lines = text.lines().peekable();

        while let Some(line) = lines.next() {
            // PAT-INV-6
            if (line.contains("OpenSpec Required") && line.contains("NO code changes without"))
                || (line.trim_start().starts_with("6.") && line.contains("OpenSpec Required"))
            {
                out.push_str("6. **No Semantic Authority & Optional OpenSpec**: CE-AI MUST NOT introduce mandatory workflow stages, artifacts, or blocking write gates beyond those defined by Compound Engineering contracts. OpenSpec is decoupled from the core domain as an optional, opt-in integration for high-complexity architectural changes, rather than an authoritarian write blocker.\n");
                continue;
            }

            // PAT-CONSTRAINT-OPENSPEC
            if line.contains("Mandatory OpenSpec Before Code Changes") {
                out.push_str("5. **No Semantic Authority & Optional OpenSpec Integration**:\n   - CE-AI must not require artifacts that Compound Engineering itself does not require. OpenSpec change packages (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`) are optional tools for complex architectural work or formal compliance, not mandatory blockers for bug fixes, docs, or routine tasks.\n");
                // Skip the immediate following bullet line if it describes the legacy requirement
                if let Some(next_line) = lines.peek() {
                    if next_line
                        .contains("NO code changes or feature implementations shall be made")
                    {
                        lines.next();
                    }
                }
                continue;
            }

            // PAT-FSM-7
            if line.contains("[Stage 2: OpenSpec Definition (MANDATORY)]") {
                let replaced = line.replace(
                    "[Stage 2: OpenSpec Definition (MANDATORY)]",
                    "[Stage 2: OpenSpec Definition (OPTIONAL)]",
                );
                out.push_str(&replaced);
                out.push('\n');
                continue;
            }

            // PAT-STAGE-2
            if line.contains("Stage 2 OpenSpec Enforcement Requirements") {
                out.push_str("### Stage 2 OpenSpec Integration (Optional)\n");
                continue;
            }
            if line.contains("Before creating any PR or writing feature code, agents MUST verify that `openspec/changes/<feature_name>/` contains all five files:") {
                out.push_str("When formal specifications are activated for complex or regulated features, OpenSpec is authored **progressively**: Stage 2 writes `proposal.md`, `exploration.md`, `design.md`, and `spec.md`; Stage 3 (`/ce-plan`) then generates the executable checklist `tasks.md` from the frozen contract. When OpenSpec is used, agents verify that `openspec/changes/<feature_name>/` contains:\n");
                continue;
            }

            // PAT-CLI-MODELS
            if line.contains("models.rs") && line.contains("ce-ai models") {
                out.push_str(
                    "    ├── fleet.rs       # ce-ai fleet pin/status/sync implementation\n",
                );
                continue;
            }

            // PAT-CLI-DECISIONS
            if line.contains("decisions.rs") && line.contains("model advisor") {
                out.push_str("    ├── guard.rs       # ce-ai guard agent guardrails & risk policy engine\n    ├── decisions.rs   # ce-ai decisions (legacy aliases & deprecated routing)\n");
                continue;
            }

            // PAT-FSM-READY
            if line.contains("so the repository FSM is left at ✓ Ready (100%) with zero unarchived change warnings before concluding") {
                let replaced = line.replace(
                    "so the repository FSM is left at ✓ Ready (100%) with zero unarchived change warnings before concluding",
                    "and archive completed OpenSpec changes via `ce-ai archive <feature>` to seal change packages and maintain zero workflow drift",
                );
                out.push_str(&replaced);
                out.push('\n');
                continue;
            }

            out.push_str(line);
            out.push('\n');
        }

        // Check if we migrated Stage 2 but Proactive Triage section is missing
        if out.contains("### Stage 2 OpenSpec Integration (Optional)")
            && !out.contains("Proactive OpenSpec Triage & Guidance for Developers")
        {
            let triage_block = r#"
### 💡 Proactive OpenSpec Triage & Guidance for Developers
AI agents MUST actively guide developers (especially onboarding/newbie contributors) on when to use OpenSpec:
- **When to Suggest OpenSpec**: If a proposed feature involves architectural changes, schema or data migrations, public API mutations, or estimated scope > 200 LOC, the agent MUST proactively ask the developer:
  > *"Esta funcionalidad tiene impacto arquitectónico o contratos (> 200 LOC). ¿Deseas que formalicemos una especificación OpenSpec (`openspec/changes/<feature>/`) para congelar el contrato antes de escribir código, o prefieres proceder con un plan ágil directo (`docs/plans/`)?"*
- **When to Bypass OpenSpec**: For bug fixes, minor tweaks, documentation updates, or tactical tasks (< 200 LOC), agents MUST NOT require or suggest OpenSpec; proceed directly with `/ce-plan`, `ce-work`, or `ce-debug`.
"#;
            if let Some(pos) = out.find("### Single Source of Truth Rule") {
                let mut with_triage = String::with_capacity(out.len() + triage_block.len());
                with_triage.push_str(&out[..pos]);
                with_triage.push_str(triage_block);
                with_triage.push('\n');
                with_triage.push_str(&out[pos..]);
                out = with_triage;
            } else {
                out.push_str(triage_block);
            }
        }

        out
    };

    let transformed_prefix = transform_slice(prefix);
    let transformed_suffix = if suffix.is_empty() {
        String::new()
    } else {
        transform_slice(suffix)
    };

    let mut result = String::with_capacity(content.len() + 512);
    result.push_str(&transformed_prefix);
    if let Some(managed) = managed_opt {
        result.push_str(managed);
        result.push_str(&transformed_suffix);
    }

    if content.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }

    (result, findings)
}

/// Adopts or upgrades a single project directory.
pub fn adopt_project(
    ctx: &Context,
    target_dir: &Path,
    tier: AdoptionTier,
    force: bool,
    migrate: bool,
    skip_rtk: bool,
    skip_companions: bool,
) -> Result<(), CeError> {
    if !target_dir.exists() || !target_dir.is_dir() {
        return Err(CeError::Usage(format!(
            "target project path '{}' does not exist or is not a directory",
            target_dir.display()
        )));
    }

    let agents_file = target_dir.join("AGENTS.md");
    let file_existed = agents_file.exists();

    let (existing_content, is_crlf) = if file_existed {
        let text = fs::read_to_string(&agents_file)?;
        let crlf = text.contains("\r\n");
        (text, crlf)
    } else {
        (String::new(), false)
    };

    let findings = scan_legacy_directives(&existing_content);
    let mut working_content = existing_content.clone();

    if migrate && !findings.is_empty() {
        if ctx.dry_run {
            if !ctx.quiet {
                println!(
                    "dry-run: found {} legacy directive(s) in '{}':",
                    findings.len(),
                    agents_file.display()
                );
                for f in &findings {
                    println!("  - [L{}] {}: {}", f.line, f.pattern_id, f.description);
                }
            }
        } else {
            let backup_file = target_dir.join("AGENTS.md.bak");
            crate::state::write_atomic(&backup_file, existing_content.as_bytes())?;
            if !ctx.quiet {
                println!(
                    "✓ Created backup of legacy directives at '{}'",
                    backup_file.display()
                );
            }
            let (migrated, _) = migrate_legacy_directives(&existing_content);
            working_content = migrated;
            if !ctx.quiet {
                println!(
                    "✓ Modernized {} legacy directive(s) in '{}'",
                    findings.len(),
                    agents_file.display()
                );
            }
        }
    } else if !findings.is_empty() && !ctx.quiet {
        eprintln!(
            "hint: {} legacy unmanaged directive(s) detected in '{}'. Re-run with --migrate to modernize them.",
            findings.len(),
            agents_file.display()
        );
    }

    let inner_body = render_block_content(tier);
    let body_sha256 = compute_sha256(inner_body);

    let newline = if is_crlf { "\r\n" } else { "\n" };

    let block_header = format!(
        "<!-- ce-ai:block begin v={} tier={} sha256={} -->",
        BLOCK_VERSION,
        tier.as_str(),
        body_sha256
    );
    let full_block = format!(
        "{}{}{}{}{}",
        block_header, newline, inner_body, newline, BLOCK_END_MARKER
    );

    // Check if block already exists
    let (new_content, is_already_up_to_date) =
        if let Some(start_idx) = working_content.find(BLOCK_BEGIN_MARKER) {
            if let Some(end_rel_idx) = working_content[start_idx..].find(BLOCK_END_MARKER) {
                let end_idx = start_idx + end_rel_idx + BLOCK_END_MARKER.len();
                let existing_block = &working_content[start_idx..end_idx];

                let up_to_date =
                    existing_block == full_block && !force && (!migrate || findings.is_empty());

                let mut updated = String::new();
                updated.push_str(&working_content[..start_idx]);
                updated.push_str(&full_block);
                updated.push_str(&working_content[end_idx..]);
                (updated, up_to_date)
            } else {
                return Err(CeError::Runtime(format!(
                    "malformed managed block in '{}': found begin marker without end marker",
                    agents_file.display()
                )));
            }
        } else {
            let mut appended = working_content.clone();
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
            path: target_dir.to_path_buf(),
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
        ensure_gitignore_block(target_dir)?;

        // Reconcile harness project rules and hooks across all supported harnesses
        reconcile_project_harness_hooks(target_dir, inner_body)?;

        if let Err(e) = crate::source::registry::SkillRegistry::sync_registry(ctx) {
            if !ctx.quiet {
                eprintln!("warning: skill registry sync failed: {e}");
            }
        }

        init_codegraph_if_available(target_dir, ctx.quiet);

        // Auto-configure RTK hook injection for detected supported harnesses unless opted out
        if !crate::harness::rtk::is_rtk_opted_out(skip_rtk, skip_companions) {
            reconcile_rtk_hooks_if_supported(target_dir, &state, ctx, false)?;
        } else if !ctx.quiet {
            println!("rtk: hook injection skipped (opted out)");
        }

        state.save(&global_state_path)?;
    } else if !crate::harness::rtk::is_rtk_opted_out(skip_rtk, skip_companions) {
        // Dry-run preview: reconciliation is dry-run aware and writes nothing. A
        // real run guarantees the derived CLAUDE.md stub, so preview Claude as a
        // candidate even on a fresh project. A best-effort registry read keeps a
        // corrupt state.json from aborting a preview that would not write.
        let state = match State::load(&ctx.config_dir.join("state.json")) {
            Ok(loaded) => loaded,
            Err(e) => {
                if !ctx.quiet {
                    eprintln!("warning: {e}; previewing without registry context");
                }
                State::default()
            }
        };
        reconcile_rtk_hooks_if_supported(target_dir, &state, ctx, true)?;
    } else if !ctx.quiet {
        println!("rtk: hook injection skipped (opted out)");
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

    if ctx.dry_run {
        if !ctx.quiet {
            if migrate && !findings.is_empty() {
                println!(
                    "dry-run: would modernize {} legacy directive(s) and adopt project at '{}' (tier: {}, block SHA: {})",
                    findings.len(),
                    target_dir.display(),
                    tier.as_str(),
                    &body_sha256[..8]
                );
            } else {
                println!(
                    "dry-run: would adopt project at '{}' (tier: {}, block SHA: {})",
                    target_dir.display(),
                    tier.as_str(),
                    &body_sha256[..8]
                );
            }
        }
        return Ok(());
    }

    if !ctx.quiet {
        println!(
            "✓ Adopted project at '{}' (tier: {}, block SHA: {})",
            target_dir.display(),
            tier.as_str(),
            &body_sha256[..8]
        );
    }

    Ok(())
}

/// Checks if the git working tree in `repo_root` is clean.
pub fn check_git_working_tree_clean(repo_root: &Path) -> bool {
    let mut cmd = std::process::Command::new("git");
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        cmd.env_remove(var);
    }
    let out = match cmd
        .args(["status", "--porcelain=v1"])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return true,
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout.trim().is_empty()
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dest_child = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_child)?;
        } else {
            fs::copy(entry.path(), dest_child)?;
        }
    }
    Ok(())
}

fn move_path_or_copy(src: &Path, dest: &Path) -> std::io::Result<()> {
    if fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    if src.is_dir() {
        copy_dir_recursive(src, dest)?;
        fs::remove_dir_all(src)?;
    } else {
        fs::copy(src, dest)?;
        fs::remove_file(src)?;
    }
    Ok(())
}

/// Migrates a repository from legacy openspec/ structure to modern docs/arch/ living architecture model.
pub fn migrate_arch_repository(
    ctx: &Context,
    repo_root: &Path,
    force: bool,
) -> Result<(), CeError> {
    // 1. Working tree cleanliness check
    if !force && !check_git_working_tree_clean(repo_root) {
        return Err(CeError::Usage(
            "git working tree is dirty; commit or stash changes before running --migrate-arch, or pass --force"
                .to_string(),
        ));
    }

    let openspec_dir = repo_root.join("openspec");
    if !openspec_dir.is_dir() {
        if !ctx.quiet {
            println!(
                "no legacy 'openspec/' directory found in '{}'",
                repo_root.display()
            );
        }
        return Ok(());
    }

    let arch_dir = repo_root.join("docs").join("arch");
    let system_dir = arch_dir.join("system");
    let adr_dir = arch_dir.join("adr");
    let archive_dir = arch_dir.join("archive");
    let deltas_dir = arch_dir.join("deltas");

    if ctx.dry_run {
        if !ctx.quiet {
            println!(
                "[dry-run] Planned architecture migration in '{}':",
                repo_root.display()
            );
            println!("  create directory: {}", system_dir.display());
            println!("  create directory: {}", adr_dir.display());
            println!("  create directory: {}", archive_dir.display());
            println!("  create directory: {}", deltas_dir.display());
            if openspec_dir.join("specs").is_dir() {
                println!(
                    "  move specs: {} -> {}",
                    openspec_dir.join("specs").display(),
                    system_dir.display()
                );
            }
            if openspec_dir.join("changes").join("archive").is_dir() {
                println!(
                    "  move archive: {} -> {}",
                    openspec_dir.join("changes").join("archive").display(),
                    archive_dir.display()
                );
            }
            if openspec_dir.join("changes").is_dir() {
                println!(
                    "  move active changes: {} -> {}",
                    openspec_dir.join("changes").display(),
                    deltas_dir.display()
                );
            }
            println!("  remove legacy directory: {}", openspec_dir.display());
        }
        return Ok(());
    }

    // 2. Create destination directories
    fs::create_dir_all(&system_dir)?;
    fs::create_dir_all(&adr_dir)?;
    fs::create_dir_all(&archive_dir)?;
    fs::create_dir_all(&deltas_dir)?;

    // 3. Move openspec/specs/* -> docs/arch/system/
    let specs_dir = openspec_dir.join("specs");
    if specs_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&specs_dir) {
            for entry in entries.flatten() {
                let dest = system_dir.join(entry.file_name());
                move_path_or_copy(&entry.path(), &dest)?;
            }
        }
    }

    // 4. Move openspec/changes/archive/* -> docs/arch/archive/
    let archive_src = openspec_dir.join("changes").join("archive");
    if archive_src.is_dir() {
        if let Ok(entries) = fs::read_dir(&archive_src) {
            for entry in entries.flatten() {
                let dest = archive_dir.join(entry.file_name());
                move_path_or_copy(&entry.path(), &dest)?;
            }
        }
        let _ = fs::remove_dir_all(&archive_src);
    }

    // 5. Move remaining openspec/changes/* -> docs/arch/deltas/
    let changes_src = openspec_dir.join("changes");
    if changes_src.is_dir() {
        if let Ok(entries) = fs::read_dir(&changes_src) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str == "archive" || name_str.starts_with('.') {
                    continue;
                }
                let dest = deltas_dir.join(&name);
                move_path_or_copy(&entry.path(), &dest)?;
            }
        }
    }

    // 6. Remove openspec/ directory
    fs::remove_dir_all(&openspec_dir)?;

    if !ctx.quiet {
        println!(
            "✓ Migrated legacy 'openspec/' to 'docs/arch/' in '{}'",
            repo_root.display()
        );
    }

    Ok(())
}

/// Options controlling project adoption and migration behavior.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InitPrjOptions {
    pub tier: Option<String>,
    pub all: bool,
    pub force: bool,
    pub migrate: bool,
    pub migrate_arch: bool,
    pub skip_rtk: bool,
    pub skip_companions: bool,
}

/// Executes `ce-ai init-prj`.
pub fn run(
    ctx: &Context,
    target_path_opt: Option<PathBuf>,
    options: InitPrjOptions,
) -> Result<(), CeError> {
    if options.all {
        let global_state_path = ctx.config_dir.join("state.json");
        let state = State::load(&global_state_path)?;
        if state.projects.is_empty() {
            if !ctx.quiet {
                println!("no registered projects found in state.json");
            }
            return Ok(());
        }
        let parsed_tier = match options.tier.as_deref() {
            Some(t) => Some(match t.to_lowercase().as_str() {
                "full" => AdoptionTier::Full,
                "minimal" => AdoptionTier::Minimal,
                "orchestrator" => AdoptionTier::Orchestrator,
                unknown => {
                    return Err(CeError::Usage(format!(
                        "unknown adoption tier '{}'. Supported tiers: full, minimal, orchestrator",
                        unknown
                    )))
                }
            }),
            None => None,
        };

        let mut count = 0;
        for p in &state.projects {
            if options.migrate_arch {
                migrate_arch_repository(ctx, &p.path, options.force)?;
            }
            let tier = parsed_tier.unwrap_or(p.tier);
            adopt_project(
                ctx,
                &p.path,
                tier,
                options.force,
                options.migrate,
                options.skip_rtk,
                options.skip_companions,
            )?;
            count += 1;
        }
        if !ctx.quiet {
            println!(
                "successfully updated {count} registered project(s) to block v={BLOCK_VERSION}"
            );
        }
        return Ok(());
    }

    let raw_target = match target_path_opt {
        Some(p) => p,
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let target_dir = match raw_target.canonicalize() {
        Ok(canonical) => canonical,
        Err(_) => raw_target,
    };

    let tier_str = options.tier.as_deref().unwrap_or("full");
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

    if options.migrate_arch {
        migrate_arch_repository(ctx, &target_dir, options.force)?;
    }

    adopt_project(
        ctx,
        &target_dir,
        tier,
        options.force,
        options.migrate,
        options.skip_rtk,
        options.skip_companions,
    )
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
    claude_rule_expected: bool,
) -> Result<(), CeError> {
    use crate::harness::HarnessKind;
    let home = crate::harness::home_dir_from_ctx(ctx);

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
        || claude_rule_expected
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
    let claude_md_nested = claude_dir.join("CLAUDE.md");
    let target_agents_md = target_dir.join("AGENTS.md");

    let claude_rule_path_opt = if claude_md_root.exists() {
        Some(claude_md_root)
    } else if claude_md_nested.exists() {
        Some(claude_md_nested)
    } else if claude_dir.exists() {
        Some(claude_dir.join("CLAUDE.md"))
    } else {
        None
    };

    if let Some(claude_rule_path) = claude_rule_path_opt {
        let existing_content = if claude_rule_path.exists() {
            fs::read_to_string(&claude_rule_path).unwrap_or_default()
        } else {
            String::new()
        };

        let delegates = target_agents_md.exists()
            && crate::harness::claude::delegates_to_agents_md(&existing_content);

        if delegates {
            // Claude Code already imports AGENTS.md, which contains the managed block.
            // If CLAUDE.md has a duplicate managed block (from prior versions or edits), strip it.
            if existing_content.contains(crate::harness::claude::CE_MANAGED_BEGIN)
                || existing_content.contains(BLOCK_BEGIN_MARKER)
            {
                let stripped = crate::harness::claude::strip_managed_block(&existing_content);
                let mut cleaned = stripped.trim_end().to_string();
                if !cleaned.is_empty() {
                    let newline = if existing_content.contains("\r\n") {
                        "\r\n"
                    } else {
                        "\n"
                    };
                    cleaned.push_str(newline);
                }
                if cleaned != existing_content {
                    crate::state::write_atomic(&claude_rule_path, cleaned.as_bytes())?;
                }
            }
        } else {
            // No delegation to AGENTS.md: only update if .claude exists or user has custom instructions
            let has_user_claude_md =
                claude_rule_path.exists() && !existing_content.trim().is_empty();
            if claude_dir.exists() || has_user_claude_md {
                crate::harness::claude::update_claude_md(&claude_rule_path, inner_body)?;
            }
        }
    }

    if claude_dir.exists() {
        let settings_path = claude_dir.join("settings.json");
        let _ = crate::harness::claude::ensure_session_start_hook(&settings_path);
        let _ = crate::harness::claude::ensure_claude_gate_hook(&settings_path);
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
