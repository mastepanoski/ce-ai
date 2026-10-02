# Design: CE-AI v2 Architectural Pivot & PRD

## Architectural Pivot Overview

CE-AI v2 shifts from an intrusive "workflow orchestrator and governance layer" to a **Cross-Host Operational Companion for Compound Engineering**.

```text
+-------------------------------------------------------------------+
|                           CE-AI v2                                |
|             Cross-Host Operational Companion                      |
+-------------------------------------------------------------------+
        |                                       |
        | Version Governance                    | Advisory Observation
        v                                       v
+-------------------------------+       +---------------------------+
| Native Host Installers        |       | Dynamic Workflow State    |
| (Claude, Codex, OpenCode, Pi) |       | (Git artifacts, PRs,      |
+-------------------------------+       | plans, receipts)          |
        |                               +---------------------------+
        v                                       |
+-------------------------------+               v
| Upstream Compound Engineering |       +---------------------------+
| Releases (vX.Y.Z)             |       | Zero-Drift Read-Only View |
+-------------------------------+       | (status, resume, doctor)  |
                                        +---------------------------+
```

## System Components

### 1. Advisory Workflow Engine (`src/workflow/`)
- **Deprecation:** `state.json` `current_stage` authoritative cursor is deprecated.
- **Model:** `WorkflowObserver` reads repository artifacts directly:
  - Active git branch (`feat/*`, `fix/*`, `main`).
  - Plan files located under `<docs_root>/plans/`.
  - Staged / unstaged git changes and commit history.
  - Open PR state via `gh pr view --json number,url,state`.
  - Run receipts emitted by skills (`mode:return-to-caller`).
- **Graph FSM:** Replaces the rigid linear 7-stage cycle with a flexible workflow graph:
  - Exploratory Path: Brainstorm -> Plan -> Work -> Review -> Ship.
  - Tactical / Bugfix Path: Problem -> Debug/Work -> Verify -> Ship.
  - Compound / Doc Path: Refresh / Compound -> PR.
- **Advisory Output:** `ce-ai workflow status` and `resume` provide non-blocking recommendations without preventing git operations.

### 2. Fleet Version Governance Engine (`src/fleet/` / `src/harness/`)
- Replaces raw file hashing and copying (`src/source/archive.rs`, `src/state/diff.rs`).
- Operates through native host CLI interfaces:
  - Claude Code: `claude plugin install / update`
  - OpenCode: OpenCode extension manifest / marketplace commands
  - Pi: `pi install / update`
  - Codex / Cursor / Copilot: Native extension/skill registration
- State management records target pinned version per harness:
  ```json
  {
    "fleet": {
      "target_ce_version": "1.32.0",
      "harnesses": {
        "claude": { "installed_version": "1.32.0", "status": "aligned" },
        "opencode": { "installed_version": "1.31.0", "status": "outdated" }
      }
    }
  }
  ```
- Command `ce-ai fleet sync` triggers native package manager updates to achieve target version parity.

### 3. Upstream CE Compatibility Layer (`src/compat/`)
An explicit, typed Rust domain module that wraps upstream contracts:

```rust
pub struct CeRelease {
    pub version: semver::Version,
    pub capabilities: CeCapabilities,
}

pub struct CeDocsConfig {
    pub docs_root: std::path::PathBuf,
    pub plans_dir: std::path::PathBuf,
    pub solutions_dir: std::path::PathBuf,
}

pub enum ProblemType {
    Bugfix,
    Architecture,
    Pattern,
    Discovery,
    Config,
    Preference,
}

pub struct CeSolutionFrontmatter {
    pub schema_version: Option<String>,
    pub module: String,
    pub date: chrono::NaiveDate,
    pub problem_type: ProblemType,
    pub component: String,
    pub related_components: Vec<String>,
    pub severity: String,
    pub tags: Vec<String>,
    pub applies_when: Option<String>,
}
```

### 4. Doc Hygiene Integration
- Discard custom frontmatter validation logic that conflicts with CE's `schema.yaml`.
- Delegate doc audits to `compound audit` CLI or upstream-compatible Python scripts when present.
- Support dynamic `docs_root` loaded from `.compound-engineering/config.yaml` with fallback to `docs/`.
- Recognize `docs/plans/` requirements-only plans as modern brainstorms.
