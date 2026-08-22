# Harness Matrix

> **Intent**: Reference — look up which configuration file and merge strategy each supported AI harness uses. For installation workflows, see the [Installation & Coexistence Guide](installation-and-coexistence-mechanisms.md).

`ce-ai` supports 12 AI coding harnesses with three integration strategies:

1. **Native JSON merger** (`opencode`, `claude`, `pi`): parses structured config, updates targeted keys, preserves unmanaged user entries.
2. **Markdown rule-block ingestion** (`cursor`, `copilot`): injects a delimited managed block (`<!-- CE-AI MANAGED BLOCK -->`) into rules files.
3. **Generic JSON merger** (all others): safe structural merge for harnesses without a dedicated adapter.

## Supported Harness Matrix

| Harness Identifier | Config File / Location | Strategy |
| :--- | :--- | :--- |
| `opencode` | `~/.config/opencode/opencode.json` | JSON Array Merger (`plugin` & `skills`) |
| `claude` | `~/.claude.json` / `~/.config/claude/` | JSON Config Merger |
| `pi` | `~/.pi/config.json` / `~/.pi/skills/` | JSON Merger & Native Skill Directory Copy |
| `cursor` | `.cursorrules` / `.cursor/rules/` | Markdown Rule Block Ingestion (`<!-- CE-AI MANAGED BLOCK -->`) |
| `copilot` | `.github/copilot-instructions.md` | Markdown Instruction Block Ingestion |
| `codex` | `~/.codex/config.json` | Generic JSON Config Merger |
| `grok` | `~/.grok/config.json` | Generic JSON Config Merger |
| `kimi` | `~/.kimi/config.json` | Generic JSON Config Merger |
| `agy` | `~/.gemini/antigravity-cli/config.json` | Generic JSON Merger & Skill Copy |
| `deepseek` | `~/.deepseek/config.json` | Generic JSON Config Merger |
| `fx` | `~/.fx/config.json` | Generic JSON Config Merger |
| `custom` | Specified via CLI flags | Fallback Custom Config Mode |

## Safety Guarantees

Every strategy is backed by the same guarantees:

- Pre-mutation timestamped backups in `~/.ce-ai/backups/` before any config write.
- SHA256 manifest indexing per installed file for drift detection.
- Atomic writes (`write_atomic`: tempfile + rename) — a crashed process never leaves a half-written config.
- Clean restoration via `ce-ai uninstall --harness <name>`.

See [Backup & Uninstall](backup-and-uninstall.md) for the full lifecycle.
