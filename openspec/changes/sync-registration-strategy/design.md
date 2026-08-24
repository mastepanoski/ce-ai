# Design: `sync-registration-strategy`

## Table

```rust
type McpRegistrar = fn(&Path, &str, &str, &[&str],
                       &BTreeMap<String, String>) -> Result<(), CeError>;

struct RegistrationSpec {
    register_mcp: Option<McpRegistrar>,      // None = No-MCP (pi)
    skills_subpath: Option<&'static str>,     // "skills" | "config/skills"
}

fn registration_spec(kind: HarnessKind) -> Option<RegistrationSpec>
```

| Kind | register_mcp | skills_subpath |
| :--- | :--- | :--- |
| cursor | register_cursor_mcp_server | skills |
| claude | register_claude_mcp_server | skills |
| codex | register_codex_mcp_server | skills |
| copilot | register_copilot_mcp_server | skills |
| grok | register_grok_mcp_server | skills |
| kimi | register_kimi_mcp_server | skills |
| agy | register_agy_mcp_server | config/skills |
| fx | register_fx_mcp_server | skills |
| pi | None | skills |

`Custom` (snapshot-driven), `Opencode` (plugin/skills JSON writer) and
`Deepseek` (de-scoped) return `None` and keep dedicated call-site arms; the
match lists every variant explicitly, so a new kind fails compilation until
classified here.

## Dispatch

```
Custom   → snapshot flow (unchanged)
Opencode → ensure_plugin_and_skills (unchanged)
spec?    → registrar ×2 ("codegraph"/"mcp", "engram"/"serve") + skills copy
None     → CeError::Runtime naming the state entry
```

## Testing

- Existing 94 black-box CLI tests: unchanged and green (behavioral net).
- New unit test pins the table: specs present for the nine table-driven kinds
  (`pi` has no registrar; `agy` uses `config/skills`), `None` for
  opencode/custom/deepseek.
