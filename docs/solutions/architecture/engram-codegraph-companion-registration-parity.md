---
title: "Engram and CodeGraph Companion Registration Parity across OpenCode, Custom, Deepseek, and Pi"
category: "architecture"
date: "2026-09-05"
tags:
  - harness
  - mcp
  - engram
  - codegraph
  - opencode
  - custom-harness
  - pi
  - deepseek
components:
  - harness::registration
  - harness::custom
  - opencode::config
  - opencode::plugins
  - source::tools_registry
applies_when: "Configuring MCP companion servers, adding native or custom harnesses, or auditing tools discovery"
---

# Engram and CodeGraph Companion Registration Parity across OpenCode, Custom, Deepseek, and Pi

## Context

In `ce-ai`, the `registration_spec` strategy table (`src/harness/registration.rs`) manages auto-registration of companion MCP servers (`codegraph` via `["mcp"]`, `engram` via `["serve"]`) across supported AI harnesses during `install` and `sync`. However, four harnesses had a registration parity gap (Issue #307):
- **OpenCode**: Had a dedicated arm returning `None` in `registration_spec` with no companion auto-registration in `install` or `sync`, forcing manual `ce-ai tools install engram`.
- **Custom Harness**: Had a dedicated arm returning `None` in `registration_spec` and no mechanism for users to supply an MCP configuration file.
- **Deepseek**: Returned `None` without explicit architectural rationale documented in code and tests.
- **Pi**: Returned `None` for MCP registration but lacked explicit delivery contract documentation and characterization testing.

## Solution Architecture

Release v1.41.0 establishes complete architectural parity across all four harnesses while respecting harness-specific constraints:

### 1. OpenCode Companion Auto-Registration
- Added `crate::opencode::config::register_companions` writing `codegraph` and `engram` into `opencode.json` under `mcpServers`.
- Wired `register_companions` in `install.rs` (OpenCode arm) and `sync.rs` (OpenCode arm).
- Added symmetric unregistration (`unregister_companions` and `remove_session_start_plugin`) that removes managed companions and cleans up `opencode.json` if no custom user configuration remains.
- Added `opencode.json` to `find_mcp_config_paths` in `src/source/tools_registry.rs`.

### 2. Custom Harness MCP File Support
- Extended `CustomHarnessConfig` and `CustomConfigFlags` with optional `mcp_file: Option<PathBuf>`, configurable via CLI `--mcp-file` or persisted in `custom_harness.json`.
- Implemented `register_custom_mcp_server`, `unregister_custom_mcp_server`, and `register_companions` in `src/harness/custom.rs` with atomic writes.
- Wired companion registration into `install.rs` and `sync.rs` for `HarnessKind::Custom`, with symmetric unregistration in `uninstall.rs`.
- Added custom `mcp_file` discovery to `find_mcp_config_paths` and `tools install`.

### 3. Formalized Pi Delivery Contract
- Pi is strictly No-MCP by design (Objective 8: skills tree only at `~/.pi/agent/skills/`).
- Pi companion capabilities are delivered via PATH binaries (`codegraph`, `engram`) rather than an artificial JSON configuration file.
- Verified that `doctor` and `tools status` probe binaries on PATH, avoiding false-positive missing-MCP diagnostics for Pi.

### 4. Deepseek Preview Documentation
- Clarified de-scoped status in `registration_spec`: DeepSeek Harness (`dsh`) operates in preview using YAML patch layers under `~/.dsh`, and `install --harness deepseek` fails fast with a Usage error.
