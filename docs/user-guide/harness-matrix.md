# Harness Matrix

> **Intent**: Reference — look up which configuration file and merge strategy each supported AI harness uses. For installation workflows, see the [Installation & Coexistence Guide](installation-and-coexistence-mechanisms.md).

`ce-ai` supports 10 native AI coding agent harnesses with dedicated native adapters, plus custom mode:

1. **Native JSON Adapters** (`opencode`, `claude`, `cursor`, `copilot`, `kimi`, `agy`, `fx`): parses structured JSON, updates targeted keys, preserves unmanaged user entries.
2. **Native TOML Adapters** (`codex`, `grok`): updates `[mcp_servers]` in native TOML configuration files.
3. **Native Skill Directory Adapter** (`pi`): copies skills natively into `~/.pi/agent/skills/`.
4. **Markdown & MDC Rules Ingestion** (`cursor`, `copilot`, project rules): injects non-destructive marker-delimited blocks into project rule files.

## Supported Harness Matrix

| Harness Identifier | Config File / Location | Strategy |
| :--- | :--- | :--- |
| `opencode` | `~/.config/opencode/opencode.json` | Native JSON Merger (`plugin` & `skills.paths`) |
| `claude` | `~/.claude.json` / `~/.claude/settings.json` | Native JSON `mcpServers` Merger |
| `pi` | `~/.pi/agent/skills/` | Native Skill Directory Manager |
| `cursor` | `~/.cursor/mcp.json` / `.cursor/rules/` | Native JSON `mcpServers` Merger & MDC Rules |
| `copilot` | `~/.config/github-copilot/mcp.json` | Native JSON `mcpServers` Merger & Markdown Rules |
| `codex` | `~/.codex/config.toml` | Native TOML `[mcp_servers]` Merger |
| `grok` | `~/.grok/config.toml` | Native TOML `[mcp_servers]` Merger |
| `kimi` | `~/.kimi-code/mcp.json` | Native JSON `mcpServers` Merger |
| `agy` | `~/.gemini/config/mcp_config.json` | Native JSON `mcpServers` Merger |
| `fx` | `~/.fx/mcp.json` | Native JSON `mcp` Root Key Merger |
| `custom` | Specified via CLI flags | Fallback Custom Config Mode |
| `deepseek` | *De-scoped* (`dsh` developer preview) | Returns `CeError::Usage` (exit code 2) guiding user to native adapters |

## Safety Guarantees

Every strategy is backed by the same guarantees:

- Pre-mutation timestamped backups in `~/.ce-ai/backups/` before any config write.
- SHA256 manifest indexing per installed file for drift detection.
- Atomic writes (`write_atomic`: tempfile + rename) — a crashed process never leaves a half-written config.
- Clean restoration via `ce-ai uninstall --harness <name>`.

See [Backup & Uninstall](backup-and-uninstall.md) for the full lifecycle.
