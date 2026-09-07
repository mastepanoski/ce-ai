---
title: "Companion MCP Registration Directory Guard & Contextual Error Wrapping"
category: "architecture"
date: "2026-09-07"
tags:
  - install
  - sync
  - mcp-registration
  - harness
  - directory-guard
  - error-handling
components:
  - harness::registration
  - commands::install
  - commands::sync
applies_when: "Investigating why 'install --harness all' fails with 'Is a directory (os error 21)', debugging companion MCP registration aborts, or understanding config file vs directory path validation in harness adapters"
---

# Companion MCP Registration Directory Guard & Contextual Error Wrapping

## Context & Problem

During `ce-ai install --harness all` (or `ce-ai sync`), `ce-ai` iterates through all detected harnesses. While the config backup step (`src/commands/install.rs:168`) used an `.is_file()` guard specifically to avoid attempting file reads on directory config paths (such as Pi's `~/.pi/agent/skills`), the MCP registration step in `RegistrationSpec::register_companions` lacked this guard.

Native MCP registrars check `if config_path.exists()` and invoke `std::fs::read_to_string(config_path)`. When `config_path` resolves to an existing directory on the host:
1. `std::fs::read_to_string` fails with `std::io::Error: Is a directory (os error 21)`.
2. `CeError::Io` formats this via passthrough `Display` as `error: I/O error: Is a directory (os error 21)` without naming the harness or path.
3. The error aborts the entire `--harness all` loop mid-execution before `state.save()` and `journal.complete()`.
4. Previous successfully installed harnesses are left unrecorded in `state.json` and rolled back by `Journal::begin` on the subsequent invocation.

## Solution Architecture

Release v1.44.1 introduces a centralized guard and contextual error wrapping in `RegistrationSpec::register_companions` (`src/harness/registration.rs`).

### 1. Self-Describing `RegistrationSpec`

`RegistrationSpec` now carries `pub(crate) kind: HarnessKind`. Every table-driven native registrar initializes `kind` in `registration_spec(kind: HarnessKind)`, allowing `register_companions` to identify its target harness without requiring callers to pass redundant arguments.

### 2. Directory Guard in `register_companions`

Before delegating to native MCP registrars, `register_companions` validates that `target_config` is a regular file if it exists:

```rust
if target_config.exists() && !target_config.is_file() {
    eprintln!(
        "warn: skipping companion MCP registration for {}: '{}' exists but is not a regular config file (expected a JSON/TOML file)",
        self.kind,
        target_config.display()
    );
    return Ok(());
}
```

- **Non-blocking warning**: Emits a clear warning identifying the specific harness and the problematic filesystem path.
- **Loop continuation**: Returns `Ok(())` so subsequent harnesses in `install --harness all` or `sync` continue installing without rollback.
- **Normal file creation preserved**: If `target_config` does not exist (`!exists()`), execution proceeds to the vendor registrar to create the config file cleanly.

### 3. Contextual Error Wrapping

Any unexpected `CeError::Io` returned during vendor registration is enriched with harness and path context:

```rust
let wrap_io = |err: CeError| match err {
    CeError::Io(e) => {
        let msg = format!("{}: '{}': {e}", self.kind, target_config.display());
        CeError::Io(std::io::Error::new(e.kind(), msg))
    }
    other => other,
};
```

This ensures that any future I/O failure names both what harness was executing and what filesystem path was being operated on, while maintaining `CeError::Io` exit code 4 compliance.
