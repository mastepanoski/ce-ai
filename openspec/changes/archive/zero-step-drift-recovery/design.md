# Design: Zero-Step Environment Drift Recovery via Live `RepoState` Sync

## 1. System Architecture & Component Interaction

```
[ce-ai workflow resume / status]
              │
              ├── 1. Load state.json & current WorkflowState
              │
              ├── 2. probe_repo_state(ctx, &workflow)
              │        │
              │        ├─► Git Context Probe:
              │        │     ├── `git rev-parse --abbrev-ref HEAD` (Active Branch)
              │        │     ├── `git rev-parse --short HEAD` (HEAD SHA)
              │        │     └── `git status --porcelain=v1` (Dirty file list)
              │        │
              │        ├─► Manifest Drift Probe:
              │        │     └── diff::diff(&desired, &desired, &managed_dir) -> manifest_drift_count
              │        │           where desired: BTreeMap<String, String> from InstallManifest::load()
              │        │
              │        ├─► Project Adoption Block Probe:
              │        │     └── check_adoption_block_status(&agents_path, tier) -> AdoptionBlockStatus
              │        │           (using SSOT helper from src/commands/init_prj.rs:34)
              │        │
              │        └─► OpenSpec Progress Probe:
              │              └── probe_openspec_context(&workflow) -> OpenSpecContextInfo
              │
              ├── 3. Format Human-Readable Output:
              │        ├── Status line + Workflow stage
              │        ├── == [Environment State & Drift Status] ==
              │        └── == [Context Re-hydration: <feature>] ==
              │
              └── 4. Format JSON Output (`--json`):
                       {
                         "workflow": { ... },
                         "repo_state": { ... },
                         "openspec_context": { ... }
                       }
```

## 2. Data Structures & Schema Design

### `RepoState` Struct (`src/commands/workflow.rs`)
```rust
use crate::commands::init_prj::AdoptionBlockStatus;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RepoState {
    /// Active git branch name (e.g. `main`, `feat/auth`), or `None` if outside git.
    pub git_branch: Option<String>,
    /// Short SHA of HEAD commit (e.g. `7a8b9c0`), or `None`.
    pub head_sha: Option<String>,
    /// True if working tree has 0 uncommitted/untracked changes.
    pub is_git_clean: bool,
    /// List of relative paths of modified, deleted, or untracked files in working tree.
    pub modified_files: Vec<String>,
    /// Number of drifted/missing files in managed plugin tree against InstallManifest.
    pub manifest_drift_count: usize,
    /// Classified adoption block status from the SSOT in init_prj.rs (Ok, DriftDetected, etc.).
    pub adoption_status: Option<AdoptionBlockStatus>,
    /// OpenSpec progress details if an active feature spec exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openspec_context: Option<OpenSpecContextInfo>,
}
```

### Probing Engine Function
```rust
pub fn probe_repo_state(
    ctx: &Context,
    wf: &Option<WorkflowState>,
) -> RepoState {
    let git_branch = probe_git_branch(&ctx.repo_root);
    let head_sha = probe_git_head_sha(&ctx.repo_root);
    let (is_git_clean, modified_files) = probe_git_dirty_files(&ctx.repo_root);
    let manifest_drift_count = probe_manifest_drift_count(ctx);
    let adoption_status = probe_adoption_status(ctx);
    let openspec_context = probe_openspec_context(wf);

    RepoState {
        git_branch,
        head_sha,
        is_git_clean,
        modified_files,
        manifest_drift_count,
        adoption_status,
        openspec_context,
    }
}
```

## 3. Fast-Path Heuristic vs. Cryptographic Truth Specification

To ensure execution completes under 15ms while preserving 100% cryptographic determinism:

1. **Git Operations:**
   - Command: `git status --porcelain=v1` and `git rev-parse --abbrev-ref HEAD` are spawned with strict execution timeouts.
   - If git commands fail (e.g., in a non-git directory or CI sandbox without git binary), `git_branch` and `head_sha` are set to `None`, and `is_git_clean` defaults to `true`.

