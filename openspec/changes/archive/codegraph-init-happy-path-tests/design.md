# Design: CodeGraph Subprocess Execution Happy Path Coverage

## 1. Helper Function `fake_codegraph` in `tests/cli.rs`
```rust
#[cfg(unix)]
fn fake_codegraph(parent: &Path) -> PathBuf {
    let bin = parent.join("fake_codegraph_bin");
    fs::create_dir_all(&bin).unwrap();
    let script = r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "codegraph v1.4.1"
    exit 0
fi
if [ "$1" = "init" ]; then
    target="${2:-.}"
    mkdir -p "$target/.codegraph"
    echo "✓ CodeGraph initialized"
    exit 0
fi
exit 0
"#;
    let path = bin.join("codegraph");
    fs::write(&path, script).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}
```

## 2. Test Cases to Add

### Test 1: `tools_init_codegraph_happy_path_creates_index`
- Create isolated temporary directory for project, config, and fake binary.
- Prepend `fake_codegraph(&tmp)` to `PATH`.
- Run `ceai` command: `tools init codegraph`.
- Assert command succeeds (exit code 0).
- Assert stdout contains `✓ Initialized CodeGraph index`.
- Assert `project_dir.join(".codegraph").exists()` is `true`.

### Test 2: `init_prj_auto_initializes_codegraph_when_present`
- Create isolated temporary directory for project and fake binary.
- Initialize clean git repository in `project_dir`.
- Prepend `fake_codegraph(&tmp)` to `PATH`.
- Run `ceai` command: `init-prj`.
- Assert command succeeds (exit code 0).
- Assert stdout contains `✓ Initialized CodeGraph index (.codegraph/)`.
- Assert `project_dir.join(".codegraph").exists()` is `true`.
- Verify second run is idempotent and does not recreate or error.
