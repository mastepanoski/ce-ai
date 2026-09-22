# Exploration: Decision Engine Documentation & CLI Ergonomics

## Technical Investigation

### Current Implementation State

The Decision Engine is implemented in `src/decisions/` and wired to CLI commands via `src/commands/decisions.rs`.

1. **`DecisionMode` enum (`src/decisions/types.rs`)**:
   ```rust
   pub enum DecisionMode {
       Off,     // Zero evaluations, zero network calls.
       Shadow,  // Evaluates in background, logs telemetry, does not enforce.
       Active,  // Probabilistic decisions actively enforce policies.
   }
   ```
   It provides `as_str()` returning `"off"`, `"shadow"`, `"active"` and `parse()` accepting `"off" | "disabled"`, `"shadow"`, `"active" | "on" | "enabled"`.

2. **Presets in `handle_setup` (`src/commands/decisions.rs:360-465`)**:
   Matches `clean.as_str()` against:
   - `"recommended"` -> `provider: "jev"`, `mode: DecisionMode::Active`, budget $5, all modules enabled.
   - `"shadow"` -> `provider: "jev"`, `mode: DecisionMode::Shadow`, budget $5, all modules enabled.
   - `"local"` -> `provider: "mock"`, `mode: DecisionMode::Active`, mock models, all modules enabled.
   - `_` -> `CeError::Usage("invalid preset '...'. Valid presets: recommended, shadow, local")`.

3. **Gaps in CLI Actions**:
   `Action` enum in `src/commands/decisions.rs` currently includes:
   `Status`, `Auth`, `Setup`, `Test`, `CheckRisk`, `CheckReadiness`, `Route`.
   Noticeable omissions:
   - No way to disable the engine from CLI (no `setup --preset off` or `mode off`).
   - No way to toggle or inspect runtime mode without re-running `setup` or editing `state.json`.

4. **Documentation Audit**:
   - `docs/user-guide/`: 18 guides present, zero covering `decisions`.
   - `README.md`: 99 lines. Contains command tables and documentation map. Decision engine is absent from both tables.

## Evaluated Options

### Option 1: Documentation Only
- Document the existing CLI as-is, noting that `state.json` must be manually edited to switch modes or disable the engine.
- **Pros**: Zero code risk.
- **Cons**: Leaves the ergonomic inconsistency in place; developer experience friction when trying to disable or switch to shadow mode.

### Option 2: Add Preset `off` + `ce-ai decisions mode` Subcommand + Comprehensive User Guide (Recommended)
- Enhance `handle_setup` to accept `"off" | "disabled"`, cleanly disabling the engine in `state.json`.
- Add `Action::Mode { mode: Option<String> }` to `ce-ai decisions mode`:
  - `ce-ai decisions mode` -> reads and displays current mode.
  - `ce-ai decisions mode <active|shadow|off>` -> parses via `DecisionMode::parse`, updates `state.json` atomically.
  - If enabling to `active` or `shadow` when `state.decisions` is None, provisions default configuration.
- Write `docs/user-guide/decision-engine-guide.md` (Diátaxis How-to / Reference).
- Update `README.md` keeping `<= 100` lines.
- **Pros**: Completely resolves user confusion, unifies terminology, adds zero breaking changes, and provides definitive documentation.
- **Cons**: Adds ~60 lines of Rust code and tests (well within work-unit budget).

## Architectural Tradeoffs

- **State Mutation Safety**: All updates must continue to use `crate::state::write_atomic` to prevent file corruption.
- **Backwards Compatibility**: Existing presets (`recommended`, `shadow`, `local`) behave identically. Adding `off` and `mode` is purely additive.
- **README Constraint**: `README.md` must not exceed 100 lines. We can condense existing table rows slightly to accommodate the new reference row.