2. **Managed Manifest Drift Calculation:**
   - Desired file mapping is extracted from the manifest:
     ```rust
     let manifest = InstallManifest::load(&ctx.opencode_config_dir);
     let desired: BTreeMap<String, String> = manifest
         .map(|m| m.files.into_iter().map(|f| (f.path, f.sha256)).collect())
         .unwrap_or_default();
     let managed_dir = ctx.opencode_config_dir.join(MANAGED_DIR);
     let diff_result = crate::state::diff::diff(&desired, &desired, &managed_dir);
     let manifest_drift_count = diff_result.actions.len();
     ```
   - **Determinism Rule:** A file's drift status is determined exclusively by comparing its SHA256 digest with the manifest SHA256. Timestamp (`mtime`) checks are used only for early-exit file system change notifications, never as a replacement for SHA256.

3. **`AGENTS.md` Adoption Block Integrity:**
   - Rather than duplicating block hashing, `probe_adoption_status` queries `check_adoption_block_status(&agents_path, tier)` in `src/commands/init_prj.rs:34`.
   - This reuses the single source of truth (SSOT) shared by `doctor.rs` and `status.rs`, returning the exact `AdoptionBlockStatus` enum (`Ok`, `StaleVersion`, `DriftDetected`, `MalformedBlock`, `BlockMissing`, `FileMissing`, `ReadError`).

## 4. CLI Interface & Rendering

### Human-Readable Format (`ce-ai workflow resume`)
```
workflow: resuming execution from latest checkpoint...
== [Workflow FSM & Progress Recovery Status] ==
  current phase: Stage 4: Work/TDD (work)
  active subtask: Implementing RepoState probe
  active feature: zero-step-drift-recovery
  last updated: 2026-09-02T03:15:00Z

== [Environment State & Drift Status] ==
  git branch: feat/drift-recovery (HEAD: a1b2c3d)
  working tree: 2 modified files (src/commands/workflow.rs, src/state/state.rs)
  manifest integrity: clean (0 drifted files)
  adoption block: ok (v3 full)

== [Context Re-hydration: zero-step-drift-recovery] ==
  spec location: openspec/changes/zero-step-drift-recovery
  has proposal: true
  has spec: true
  has tasks: true
  tasks progress: 2/4 completed ([x])

workflow: re-hydrated context successfully. Proceeding with active task.
```

If manifest drift is present:
```
  manifest integrity: ! 2 files modified outside ce-ai
  ! Warning: Drift detected in managed files. Run 'ce-ai sync' to reconcile.
```

### JSON Format (`ce-ai workflow resume --json`)
```json
{
  "workflow": {
    "stage": "worktdd",
    "task": "Implementing RepoState probe",
    "feature_name": "zero-step-drift-recovery",
    "updated_at": "2026-09-02T03:15:00Z"
  },
  "repo_state": {
    "git_branch": "feat/drift-recovery",
    "head_sha": "a1b2c3d",
    "is_git_clean": false,
    "modified_files": [
      "src/commands/workflow.rs",
      "src/state/state.rs"
    ],
    "manifest_drift_count": 0,
    "adoption_status": "ok",
    "openspec_context": {
      "feature": "zero-step-drift-recovery",
      "path": "openspec/changes/zero-step-drift-recovery",
      "has_proposal": true,
      "has_spec": true,
      "has_tasks": true,
      "completed_tasks": 2,
      "total_tasks": 4
    }
  },
  "openspec_context": {
    "feature": "zero-step-drift-recovery",
    "path": "openspec/changes/zero-step-drift-recovery",
    "has_proposal": true,
    "has_spec": true,
    "has_tasks": true,
    "completed_tasks": 2,
    "total_tasks": 4
  }
}
```
