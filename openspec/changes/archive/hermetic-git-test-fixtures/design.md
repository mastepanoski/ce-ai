# Design: Hermetic Git Environment in Test Fixtures

## 1. Helper Function Contract

In `tests/cli.rs`:

```rust
/// Returns a standard `std::process::Command` targeting the `git` binary,
/// with all outer Git plumbing environment variables removed (`GIT_DIR`,
/// `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_PREFIX`).
///
/// This guarantees that tests creating temporary git repositories inside
/// fixture folders behave hermetically even when executed from inside a
/// git hook (e.g., `.githooks/pre-commit`).
fn git_cmd() -> std::process::Command {
    let mut cmd = std::process::Command::new("git");
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        cmd.env_remove(var);
    }
    cmd
}
```

## 2. Refactoring Existing Invocations

All raw instantiations of `std::process::Command::new("git")` in `tests/cli.rs` will be replaced with `git_cmd()`:
- `isolated_git` lambdas around lines 1213-1221, 4313-4325 can be simplified to call `git_cmd()`.
- Unprotected `std::process::Command::new("git")` calls in lines 4992, 5063, 5125, 5203, 5318, 5394, 5587, 5592, 5694, 5736, 5825, 5897 will use `git_cmd()`.

## 3. Hermetic Guarantee
Any future test using `git_cmd()` automatically inherits the stripped environment, preventing regression.
