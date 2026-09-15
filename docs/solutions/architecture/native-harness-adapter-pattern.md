---
title: "Native Harness Adapter Integration Pattern"
category: "architecture"
problem_type: "architecture"
date: "2026-09-15"
applies_when: "integrating new AI coding harnesses or refactoring existing adapters to support native configuration formats, MCP servers, and environment overrides"
tags:
  - harness
  - adapter
  - mcp
  - toml
  - json
  - concurrency
components:
  - src/harness/copilot.rs
  - src/harness/grok.rs
  - src/harness/kimi.rs
  - src/harness/mod.rs
---

# Native Harness Adapter Integration Pattern

## Problem Statement
In earlier iterations of `ce-ai`, external AI coding tools (e.g. GitHub Copilot CLI, xAI Grok Build CLI, Moonshot Kimi Code CLI) were modeled as generic JSON harnesses mapped to standard paths like `~/.config/<harness>/<harness>.json`. In reality, modern AI harnesses exhibit divergent native conventions:
- **Divergent Root Directories**: Dedicated native directories like `~/.copilot/`, `~/.grok/`, or `~/.kimi-code/`, rather than `~/.config/`.
- **Divergent Schema & Formats**: TOML tables (`[mcp_servers.<name>]` in `~/.grok/config.toml`), specialized JSON objects (`mcpServers` in `~/.copilot/mcp-config.json` and `~/.kimi-code/mcp.json`), or custom rule folders (`.grok/rules/`, `.github/copilot-instructions.md`, `.kimi-code/rules/`).
- **Isolation and Test Concurrency**: Testing harness root resolution in parallel cargo test runners caused race conditions when modifying environment variables concurrently.

## Unified Architectural Solution

`ce-ai` standardizes native harness integration via dedicated adapter modules in `src/harness/<name>.rs` adhering to 5 core architectural invariants:

### 1. Harness Configuration Matrix

| Harness | Adapter Module | Configuration Path | MCP Server Format | Environment Override | Skills Directory |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Copilot** | `src/harness/copilot.rs` | `~/.copilot/mcp-config.json` | JSON (`mcpServers.<name>`) | `$COPILOT_CONFIG_DIR` | `~/.copilot/skills/` |
| **Grok** | `src/harness/grok.rs` | `~/.grok/config.toml` | TOML (`[mcp_servers.<name>]`) | `$GROK_HOME` | `~/.grok/skills/` |
| **Kimi** | `src/harness/kimi.rs` | `~/.kimi-code/mcp.json` | JSON (`mcpServers.<name>`) | `$KIMI_CODE_HOME` | `~/.kimi-code/skills/` |

### 2. Surgical Configuration Preservation
When modifying native configuration files:
- **User Key Retention**: Never overwrite unmanaged user keys or custom MCP servers. Read the existing AST (`serde_json::Value` or `toml::Table`), mutate only the targeted sidecar entries (`ce-ai` managed servers), and serialize cleanly using `crate::state::write_atomic`.
- **Clean Environment Map Replacement**: When updating server environment variables, replace the mapping atomically:
  ```rust
  server_entry.env = env.clone();
  ```
  Coupled with `#[serde(skip_serializing_if = "BTreeMap::is_empty")]`, this guarantees empty environment variable maps are omitted entirely rather than leaving dangling or stale keys.

### 3. Thread-Safe Test Isolation (`HARNESS_ENV_LOCK`)
Native directory resolution functions inspect environment overrides (`$COPILOT_CONFIG_DIR`, `$GROK_HOME`, `$KIMI_CODE_HOME`) before falling back to `$HOME`.
Because Cargo runs unit tests in parallel threads within the same process, modifying environment variables without synchronization causes cross-test race conditions.
All harness tests modify environment overrides under a process-wide mutex guard:
```rust
let _guard = crate::harness::HARNESS_ENV_LOCK.lock().unwrap();
std::env::set_var("GROK_HOME", custom_path);
// Test assertions...
std::env::remove_var("GROK_HOME");
```

### 4. Demarcated Project Rule Adoption
Project rule adoption (`ce-ai init-prj`) injects standardized Compound Engineering guidelines into the harness's expected rule location:
- Copilot: `.github/hooks/hooks.json` (SessionStart hook) and `.github/copilot-instructions.md`.
- Grok: `.grok/rules/compound-engineering.md`.
- Kimi: `.kimi-code/rules/compound-engineering.md` or `.kimi-code/AGENTS.md`.
All blocks are enclosed within SHA-verified delimiters (`<!-- ce-ai:block begin ... -->`), enabling deterministic drift detection and idempotent updates.

### 5. Surgical Uninstallation Lifecycle
Running `ce-ai uninstall --harness <name>`:
- Unregisters `ce-ai` companion sidecars (e.g. Engram, CodeGraph) from the native config file.
- Removes the managed skills directory without deleting the harness configuration file or user custom skills.
- Logs explicit warnings to `stderr` if permissions prevent directory deletion, rather than silently suppressing errors with `let _ =`.

## Verification & Testing
- **Unit Tests**: In `src/harness/<name>.rs`, verify schema serialization, zero OpenCode key leakage, clean env map replacement, and thread safety under `HARNESS_ENV_LOCK`.
- **CLI Integration Tests**: In `tests/cli.rs`, verify full lifecycle execution (`install`, `sync`, `init-prj`, `uninstall`) using hermetic temporary directories and custom environment variables.
