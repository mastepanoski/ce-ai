# Proposal: Guard Companion MCP Registration Against Directory Config Paths

## Problem Statement

When running `ce-ai install --harness all` (or `ce-ai sync`), `ce-ai` iterates through all detected harnesses and configures companion MCP servers (`codegraph`, `engram`) via `RegistrationSpec::register_companions`.

While the config backup step in `src/commands/install.rs:168` checks `target_config.is_file()` (specifically documented to prevent `std::fs::read` from failing when a harness config path resolves to a directory, such as Pi's `~/.pi/agent/skills`), the MCP registration step in `RegistrationSpec::register_companions` (`src/harness/registration.rs:29`) lacks this check.

Consequently, native MCP registrars perform `if config_path.exists() { let content = std::fs::read_to_string(config_path)?; ... }`. When `config_path` is an existing directory:
1. `std::fs::read_to_string` fails with `std::io::Error: Is a directory (os error 21)`.
2. `CeError::Io` formats this via passthrough `Display` as `error: I/O error: Is a directory (os error 21)` with zero harness or path context.
3. The error aborts the entire `--harness all` loop mid-execution, before `state.save()` and `journal.complete()`.
4. Previous successfully installed harnesses are unrecorded in `state.json` and rolled back by `Journal::begin` on the subsequent run.

## Scope Boundaries

### In Scope
- Add a directory check (`target_config.exists() && !target_config.is_file()`) in `RegistrationSpec::register_companions` (`src/harness/registration.rs`).
- Emit an explicit non-fatal warning containing both the harness name and the config path:
  `warn: skipping companion MCP registration for <harness>: '<path>' exists but is not a regular config file (expected a JSON/TOML file)`
- Return `Ok(())` so the `--harness all` / `sync` loop continues cleanly for remaining harnesses.
- Enrich any unexpected `std::io::Error` returned by MCP registrars with harness kind and file path context.
- Associate `HarnessKind` with `RegistrationSpec` so the registrar is self-describing.
- Add unit tests in `src/harness/tests/registration.rs` verifying directory skipping and error wrapping.
- Add an integration test in `tests/cli.rs` verifying that `ce-ai install --harness all` does not abort when a harness config path is an existing directory.

### Out of Scope
- Per-harness transactional committing of `state.json` and journal (issue #314 suggested fix 3 — architectural redesign).
- Refactoring `register_custom_mcp_server` (`custom.rs:322`) where config path is explicitly user-supplied via `--mcp-file`.
- Modifying individual native adapter registration functions (`claude.rs`, `cursor.rs`, etc.).

## Risk Evaluation
- **Low Risk**: The check is placed in the single shared entry point `RegistrationSpec::register_companions`. If a config path does not exist, `exists()` is false and normal creation proceeds. If it is a regular file, `is_file()` is true and normal registration proceeds. Only existing non-regular files (directories) trigger the warning and non-fatal skip.

## Success Criteria
1. `RegistrationSpec::register_companions` skips directory config paths with an explicit warning naming harness and path, returning `Ok(())`.
2. I/O errors occurring during registration carry harness and path context.
3. `ce-ai install --harness all` does not fail when an individual harness config path is a directory; previous and subsequent harnesses are preserved and journal completes.
4. All unit, CLI integration, and containerized E2E tests pass 100%.
