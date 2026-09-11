# Exploration: Hermetic Git Environment in Test Fixtures

## 1. Technical Investigation
When Git invokes a hook script (such as `.githooks/pre-commit`), Git exports several plumbing environment variables into the subshell:
- `GIT_DIR`: Absolute or relative path to the `.git` directory of the host repository.
- `GIT_INDEX_FILE`: Path to the index file (e.g. `.git/index`).
- `GIT_PREFIX`: The relative directory where the command was executed.
- `GIT_WORK_TREE`: The root of the working tree.

In `tests/cli.rs`, `ceai()` already acknowledged this issue:
```rust
// Hermetic git resolution: under the pre-commit hook GIT_DIR points at the
// real checkout, which would make doctor's repo probes leave the fixture.
for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
    cmd.env_remove(var);
}
```
However, test setup functions scattered across `tests/cli.rs` repeatedly called:
```rust
std::process::Command::new("git").args(["init", ...]).output().unwrap();
```
Because these commands did not remove the `GIT_*` environment variables, `git init` was evaluated with `GIT_DIR` pointing to the host checkout.

## 2. Reproduction and Verification
By simulating the hook environment:
```bash
GIT_DIR="/Users/mastepanoski/projects/web/ai/ce-ai/.git" \
GIT_INDEX_FILE="/Users/mastepanoski/projects/web/ai/ce-ai/.git/index" \
GIT_PREFIX="" \
cargo test --test cli
```
The exact 3 tests failed deterministically:
1. `audit_suggests_codegraph_init_without_gentle_ai`: `git rev-parse --show-toplevel` failed because the test directory was not a git repo, omitting `code-intelligence` checks from the audit.
2. `doctor_workspace_scope_opencode_install_has_no_false_positive_findings`: `install --scope workspace` fell back to `ctx.opencode_config_dir` (global), causing `state-inconsistent`.
3. `install_workspace_scope_ensures_compound_engineering_in_gitignore`: `install --scope workspace` fell back to global config, failing to write `.gitignore` in the test workspace.

## 3. Evaluated Options

### Option A: Ad-hoc `env_remove` in failing tests
- **Drawbacks**: Fragile. Future tests written with `Command::new("git")` will reintroduce the flake.

### Option B: Centralized `git_cmd()` helper in `tests/cli.rs` (Recommended)
- **Mechanism**:
  Define:
  ```rust
  fn git_cmd() -> std::process::Command {
      let mut cmd = std::process::Command::new("git");
      for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
          cmd.env_remove(var);
      }
      cmd
  }
  ```
  Replace all `std::process::Command::new("git")` across `tests/cli.rs` with `git_cmd()`.
- **Benefits**:
  - Completely hermetic test execution under git hooks, CI runners, worktrees, and shell sessions.
  - DRY and consistent across the entire test suite.
