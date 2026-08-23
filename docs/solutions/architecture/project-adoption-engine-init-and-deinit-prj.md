---
module: commands/prj
date: "2026-08-22"
last_updated: "2026-08-22"
problem_type: architecture_pattern
category: architecture
component: tooling
severity: medium
applies_when:
  - Non-destructive adoption of ce-ai workflow instructions into existing projects using marker-delimited AGENTS.md blocks
  - Generating derived CLAUDE.md reference stubs (@AGENTS.md) without overwriting user instructions
  - Tracking project adoption state in state.json, exposing status/doctor health probes, and providing TUI shortcuts
tags:
  - project-adoption
  - init-prj
  - deinit-prj
  - agents-md
  - claude-md
  - harness-adapter
  - state-json
  - status-probe
  - doctor-probe
  - tui-shortcut
---

# Project Adoption Engine: Non-Destructive Multi-Harness Governance

## Context
When introducing AI agent governance (such as `AGENTS.md` instructions, 7-stage development cycle directives, and OpenSpec requirements) into software repositories, tools often face a conflict between enforcing standardized governance and respecting pre-existing project documentation. Pre-existing repositories frequently already contain custom developer guidelines, notes, or harness configuration files across up to 12 AI coding agent harnesses (e.g., Claude, Cursor, Codex, OpenCode).

Directly overwriting or mutating configuration files risks destroying user notes, custom agent prompts, or critical project instructions. Conversely, failing to adopt projects into unified governance leads to inconsistent agent behaviors, skipped verification cycles, and missing specifications.

To solve this, `ce-ai` introduced the **Project Adoption Engine** (`ce-ai init-prj` and `ce-ai deinit-prj`). The engine provides a non-destructive, fully reversible mechanism to adopt both new and pre-existing projects into Compound Engineering governance without clobbering existing developer content or breaking backward compatibility with existing state schemas.

## Guidance

When building managed configuration injection engines or project adoption tools, adhere to the following design principles:

