# Design: Cargo Build Profiles & Maintenance Rule

## Profile Configuration (`Cargo.toml`)

```toml
[profile.dev]
debug = "line-tables-only"
incremental = false

[profile.test]
debug = "line-tables-only"
incremental = false
```

- `debug = "line-tables-only"`: Instructs `rustc` to emit `-C debuginfo=1` (line tables only).
- `incremental = false`: Prevents `rustc` from maintaining incremental compilation caches in `target/debug/incremental/`.

## Makefile Integration (`Makefile`)

```makefile
# Prunes build artifacts not accessed in the last 14 days
sweep:
	@command -v cargo-sweep >/dev/null 2>&1 || { echo "installing cargo-sweep (one-time)..."; cargo install --locked cargo-sweep; }
	cargo sweep --time 14
```
