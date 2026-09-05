# Exploration: Git-Hooks Probe Adoption Guard

## Problem Analysis
In `src/commands/doctor.rs:297-333`, the git-hooks probe executed the following:
```rust
if let Ok(hooks_output) = std::process::Command::new("git")
    .args(["config", "--get", "core.hooksPath"])
    .current_dir(root_path)
    .output()
{
    if hooks_output.status.success() {
        let raw_val = String::from_utf8_lossy(&hooks_output.stdout);
        let hooks_val = raw_val.trim().trim_end_matches('/').trim_end_matches('\\');
        let hooks_path = std::path::Path::new(hooks_val);
        if !hooks_val.ends_with(".githooks")
            && hooks_path.file_name() != Some(std::ffi::OsStr::new(".githooks"))
        {
            findings.push(format!(
                "git-hooks: core.hooksPath set to '{}', expected '.githooks'",
                hooks_val
            ));
        } else {
            let pre_commit = root_path.join(".githooks").join("pre-commit");
            if !pre_commit.exists() {
                findings.push("git-hooks: .githooks/pre-commit missing".into());
            }
        }
    } else {
        println!("doctor-info: git-hooks core.hooksPath not set");
    }
}
```

### Trace to Origin
As documented in `docs/solutions/architecture/context-exhaustion-resilience-and-deterministic-invariants.md`, `ce-ai` uses `.githooks` locally to prevent committing transient files or secret credentials. During multi-worktree operations, creating sibling worktrees could unbind or modify the local git configuration. A probe was added to `doctor` to alert developers if `core.hooksPath` shifted away from `.githooks`.

However, when shipped in `ce-ai` binary as part of the public `doctor` command, this probe ran against any repository where `ce-ai doctor` was run. Many modern codebases utilize other managers:
- Husky (`core.hooksPath = .husky/_`)
- lefthook (`core.hooksPath = .lefthook/...`)
- pre-commit (`core.hooksPath = ...`)

Flagging these projects as broken (exit code 1 or 2) is a false positive that breaks CI and confuses users.

## Evaluated Options

### Option 1: Completely remove the git-hooks probe from `doctor`
- **Pros**: Eliminates all false positives across external projects.
- **Cons**: Removes drift protection for projects that *have* adopted `.githooks` (including `ce-ai` itself and other projects following this pattern).

### Option 2: Conditionally enforce `.githooks` only when `.githooks/` directory exists (Selected)
- **Pros**:
  - If a project contains a `.githooks/` directory at its root, it has explicitly opted into this convention. Any deviation in `core.hooksPath` indicates actual drift (e.g. git worktree contention or accidental unsetting), which is correctly reported as a finding.
  - If a project does not contain `.githooks/` and has `core.hooksPath` set to something else (e.g. `.husky/_`), doctor recognizes this as an external hooks manager and logs an informational notice (`doctor-info`) rather than a failure finding.
- **Cons**: None identified.
