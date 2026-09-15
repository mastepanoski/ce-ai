---
title: "Detecting Divergence Between Claude Code Native Plugin Marketplace and ce-ai"
category: "architecture"
date: "2026-09-08"
tags:
  - claude
  - doctor
  - marketplace
  - plugins
  - divergence
components:
  - harness::claude
  - commands::doctor
applies_when: "Diagnosing why updates to compound-engineering via ce-ai upgrade do not take effect in Claude Code, or inspecting native plugin marketplace version differences across scopes"
problem_type: architectural_refactor
---

# Detecting Divergence Between Claude Code Native Plugin Marketplace and ce-ai

## Problem & Context
`ce-ai` manages an isolated installation tree of `compound-engineering` across configured harnesses (`~/.claude/compound-engineering/`, `~/.config/opencode/compound-engineering/`, etc.) tracked in `state.json` (`installed_harnesses` and `release_provenance`).

However, Claude Code also maintains an internal native plugin marketplace at `~/.claude/plugins/` (tracked in `installed_plugins.json` and cached under `~/.claude/plugins/cache/`). When a user enables `compound-engineering@compound-engineering-plugin` via Claude Code's native plugin manager (`~/.claude/settings.json` has `enabledPlugins: { "compound-engineering@compound-engineering-plugin": true }`), Claude Code resolves and runs skills directly from its native cache, completely bypassing the content tree maintained by `ce-ai`.

Consequently, running `ce-ai upgrade` correctly updates `ce-ai`'s tree to the latest release (e.g. `3.24.0`), but active Claude Code sessions continue running an older native version (e.g. `3.8.4` or `3.17.1`). Prior to this fix, `ce-ai doctor` had zero visibility into native plugin installations, resulting in silent version divergence that required manual filesystem inspection to diagnose.

## Architectural Decision
1. **Best-Effort Native Marketplace Inspection (`src/harness/claude.rs`)**:
   - `check_claude_marketplace_divergence(&state, &cwd, &claude_dir)` inspects `<claude_dir>/plugins/installed_plugins.json` using tolerant serde deserialization.
   - Guarded by the presence of the `claude` harness in `state.installed_harnesses`.
   - Strictly read-only: zero mutations or command executions against `<claude_dir>/plugins/`.
2. **Scope Applicability Logic**:
   - `user` scope: applies globally to all directories.
   - `project` and `local` scopes: applies when the working directory (`cwd`) or repo root matches or is a descendant of the entry's `projectPath`.
   - Avoids guessing internal precedence between scopes by reporting all applicable divergent entries transparently.
3. **Semantic Version Normalization**:
   - `normalize_plugin_version` strips `compound-engineering-`, `compound-engineering@`, and `v` prefixes before comparison, preventing false positives when comparing tags like `compound-engineering-v3.24.0` against native version `3.24.0`.
4. **Advisory Non-Blocking Doctor Diagnostic (`src/commands/doctor.rs`)**:
   - Emits `doctor-info:` notices identifying the plugin, scope, native version, and `ce-ai` managed version.
   - Details the exact remediation command: `claude plugin marketplace update <marketplace> && claude plugin update <plugin>`.
   - Does not add to `findings`, ensuring `ce-ai doctor` exits `0`.

## Sibling Probe: Kimi Code (v1.48.0)

The same divergence class exists for Kimi Code's native plugin manager, and `ce-ai` ships a structurally identical probe (`src/harness/kimi.rs`). This document stays Claude-focused; the Kimi probe is documented here as a cross-link only:

- `check_kimi_marketplace_divergence` (kimi.rs:193) reads `~/.kimi-code/plugins/installed.json`, reuses `normalize_plugin_version` from the Claude adapter, and applies the same scope-applicability rules (global always, workspace only under the registered `target_dir`), emitting the same advisory `doctor-info` contract.
- `check_kimi_orphan_managed_tree` (kimi.rs:303) additionally detects the inverse failure mode: a ce-ai managed tree under `~/.kimi-code/compound-engineering/` that Kimi's config does not reference via `extra_skill_dirs` in `~/.kimi-code/config.toml`, reported as `doctor-warn` (the ce-ai-managed skills are installed but inactive for Kimi).
- Both are wired into `ce-ai doctor` (doctor.rs:284-303) as advisory output only — neither contributes to `findings`, so `ce-ai doctor` still exits `0`.
- See the dedicated architectural decision doc for Kimi: [`kimi-native-plugin-divergence-visibility-not-control.md`](kimi-native-plugin-divergence-visibility-not-control.md) ([Issue #349](https://github.com/mastepanoski/ce-ai/issues/349)), which independently ratifies the "visibility, not control" principle against Kimi's runtime architecture.
