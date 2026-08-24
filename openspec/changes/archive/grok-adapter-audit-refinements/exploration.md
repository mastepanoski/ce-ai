# Exploration: Grok Adapter Audit Refinements

## 1. Test Environment Race Condition
In `src/harness/grok.rs`, unit tests `grok_adapter_default_paths` and `grok_adapter_respects_grok_home_env` run in parallel threads during `cargo test`. Mutating process-global `GROK_HOME` via `std::env::set_var` without thread synchronization causes `grok_adapter_default_paths` to occasionally read the mutated `GROK_HOME` value and fail. Protecting environment variable modifications with a static `Mutex` lock ensures test thread safety.

## 2. Legacy Generic JSON Removal
`src/harness/generic_json.rs` contained legacy mapping for `HarnessKind::Grok` pointing to `.grok/config.json`. Removing this dead entry aligns Grok with Claude, Codex, Cursor, and Copilot native adapters.
