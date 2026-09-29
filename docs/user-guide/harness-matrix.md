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
| `custom` | `~/.ce-ai/custom_harness.json` or `--plugins-dir/--skills-dir/--rules-file` | User-Configured Directory Install; surgical manifest-driven uninstall |
| `deepseek` | *De-scoped* (`dsh` developer preview) | Returns `CeError::Usage` (exit code 2) guiding user to native adapters |

## Usage Telemetry Ingestion Matrix

`ce-ai usage sync` captures local turn-level token and model telemetry from supported coding harnesses into the author's local usage ledger (`~/.ce-ai/usage/<author>.jsonl`):

| Harness Identifier | Transcript Location | Supported Telemetry |
| :--- | :--- | :--- |
| `claude` | `~/.claude/projects/` | Input, output, cache read/write |
| `opencode` | `~/.local/share/opencode/sessions/` or `~/.config/opencode/sessions/` | Input, output, cache read/write, reasoning |
| `codex` | `~/.codex/sessions/*.jsonl` | Input, output, cache read (`cached_tokens`), reasoning |
| `pi` | `~/.pi/agent/sessions/*.json` | Input, output, cache read/write, reasoning |

Use `ce-ai usage sync --harness <name>` to scope capture to a single harness, or `ce-ai usage sync` (or `--harness all`) to discover and ingest across all available harnesses.

## Safety Guarantees

For supported installations with an existing harness configuration, `ce-ai` provides these safeguards:

- A timestamped per-harness pre-mutation backup in `~/.ce-ai/backups/`.
- SHA256 manifest indexing per installed file for drift detection.
- Atomic writes (`write_atomic`: tempfile + rename) for managed configuration writes.
- On uninstall, restoration of the matching pre-install snapshot when one exists; otherwise, removal of CE-managed entries.

See [Backup & Uninstall](backup-and-uninstall.md) for the full lifecycle.
