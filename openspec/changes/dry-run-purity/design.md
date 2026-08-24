# Design: Global `--dry-run` Purity & Zero-Mutation Contract

## Architecture

### 1. Workflow Checkpoint Dry-Run Guard
In `src/commands/workflow.rs`:
```rust
pub fn checkpoint_lines(
    ctx: &Context,
    stage: WorkflowStage,
    task: &str,
    feature: Option<&str>,
) -> Result<Vec<String>, CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;

    // Transition validation
    state.validate_and_set_workflow(stage, task, feature)?;

    if !ctx.dry_run {
        state.save(&state_path)?;
    }

    Ok(vec![
        "workflow: checkpoint saved successfully!".to_string(),
        format!("  stage: {} ({})", stage.number(), stage.as_str()),
        format!("  task: {task}"),
    ])
}
```

### 2. Remote Download Dry-Run Isolation
In `src/commands/install.rs` and `src/commands/upgrade.rs`:
When `--dry-run` is active, remote source tarball extraction resolves to a transient temporary directory (`tempfile::TempDir`) rather than writing to `ctx.config_dir.join("cache")` or updating `release_provenance` in `state.json`.

### 3. Snapshot Assertion Helper Across All Directories
In `tests/cli.rs`:
```rust
fn dir_snapshot(dir: &Path) -> BTreeMap<PathBuf, String> {
    // Collect relative path -> sha256 hash for all files
}

fn assert_dry_run_zero_mutation(
    cmd: &mut Command,
    config_dir: &Path,
    home_dir: &Path,
    workspace_dir: &Path,
) {
    let before_config = dir_snapshot(config_dir);
    let before_home = dir_snapshot(home_dir);
    let before_workspace = dir_snapshot(workspace_dir);

    cmd.assert().success();

    let after_config = dir_snapshot(config_dir);
    let after_home = dir_snapshot(home_dir);
    let after_workspace = dir_snapshot(workspace_dir);

    assert_eq!(before_config, after_config, "config_dir mutated during dry-run!");
    assert_eq!(before_home, after_home, "home_dir mutated during dry-run!");
    assert_eq!(before_workspace, after_workspace, "workspace_dir mutated during dry-run!");
}
```
