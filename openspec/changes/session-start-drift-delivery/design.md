# Technical Design: Session-Start Hook & Enforced Drift Delivery

## System Architecture

```
                               ce-ai init-prj
                                     │
         ┌───────────────────────────┴───────────────────────────┐
         ▼                                                       ▼
.claude/settings.json                                        AGENTS.md
SessionStart hook injected                                 v4 Block with Turn-0 Directive
(command: "ce-ai workflow resume")                         Mandatory prompt directive
         │                                                       │
         ▼                                                       ▼
[Claude Code SessionStart]                                 [Other Harness Runtimes]
Hooks trigger runs workflow resume                         Agent executes workflow resume
         │                                                       │
         └───────────────────────────┬───────────────────────────┘
                                     ▼
                     Turn-0 Canonical RepoState Injected
                                     │
                       ce-ai workflow checkpoint
                                     │
                     probe_repo_state checks drift
                                     ▼
                   Non-blocking Warnings if Desynced
```

## Detailed Interfaces & Logic

### 1. Claude Code Settings Hooks (`src/harness/claude.rs`)

```rust
pub const RESUME_COMMAND: &str = "ce-ai workflow resume";

/// Non-destructively injects the SessionStart hook into `.claude/settings.json`.
/// Returns Ok(true) if modified or created.
pub fn ensure_session_start_hook(settings_path: &Path) -> Result<bool, CeError>;

/// Surgically removes the `ce-ai workflow resume` hook from `.claude/settings.json`.
/// If the file becomes empty `{}` and was created by ce-ai, deletes the file.
pub fn remove_session_start_hook(settings_path: &Path) -> Result<bool, CeError>;

/// Checks if `.claude/settings.json` contains the `ce-ai workflow resume` hook.
pub fn has_session_start_hook(settings_path: &Path) -> bool;
```

#### JSON Manipulation Contract
- Read existing file if present; if absent, start with `{ "hooks": { "SessionStart": [] } }`.
- Find or create `hooks.SessionStart` array.
- Check if any item has `hooks[].command == "ce-ai workflow resume"`.
- If missing, append:
  ```json
  {
    "matcher": ".*",
    "hooks": [
      {
        "type": "command",
        "command": "ce-ai workflow resume"
      }
    ]
  }
  ```
- Write back atomically via `crate::state::write_atomic`.

### 2. Adoption Block Template & Version (`src/commands/init_prj.rs`)
- Bump `BLOCK_VERSION: u32 = 4;`.
- Update `render_block_content(AdoptionTier::Full)` to include:
  ```markdown
  ### ⚡ Turn-0 Session Directives (Zero-Step Drift Recovery)
  At the start of EVERY session or after context compaction, before running any task or reading historical chat assumptions, the AI agent MUST run:
  `ce-ai workflow resume`
  to synchronize live Git working tree state, active branch, manifest SHA256 integrity, and active OpenSpec progress.
  ```

### 3. Checkpoint Verification Gate (`src/commands/workflow.rs`)
- In `checkpoint_lines`:
  ```rust
  let repo_root = ctx.repo_root();
  let repo_state = probe_repo_state(&repo_root);
  if repo_state.manifest_drift_count > 0 {
      lines.push(format!(
          "! Warning: Drift detected in {} managed files. Run 'ce-ai sync' to reconcile.",
          repo_state.manifest_drift_count
      ));
  }
  ```

### 4. Health Check Probing (`src/commands/doctor.rs`)
- In `run`:
  ```rust
  for project in &state.projects {
      let claude_dir = project.path.join(".claude");
      if claude_dir.exists() {
          let settings = claude_dir.join("settings.json");
          if !crate::harness::claude::has_session_start_hook(&settings) {
              findings.push(format!(
                  "claude-hook-missing: Claude Code SessionStart hook missing at '{}' — re-run ce-ai init-prj to configure",
                  settings.display()
              ));
          }
      }
  }
  ```
