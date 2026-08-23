---
title: "Transactional Error Propagation & State Commit Integrity"
category: "bugfixes"
module: "src/commands/"
tags: ["transactional", "error-handling", "bugfix", "state-commit"]
problem_type: "bug"
severity: "P1"
---

# Transactional Error Propagation & State Commit Integrity

## Problem
In CLI commands performing destructive or cleanup operations (`uninstall`, `deinit-prj`, `init-prj`), errors during file removal, backup restoration, atomic writes, or `.gitignore` block cleanups were previously suppressed using `let _ =`. This caused two severe bugs:
1. **False Positives**: The CLI printed success indicators (`"✅ Uninstalled cleanly"`, `"✓ Removed project adoption block"`) even when underlying filesystem mutations failed due to IO or permission errors.
2. **State Corruption**: `state.json` was saved *before* or *despite* failed filesystem mutations, falsely recording project or harness removal on disk.

## Solution
1. **Required vs Best-Effort Error Policy**:
   - **Required operations** (user-facing file deletions, backup restorations, atomic writes, `.gitignore` sentinel block updates): Errors MUST propagate immediately via `?`, returning a `CeError` and exiting with non-zero status.
   - **Best-effort operations** (secondary registry updates): Failures emit a `warning:` to `stderr` (unless `--quiet` is specified) without aborting the core command.
2. **Transactional State Commit Ordering**:
   - `state.save(&state_path)` is strictly delayed until ALL required filesystem operations have executed and succeeded. If any filesystem operation fails, `state.json` remains untouched on disk for easy retries.
3. **Failure-Injection Test Suite**:
   - Added CLI integration tests in `tests/cli.rs` (`uninstall_failure_propagates_error_and_preserves_state`, `uninstall_invalid_harness_name_returns_usage_error`) verifying non-zero exit codes and state preservation upon failure.

## Key Pattern
```rust
// 1. Execute required filesystem mutations with `?` propagation
for target in &targets {
    if let Ok(harness_kind) = target.parse::<HarnessKind>() {
        let config_dir = harness_kind.harness_dir(&home_dir);
        let target_config = harness_kind.config_path(&config_dir);
        if let Some(backup) = newest_backup_for_harness(&backups, target)? {
            restore_backup_by_id(&backups, &backup.id, &target_config)?;
        } else if target_config.exists() {
            std::fs::remove_file(&target_config)?;
        }
    }
}

// 2. Log warnings for optional best-effort cleanups
if let Err(e) = SkillRegistry::remove(ctx) {
    if !ctx.quiet {
        eprintln!("warning: skill registry cleanup failed: {e}");
    }
}

// 3. Commit state ONLY after required filesystem operations succeed
state.save(&state_path)?;
```
