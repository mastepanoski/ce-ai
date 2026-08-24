---
title: "Google Antigravity (AGY) Adapter Audit Refinements"
category: "architecture"
tags: ["agy", "antigravity", "gemini", "harness-adapter", "audit", "mcp-json"]
date: "2026-08-24"
problem_type: "audit_refinements"
---

# Google Antigravity (AGY) Adapter Audit Refinements

## Problem Statement
The cross-adapter audit suite identified four specific audit findings for the Google Antigravity (`agy`) native harness adapter:
1. Environment variables `ANTIGRAVITY_CONFIG_DIR` and `GEMINI_HOME` are custom `ce-ai` relocation conventions not specified in official Google documentation.
2. Workspace-level `.agents/rules/*.md` rule loading is a `ce-ai` extension convention; `GEMINI.md` serves as canonical instruction file.
3. Managed server name collisions (`codegraph`, `engram`) convert pre-existing remote `serverUrl` / `url` entries to stdio command definitions, cleaning up stale remote headers/transport keys.
4. `HarnessAdapter` trait signatures evolved cleanly (`canonical_instruction_file`, `derived_stub_files`).

## Refinements & Architecture

### 1. OpenSpec Contract Documentation
- Documented `$ANTIGRAVITY_CONFIG_DIR` (primary) and `$GEMINI_HOME` (secondary) environment variable overrides in OpenSpec `design.md` and `spec.md`.
- Documented project rules architecture (`GEMINI.md` canonical, `.agents/rules/compound-engineering.md` derived stub).
- Documented server name collision policy (converting remote entries to stdio command servers while preserving non-colliding remote servers).

### 2. Implementation Hardening
- Added Serde alias `url` for `server_url` in `AgyMcpServer`.
- Ensured `register_agy_mcp_server` removes stale remote keys (`url`, `serverUrl`, `headers`, `transport`) from `extra` on server name collision.
- Used cross-platform path joining (`PathBuf::from(".agents").join("rules").join("compound-engineering.md")`) in `derived_stub_files`.
- Expanded unit tests in `src/harness/agy.rs` covering environment variable precedence, serverUrl reset, and OpenCode key exclusion.
