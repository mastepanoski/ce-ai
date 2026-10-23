---
title: "Harness Architecture & Multi-Agent Integration"
domain: harnesses
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Harness Architecture & Multi-Agent Integration

## 1. Overview & Architectural Boundaries

`ce-ai` acts as a workflow integration and asset lifecycle manager across 12 AI coding harnesses: `opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, and `custom`. It enforces clean separation between harness-managed assets and user-owned configuration.

## 2. Capabilities & Requirements

### R1. Multi-Harness Detection & Registration
WHEN `ce-ai` inspects the host environment  
THEN the system MUST detect installed AI harnesses by scanning standard user configuration paths (`~/.config/opencode`, `~/.claude`, `~/.cursor`, etc.) without requiring manual path entry.

### R2. User Configuration Preservation
WHEN `ce-ai` mutates harness settings (e.g. `opencode.json`, `.claude/settings.json`)  
THEN the system MUST preserve all unmanaged custom plugins, custom skills, and user keybindings using surgical JSON parsing and re-serialization.

### R3. Managed Asset Directory Isolation
WHEN `ce-ai` installs or synchronizes plugin assets  
THEN all files MUST be isolated within dedicated managed subdirectories (`compound-engineering/`) and tracked in `manifest.json` with cryptographic SHA256 digests.

### R4. Visibility, Not Control Principle
WHEN interacting with external harnesses that feature private native marketplaces (e.g. Claude Code, Kimi Code)  
THEN `ce-ai` MUST limit operations to read-only version drift detection and diagnostic guidance, avoiding unauthorized writes into the harness's internal cache or registry.

## 3. Data Models & CLI Contracts

- `HarnessKind` enum: `OpenCode`, `Claude`, `Pi`, `Cursor`, `Copilot`, `Codex`, `Grok`, `Kimi`, `Agy`, `DeepSeek`, `Fx`, `Custom`.
- `InstallManifest`: `{ files: { [relative_path: string]: sha256_hex } }`
- CLI commands: `ce-ai install [--harness <name>]`, `ce-ai sync [--watch]`, `ce-ai upgrade`.

## 4. Invariants & Operational Boundaries

- All writes to harness configuration files MUST use `crate::state::write_atomic`.
- Backups MUST be generated prior to any destructive operation or major upgrade.
