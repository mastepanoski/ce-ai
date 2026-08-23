# Design: Grok Adapter Audit Refinements

## 1. Static Mutex Environment Guard for Unit Tests (`src/harness/grok.rs`)

```rust
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn grok_adapter_default_paths() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("GROK_HOME");
    let adapter = GrokAdapter;
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/tmp/home/.grok/config.toml")
    );
}

#[test]
fn grok_adapter_respects_grok_home_env() {
    let _guard = ENV_LOCK.lock().unwrap();
    let adapter = GrokAdapter;
    let home = PathBuf::from("/tmp/home");
    std::env::set_var("GROK_HOME", "/custom/grok/dir");
    let path = adapter.default_config_path(&home);
    std::env::remove_var("GROK_HOME");
    assert_eq!(path, PathBuf::from("/custom/grok/dir/config.toml"));
}
```

## 2. Legacy Code Removal (`src/harness/generic_json.rs`)
Remove `HarnessKind::Grok => home.join(".grok").join("config.json"),` from `generic_json.rs`, update the module header docstring (removing Grok), and update generic JSON unit tests.
