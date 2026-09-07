# Tasks: Companion MCP Registration Directory Guard & Contextual Error Wrapping

- [ ] 1. Update `RegistrationSpec` struct and implement directory guard in `src/harness/registration.rs` (~35 LOC)
  - Add `kind: HarnessKind` field to `RegistrationSpec`.
  - Update `registration_spec(kind: HarnessKind)` to initialize `kind`.
  - In `RegistrationSpec::register_companions`, check `target_config.exists() && !target_config.is_file()`, emit warning to stderr, and return `Ok(())`.
  - Wrap any `CeError::Io` from `register(...)` with harness kind and target config path.
- [ ] 2. Unit tests in `src/harness/tests/registration.rs` (~45 LOC)
  - Update existing tests for new `kind` field on `RegistrationSpec`.
  - Test `register_companions` skips directory config path, prints warning to stderr, and returns `Ok(())` without calling registrar.
  - Test `register_companions` wraps `CeError::Io` with harness and path context.
- [ ] 3. CLI integration test in `tests/cli.rs` (~65 LOC)
  - Test `ce-ai install --harness all` (or native harness with directory config) continues past directory config path, outputs warning, commits state/manifest, and succeeds with exit 0.
- [ ] 4. Documentation and Solution architecture capture (~60 LOC)
  - Create solution architecture doc `docs/solutions/architecture/install-directory-config-mcp-guard.md`.
- [ ] 5. SemVer bump and CHANGELOG update (~15 LOC)
  - Bump patch version to `1.44.1` in `Cargo.toml` and `Cargo.lock`.
  - Update `CHANGELOG.md` under `[1.44.1]` with bugfix details.

Total estimated LOC: ~220 lines.