### 1. Marker-Delimited Managed Blocks
- Use HTML comment markers to encapsulate injected instructions inside markdown files (`AGENTS.md`).
- Standardize the header format with explicit metadata attributes: `<!-- ce-ai:block begin v={version} tier={tier} sha256={sha} -->` and `<!-- ce-ai:block end -->`.
- Derive the version from a single shared constant (`pub const BLOCK_VERSION: u32 = 2;`) consumed by BOTH the on-disk header and the `state.json` entry (`block_version`) — two independent version literals drift silently (this bit us once: the header said `v=1` while the state literal was a separate `1`).
- HTML comment markers render invisibly in standard GitHub Flavored Markdown (GFM) viewers while remaining fully parsable by CLI tools.
- Include a cryptographic hash (`sha256`) of the managed block body within the marker to enable instant integrity verification and drift detection without re-rendering templates. Idempotent re-runs compare the whole rendered block, so content changes (e.g. block v2's Single Source of Truth guidance in `full`/`orchestrator` tiers) upgrade adopted projects in place — no migration command needed.

### 2. Derived Harness Stubs
- Automatically generate minimal derived stub files (e.g., `CLAUDE.md` containing `@AGENTS.md`) for sub-harnesses that support file inclusion/import primitives.
- Avoid duplicating managed content across multiple harness files. Canonical rules reside strictly in `AGENTS.md`, while sub-harnesses reference `AGENTS.md` via native `@` directives.

### 3. Reversible Operation & Atomic Restoration (`deinit-prj`)
- `ce-ai deinit-prj` must perform surgical extraction of the managed block between the begin and end markers, restoring surrounding content byte-for-byte (including preserving original line endings such as CRLF vs LF).
- Track whether `ce-ai` created the instruction file (`created_file: true` in state). If `deinit-prj` strips the block and the file was created by `ce-ai` and is now empty, automatically delete the file and any empty derived stubs (`CLAUDE.md`).
- **`created_file` is provenance, not current state**: upgrade re-runs that replace a registry entry must preserve the prior flag instead of recomputing it from `file_existed`. Recomputing flipped the flag to `false` for ce-ai-created files and left orphans behind after upgrade→deinit (see [the dedicated bug write-up](../logic-errors/init-prj-created-file-clobber-on-re-adoption-2026-08-22.md)).
- If pre-existing user content surrounded the block, preserve that content untouched upon de-initialization.

### 4. Backward Compatible State Schemas
- Extend global application state (`state.json`) with `pub projects: Vec<ProjectAdoptionEntry>`.
- Annotate new collection fields with Serde attributes: `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.
- Guarantee 100% backward and forward compatibility with legacy `state.json` versions without requiring database migrations or breaking older binaries.

### 5. Atomic Writes (`write_atomic`)
- Perform all filesystem mutations targeting `state.json`, `AGENTS.md`, or derived stubs using atomic write utilities (temporary file creation followed by atomic rename).
- Protect configuration files against corruption during unexpected process termination, power loss, or concurrent execution.

### 6. Observability & Health Probes
- Integrate adoption status into system observability commands:
  - `ce-ai status`: Report total adopted projects, individual project paths, adoption tiers, and SHA256 block integrity status.
  - `ce-ai doctor`: Run automated diagnostic probes checking for missing `AGENTS.md` files, unmanaged marker drift, or state file discrepancies.
- Expose interactive actions in TUI dashboards (e.g., `[I] Init Prj` shortcut).

## Why This Matters

1. **Data Integrity & Trust**: Developers can safely test and adopt `ce-ai` governance on established codebases without risking loss of custom notes, team guidelines, or manual setup.
2. **Multi-Harness Compatibility**: Centralizing rules in `AGENTS.md` and using derived stubs (`@AGENTS.md` in `CLAUDE.md`) ensures all 12 supported AI coding agent harnesses operate under unified rules without duplicating content.
3. **Zero Visual Clutter**: Marker-delimited blocks use HTML comments (`<!-- ... -->`) which render cleanly in GFM, IDE markdown previews, and web browsers, keeping documentation readable for human developers.
4. **Reliable Rollbacks**: Complete reversibility ensures zero lock-in; projects can be de-initialized at any time, leaving the workspace in its exact original state.

## When to Apply

Apply these patterns when:
- Designing CLI utilities that inject managed rules, policies, or boilerplate into user-owned documentation or configuration files.
- Implementing multi-agent governance across heterogenous tools that read different instruction file paths (`AGENTS.md`, `CLAUDE.md`, `.cursorrules`).
- Building stateful CLI engines where projects or workspaces are registered in a global tracking manifest (`state.json`).
- Adding feature blocks that require drift detection, integrity verification, and clean automated uninstallation.

## Examples

### Marker-Delimited Block Structure in `AGENTS.md`

```markdown
# My Pre-Existing Project Notes
User-written notes and repository docs remain untouched here.

<!-- ce-ai:block begin v=2 tier=full sha256=a1b2c3d4e5f6... -->
## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement

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
<!-- ce-ai:block end -->

Additional developer guidelines preserved after the block.
```

### State Schema (`src/state/state.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AdoptionTier {
    Full,
    Minimal,
    Orchestrator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectAdoptionEntry {
    pub path: PathBuf,
    pub tier: AdoptionTier,
    pub created_file: bool,
    pub block_version: u32,
    pub block_sha256: String,
    pub adopted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub active_profile: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<ProjectAdoptionEntry>,
}
```

### Adoption Subcommand (`src/commands/init_prj.rs`)

```rust
pub const BLOCK_BEGIN_MARKER: &str = "<!-- ce-ai:block begin";
pub const BLOCK_END_MARKER: &str = "<!-- ce-ai:block end -->";

/// Managed block schema version, shared by the on-disk header and the
/// state.json adoption entry so the two cannot drift apart.
pub const BLOCK_VERSION: u32 = 2;

pub fn run(
    ctx: &Context,
    target_path_opt: Option<PathBuf>,
    tier_str: &str,
    force: bool,
) -> Result<(), CeError> {
    let target_dir = resolve_target_dir(target_path_opt)?;
    let tier = parse_tier(tier_str)?;

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

    let new_content = inject_or_replace_block(&existing_content, &full_block, force)?;

    // Atomic write to AGENTS.md
    write_atomic(&agents_file, new_content.as_bytes())?;

    // Create derived harness stubs if missing (e.g. CLAUDE.md containing @AGENTS.md)
    let claude_stub = target_dir.join("CLAUDE.md");
    if !claude_stub.exists() {
        write_atomic(&claude_stub, b"@AGENTS.md\n")?;
    }

    // Register project in global state.json atomically. On replacement,
    // preserve the prior entry's created_file (provenance) instead of
    // recomputing it — see Guidance §3 and the logic-errors write-up.
    update_state_register_project(ctx, &target_dir, tier, !file_existed, body_sha256)?;

    Ok(())
}
```

### De-adoption Subcommand (`src/commands/deinit_prj.rs`)

> Error handling elided for brevity in this excerpt; the real implementation maps filesystem failures to `CeError` instead of discarding them.

```rust
pub fn run(ctx: &Context, target_path_opt: Option<PathBuf>) -> Result<(), CeError> {
    let target_dir = resolve_target_dir(target_path_opt)?;
    let agents_file = target_dir.join("AGENTS.md");

    let global_state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&global_state_path)?;

    let registry_pos = state.projects.iter().position(|p| p.path == target_dir);
    let created_file = registry_pos
        .map(|idx| state.projects[idx].created_file)
        .unwrap_or(false);

    if agents_file.exists() {
        let existing_content = fs::read_to_string(&agents_file)?;
        let cleaned_content = strip_managed_block(&existing_content)?;
        let is_empty_now = cleaned_content.trim().is_empty();

        if created_file && is_empty_now {
            let _ = fs::remove_file(&agents_file);

            // Clean up derived stub if created by ce-ai and untouched
            let claude_stub = target_dir.join("CLAUDE.md");
            if claude_stub.exists() {
                if let Ok(stub_text) = fs::read_to_string(&claude_stub) {
                    if stub_text.trim() == "@AGENTS.md" {
                        let _ = fs::remove_file(&claude_stub);
                    }
                }
            }
        } else {
            write_atomic(&agents_file, cleaned_content.as_bytes())?;
        }
    }

    // Unregister project from state.json
    if let Some(idx) = registry_pos {
        state.projects.remove(idx);
        state.save(&global_state_path)?;
    }

    Ok(())
}
```

## Related Documentation

- [Multi-Harness Support Implementation](file:///Users/mastepanoski/projects/web/ai/ce-ai/docs/solutions/multi-harness-support-implementation.md)
- [Workspace Configuration Overrides & Multi-Harness Uninstall Parity](file:///Users/mastepanoski/projects/web/ai/ce-ai/docs/solutions/architecture/workspace-configuration-overrides-and-multi-harness-uninstall.md)
- [Proactive Workflow Observability: TUI FSM Dashboard & Extended Doctor Health](file:///Users/mastepanoski/projects/web/ai/ce-ai/docs/solutions/architecture/proactive-workflow-observability-fsm-tui-sync-watcher.md)
