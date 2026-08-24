# Exploration: Google Antigravity (AGY) Adapter Audit Refinements

## Audit Analysis

### 1. Relocation Environment Variables (`ANTIGRAVITY_CONFIG_DIR`, `GEMINI_HOME`)
- **Status**: Official Google Antigravity documentation relies on default path `~/.gemini/`.
- **ce-ai Extension**: `ce-ai` supports `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` environment overrides for custom test setups and isolated harness environments.
- **Action**: Document as explicit `ce-ai` extensions in `design.md` and `spec.md`.

### 2. Project Rules Architecture
- **Canonical Instruction**: `GEMINI.md` at repository root.
- **Derived Stub**: `.agents/rules/compound-engineering.md` when `.agents/` pre-exists.
- **Action**: Document dual instruction resolution strategy in `design.md`.

### 3. Server Registration Name Collision Policy
- **Behavior**: When registering managed tools (`codegraph`, `engram`), if `mcp_config.json` already contains an entry with `serverUrl`, setting `server_url = None` converts the entry to a stdio command server (`command`, `args`, `env`).
- **Action**: Verify with unit test assertion in `src/harness/agy.rs`.

### 4. HarnessAdapter Trait Evolution
- **Behavior**: `canonical_instruction_file()` and `derived_stub_files()` provide clean polymorphism for instruction file targets.
- **Action**: Maintain trait integrity.
