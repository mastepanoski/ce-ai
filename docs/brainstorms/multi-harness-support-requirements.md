# Multi-Harness Support Requirements

**Date:** 2026-08-21  
**Status:** Approved Requirements  
**Target Feature:** Issue #1 — Support for Pi, Claude Code, Codex, Grok, Kimi Code, AGY, Cursor, Copilot, DeepSeek, fx.sh, and Custom Fallback Mode  

---

## 🎯 Executive Summary & Problem Statement

`ce-ai` currently manages plugin installation, model assignments, SHA256 file manifests, and backup restoration for the **OpenCode** harness. As AI coding environments diversify, developers use different AI harnesses (such as Claude Code, Pi, Cursor, Copilot, Codex, Grok, Kimi, AGY, DeepSeek, and custom shell harnesses like `fx.sh`).

`ce-ai` must provide unified, multi-harness management that allows users to install, synchronize, and assign models across any AI coding harness without destroying existing user configurations or clobbering unmanaged rules.

---

## 👥 Key Actors & Use Cases

- **Developer / Agent Operator**: Uses `ce-ai install --harness <name>` to install Compound Engineering across one or multiple harnesses.
- **Multi-Harness Developer**: Uses `ce-ai install --all` or `ce-ai sync` to keep all installed harnesses updated with identical skill packages and model assignments.
- **Custom Harness User**: Uses `ce-ai install --harness custom --plugins-dir <path>` to install CE skills into unlisted or proprietary harnesses.

---

## 🛠️ Supported Harness Matrix

| Harness Identifier | Config File / Location | Strategy |
| :--- | :--- | :--- |
| `opencode` | `~/.config/opencode/opencode.json` | JSON Array Merger (`plugin` & `skills`) |
| `claude` | `~/.claude.json` / `~/.config/claude/` | JSON Key Merger & Skill Path Injector |
| `pi` | `~/.pi/config.json` / `~/.pi/skills/` | JSON Merger & Native Skill Directory Copy |
| `cursor` | `.cursorrules` / `.cursor/rules/` | Markdown Rule Block Ingestion (`<!-- CE-AI BEGIN -->`) |
| `copilot` | `.github/copilot-instructions.md` | Markdown Instruction Block Ingestion |
| `codex` | `~/.codex/config.json` | JSON Config Merger |
| `grok` | `~/.grok/config.json` | JSON Config Merger |
| `kimi` | `~/.kimi/config.json` | JSON Config Merger |
| `agy` | `~/.gemini/antigravity-cli/config.json` | JSON Merger & Native Skill Copy |
| `deepseek` | `~/.deepseek/config.json` | JSON Config Merger |
| `fx` | `~/.fx/config.json` | JSON / Shell Loader Script Injector |
| `custom` | Specified via CLI flags / interactive prompt | Flexible Directory Copy & Optional Rule Ingestion |

---

## 🔍 Core Requirements (R1 - R5)

### R1: Harness Registry & Resolver
- The CLI must parse `--harness <identifier>` against a statically typed `HarnessKind` enum.
- Supported identifiers: `opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`.
- `--harness all` or `--all` must auto-detect installed harnesses on the host system and run the target command for each detected harness.

### R2: Structured JSON Merging + Markdown Rule Ingestion Adapters
- For JSON-based harnesses (`opencode`, `claude`, `pi`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`), `ce-ai` MUST parse existing JSON, update managed keys without deleting unmanaged user keys, and write back atomically via `write_atomic`.
- For Markdown-based harnesses (`cursor`, `copilot`), `ce-ai` MUST insert/update a demarcated comment section (`<!-- CE-AI MANAGED BLOCK BEGIN --> ... <!-- CE-AI MANAGED BLOCK END -->`) without modifying surrounding user instructions.

### R3: Custom Harness Fallback Mode (`--harness custom`)
- Accept `--plugins-dir <path>`, `--skills-dir <path>`, and `--rules-file <path>` CLI flags.
- If flags are omitted in TTY mode, interactively prompt the user using `inquire`.
- Store custom harness definitions in `state.json` under custom harness profiles for seamless `sync`, `status`, and `uninstall`.

### R4: Multi-Harness Model Assignment Synchronization
- `ce-ai models set <slot> = <model>` updates central state in `~/.ce-ai/state.json`.
- The translation engine translates slot assignments (`ce-brainstorm`, `ce-plan`, `ce-work`, `ce-code-review`, `ce-debug`) into native config structures for all installed harnesses.

### R5: Safety, Manifest Integrity & Atomic Uninstallation
- All mutations across all harnesses MUST record SHA256 file hashes in `manifest.json`.
- Automatic timestamped backups (`~/.ce-ai/backups/<harness>/<timestamp>/`) MUST be created before any file modification.
- `ce-ai uninstall --harness <name>` MUST cleanly restore pre-installation backups and remove managed files.

---

## 🚫 Out of Scope / Non-Goals

- Replacing or overriding user-written non-CE custom skills or rules in third-party harnesses.
- Executing model API calls directly (model routing is delegated to the respective harness runtime).

---

## ✅ Success Criteria

1. 100% of supported harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`) pass unit, CLI integration, and E2E validation.
2. Zero user configuration clobbering across all JSON and Markdown harnesses.
3. Full compliance with ISO 27001/27002, ISO 42001, NIST AI RMF, and `ce-ai` Definition of Done.
