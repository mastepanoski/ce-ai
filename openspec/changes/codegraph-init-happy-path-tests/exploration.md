# Exploration: CodeGraph Subprocess Execution Happy Path Coverage

## 1. Existing Mock Patterns in Codebase
In `tests/cli.rs:4245-4270`, the test suite previously solved a similar problem for `gh` via `fake_gh`:
```rust
#[cfg(unix)]
fn fake_gh(parent: &Path, behavior: &str) -> PathBuf {
    let bin = parent.join("fake_bin");
    fs::create_dir_all(&bin).unwrap();
    let script = match behavior {
        "protected" => "#!/bin/sh\necho '{\"required_status_checks\":{\"contexts\":[\"ci\"]}}'\n",
        ...
    };
    let path = bin.join("gh");
    fs::write(&path, script).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}
```
This pattern allows injecting a directory into `PATH` before invoking `ceai(config_dir, home)`:
```rust
let path_var = std::env::var("PATH").unwrap_or_default();
cmd.env("PATH", format!("{}:{path_var}", fake_bin.display()));
```

## 2. Requirements for `fake_codegraph`
When `ce-ai` probes CodeGraph, it calls:
1. `codegraph --version`:
   - `init_codegraph` checks `std::process::Command::new("codegraph").arg("--version").output()`.
   - `init_codegraph_if_available` checks whether `status.success()` is true.
2. `codegraph init <path>`:
   - `init_codegraph` runs `std::process::Command::new("codegraph").arg("init").arg(&target_path).status()`.
   - `init_codegraph_if_available` runs `std::process::Command::new("codegraph").arg("init").arg(target_dir).status()`.

Therefore, the mock script only needs to:
- If `$1` == `--version`: print `codegraph v1.4.1` and exit `0`.
- If `$1` == `init`: parse `$2` (target directory, defaulting to `.`), create `$target/.codegraph`, print `✓ CodeGraph initialized`, and exit `0`.

## 3. Evaluated Options
- **Option A: Pure Rust Mock Binary built in target/**: Heavyweight, slows down test compilation.
- **Option B: Hermetic Shell Script with `#[cfg(unix)]`**: Fast, lightweight, identical to `fake_gh`, reliable across Linux and macOS runners in CI.
- **Decision**: Option B matches repository conventions and provides clean test isolation.
