# Spec: Companion MCP Registration Directory Guard & Contextual Error Wrapping

## Requirements & Acceptance Criteria

### R1: Guard Directory Config Paths in `register_companions`
- **WHEN** `RegistrationSpec::register_companions` is invoked with a `target_config` path that exists and is a directory (`target_config.exists() && !target_config.is_file()`),
- **THEN** it MUST NOT call the underlying vendor MCP registrar.
- **AND** it MUST emit a diagnostic warning to stderr in the format:
  `warn: skipping companion MCP registration for <harness>: '<path>' exists but is not a regular config file (expected a JSON/TOML file)`
- **AND** it MUST return `Ok(())` so the calling lifecycle loop can continue uninterrupted.

### R2: Normal Registration for Regular Files or Non-Existent Paths
- **WHEN** `RegistrationSpec::register_companions` is invoked with a `target_config` path that is a regular file (`target_config.is_file()`) OR does not yet exist (`!target_config.exists()`),
- **THEN** it MUST proceed to call the vendor registrar to configure `codegraph` and `engram`.

### R3: Contextual Error Wrapping for I/O Failures
- **WHEN** the underlying vendor MCP registrar returns a `CeError::Io(err)`,
- **THEN** `RegistrationSpec::register_companions` MUST wrap the error in a new `std::io::Error` whose message contains the harness kind and target config path (`format!("{}: '{}': {err}", self.kind, target_config.display())`).
- **AND** the resulting error MUST remain a `CeError::Io` (exit code 4).

### R4: Loop Resilience in `install --harness all` and `sync`
- **WHEN** `ce-ai install --harness all` or `ce-ai sync` runs in an environment where a native harness has a directory at its config path,
- **THEN** the command MUST NOT abort with an I/O error.
- **AND** the command MUST successfully install/sync all other detected harnesses.
- **AND** the install manifest, `state.json`, and transaction journal MUST successfully commit.
- **AND** the stderr output MUST contain the warning line naming the affected harness and directory path.
