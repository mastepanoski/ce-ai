# Exploration: model-defaults-tui-orchestrator

## Investigation

- `apply_model_assignment` (src/opencode/config.rs:91) is the only writer of `agent.<slot>.model` and always emits `"variant": ""`. Its output shape was used as forensic fingerprint in issue #111.
- `ce-ai models set` (src/commands/models.rs) writes opencode.json first, then state.json, then an append-only snapshot — a failure in config leaves state untouched.
- `opencode models` CLI prints one `provider/model` token per line (verified live: 491 entries across providers on the host). This is the authoritative, account-aware catalog — it reflects exactly what the configured providers offer.
- `install.rs` merges plugin + skills path via `ensure_plugin_and_skills`, backs up prior config, records mutations in the manifest. No model logic existed there.
- `doctor.rs` collects string findings and exits non-zero via `CeError::Runtime`; heavy external probes (git/gh) make end-to-end testing impractical → drift check extracted into a pure function for unit tests.
- `sync.rs::sync_with` already reloads state before its final save; the config→state import slots in naturally there.
- TUI key handling is a single match with a modal short-circuit; a picker modal reuses this pattern.

## Options evaluated

### Default seeding location
1. **Inside `install.rs` after harness loop** (chosen): defaults belong to install; single call site.
2. Inside sync: wrong semantics — sync repairs, it should not introduce new assignments.
3. Separate `ce-ai models init-defaults` command: extra CLI surface users must know about.

### Picker catalog source
1. **Harness CLI discovery** (chosen): `opencode models` output parsed into `provider/model` tokens. Account- and provider-aware: Claude offers its own models, opencode exposes every provider variant, and future harnesses are covered by their own CLIs. Parsing logic (`parse_models_output`) is pure and unit-tested.
2. Hardcoded const catalog: rejected — instantly stale, ignores what the user's accounts/providers actually offer.
3. Provider HTTP APIs: requires credentials/network in the TUI path; overkill when the harness CLI already aggregates them.

### Drift repair direction
1. **config→state import** (chosen): opencode.json is the effective runtime truth and may contain user edits made outside ce-ai. Import never mutates user files.
2. state→config re-apply: would clobber manual user edits with possibly stale state — violates the preserve-user-configs invariant.

### Doctor finding granularity
Pure function `model_drift_findings(&State, &serde_json::Value) -> Vec<String>` evaluated over: slots present in state but missing/different in config, and CE-known slots present in config but missing from state. Arbitrary third-party agent slots without state entries are ignored to avoid false positives.

### TUI editing interaction
1. Inline text input: needs a text-input widget/state machine; heavy for the gain.
2. **Slot cursor + discovery-backed picker modal** (chosen): `n`/`p` move slot selection, `m` opens a picker populated by `discover_models()` (plus the current value if absent); Up/Down select, Enter applies via `models::set`, Esc cancels. Discovery failures surface an explicit error modal — no silent static fallback.

## Tradeoffs

- Defaults remain compile-time constants (`DEFAULT_MODEL_ASSIGNMENTS`) while the picker is dynamic: defaults are deliberate seeds documented in the spec, not a catalog of everything available.
- Discovery shells out to `opencode` per picker open: acceptable latency for an interactive action; errors are explicit.
