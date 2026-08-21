# Technical Execution Plan: Release v0.7.0 — Workspace Overrides & Multi-Harness Uninstall Parity

**Title**: Release v0.7.0 — Workspace Overrides & Multi-Harness Uninstall Parity  
**Date**: 2026-08-21  
**Origin Document**: [`openspec/changes/v0_7_0_roadmap/spec.md`](openspec/changes/v0_7_0_roadmap/spec.md)  
**Related Issue**: [#64](https://github.com/mastepanoski/ce-ai/issues/64)  
**Target Release**: `v0.7.0`  

---

## 1. Problem Frame & Context

`ce-ai` v0.6.0 introduced proactive workflow observability, TUI FSM dashboards, extended health diagnostics, and real-time sync watchers. However, two architectural parity gaps remain:

1. **Lack of Repository-Local Overrides (`.ce-ai.json`)**:
   - Model assignments (`ce-work`, `ce-plan`, `ce-brainstorm`) and active profiles are currently strictly global (`~/.config/ce-ai/state.json`).
   - Teams working on distinct repositories cannot specify repo-local LLM overrides or project configurations without mutating global developer state.
2. **Incomplete Multi-Harness Uninstall Parity (Issue #64)**:
   - `ce-ai uninstall` currently only restores the latest `opencode.json` backup for OpenCode.
   - Users who want to completely remove managed plugin loaders (`compound-engineering.js`), skills directories (`.opencode/skills/`), and SHA256 manifests across all installed harnesses (`--harness all --all`) have no supported command path.

---

## 2. Architectural Design & Layer Precedence

### Configuration Hierarchy Precedence
When resolving configuration state, `ce-ai` will enforce the following precedence chain (highest to lowest):
1. **CLI Flags & Command Arguments** (e.g. `--harness claude`)
2. **Workspace Overrides (`.ce-ai.json`)** (located in `git_root()` or current working directory)
3. **Global User Configuration (`~/.config/ce-ai/state.json`)**
4. **Hardcoded Engine Defaults**

```
                     ┌──────────────────────────────────────┐
                     │   CLI Arguments / Commands           │ (Highest Priority)
                     └──────────────────┬───────────────────┘
                                        │
                                        ▼
                     ┌──────────────────────────────────────┐
                     │  Workspace Overrides (.ce-ai.json)  │ (Key-Level Merging)
                     └──────────────────┬───────────────────┘
                                        │
                                        ▼
                     ┌──────────────────────────────────────┐
                     │  Global State (~/.config/ce-ai/...)  │ (Baseline State)
                     └──────────────────────────────────────┘
```

---

## 3. Implementation Units & Codebase Changes

### Unit 1: Workspace Overrides Merging Engine (`src/state/state.rs`)
- **Target File**: `src/state/state.rs`
- **Responsibilities**:
  - Implement `State::load_with_workspace_overrides(global_path: &Path, workspace_root: Option<&Path>) -> Result<Self, CeError>`.
  - Implement `merge_overrides(&mut self, local_state: State)` merging field-by-field (e.g., `model_assignments`).
  - Add unit tests verifying that local assignments override global assignments while un-specified keys inherit global defaults.

### Unit 2: Complete Multi-Harness Uninstall (`src/harness/mod.rs` & `src/commands/uninstall.rs`)
- **Target Files**: `src/harness/mod.rs`, `src/commands/uninstall.rs`, `src/main.rs`
- **Responsibilities**:
  - Extend `HarnessAdapter` trait with `fn uninstall(&self, ctx: &Context, all: bool) -> Result<(), CeError>`.
  - Extend `UninstallArgs` in `src/main.rs` with `--harness <name|all>`, `--all`, and `--yes` / `-y` flags.
  - Implement per-harness loader and skills removal in `src/commands/uninstall.rs`.
  - Enforce interactive confirmation prompt unless `--yes` / `-y` is supplied.
  - Preserve all unmanaged user configurations and custom skills.

### Unit 3: CLI Integration Test Suite (`tests/cli.rs`)
- **Target File**: `tests/cli.rs`
- **Responsibilities**:
  - Add `workspace_overrides_precedence_test` verifying `.ce-ai.json` overriding global model assignments.
  - Add `uninstall_harness_all_with_yes_flag_test` verifying complete multi-harness removal.

### Unit 4: Teacher-Style User Documentation & Sitemap (`docs/user-guide/`, `README.md`, `CHANGELOG.md`)
- **Target Files**: `docs/user-guide/harnesses-loops-and-context-masterclass.md`, `README.md`, `CHANGELOG.md`
- **Responsibilities**:
  - Add teacher-style explanations for workspace overrides (`.ce-ai.json`) using the analogy of *local cockpit settings vs global flight plan*.
  - Update `README.md` features and CLI subcommands table.
  - Update `CHANGELOG.md` for `v0.7.0`.

---

## 4. Test Scenarios & Edge Cases

| Scenario | Expected Behavior | Verification Method |
| :--- | :--- | :--- |
| **Partial `.ce-ai.json`** | Local `.ce-ai.json` defines `ce-work` slot; `ce-plan` slot is inherited from `state.json`. | Unit test in `src/state/state.rs` |
| **Missing `.ce-ai.json`** | System loads global `state.json` without error. | Unit test in `src/state/state.rs` |
| **`uninstall --harness all --all --yes`** | Removes managed loaders and skills across all 12 installed harnesses without prompting. | Integration test in `tests/cli.rs` |
| **`uninstall --all` (no `--yes`)** | Displays interactive confirmation prompt before deleting files. | Interactive test in `tests/cli.rs` |
| **Unmanaged User Plugins** | Custom user plugins in `opencode.json` or `.claude/` remain untouched during `--all` uninstall. | Safety test in `tests/cli.rs` |

---

## 5. Required 7-Stage Development Cycle & Governance

1. **Stage 1 (Ideation)**: ✅ Completed (`ce-brainstorm` completed).
2. **Stage 2 (OpenSpec)**: ✅ Completed (`openspec/changes/v0_7_0_roadmap/` created).
3. **Stage 3 (Plan & Review)**: 🚧 Current (`ce-plan` execution plan + `ce-doc-review`).
4. **Stage 4 (Work & Refactor)**: `ce-work` ➔ `ce-simplify-code` ➔ `ce-code-review`.
5. **Stage 5 (Verification)**:
   - `cargo fmt --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test`
   - `make e2e`
6. **Stage 6 (Compound & Documentation)**:
   - Teacher-style guide explanations in `docs/user-guide/`.
   - Update `README.md` & `CHANGELOG.md`.
   - `ce-doc-review` doc pass.
   - `ce-compound` solution documentation.
7. **Stage 7 (Git Delivery)**:
   - Feature branch `feature/v0-7-0-workspace-overrides-and-uninstall-parity`.
   - Commit, push, open PR, watch CI, and merge to `main`.
