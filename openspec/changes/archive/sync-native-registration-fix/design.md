# Design: `sync-native-registration-fix`

## Registration arms (order inside `sync_with` loop)

```
Custom   → snapshot-driven re-copy (v1.19.0, unchanged)
Cursor   → register_cursor_mcp_server ×2
Claude   → register_claude_mcp_server ×2 + copy_managed_skills(config/skills)
Codex    → register_codex_mcp_server ×2  + copy_managed_skills(skills)
Copilot  → register_copilot_mcp_server ×2 + copy_managed_skills(skills)
Grok     → register_grok_mcp_server ×2  + copy_managed_skills(skills)
Kimi     → register_kimi_mcp_server ×2  + copy_managed_skills(skills)   ← NEW
Agy      → register_agy_mcp_server ×2   + copy_managed_skills(config/skills) ← NEW
Fx       → register_fx_mcp_server ×2    + copy_managed_skills(skills)   ← NEW
Pi       → copy_managed_skills(skills)                                  ← NEW (No-MCP)
Opencode → ensure_plugin_and_skills (own registration; errors propagate) ← made explicit
else     → CeError::Runtime("cannot re-sync unsupported harness '<name>'")
```

All `register_*` calls use the same `(codegraph → ["mcp"], engram → ["serve"])`
pair with an empty env map, matching install.

## Shared helper

```rust
fn copy_managed_skills(managed_dir: &Path, dest: &Path) -> Result<(), CeError>
```

- Source: `<managed_dir>/skills`; no-op when absent.
- Delegates to `source::archive::copy_dir_all`; IO failures become
  `CeError::Runtime` naming the destination (no swallowing).
- Existing Claude/Codex/Copilot/Grok arms refactored onto it. Claude's old
  `let _ = copy_dir_all(...)` becomes error-propagating — intentional
  invariant-#5 alignment, called out in the changelog.

## Verification matrix

Checked group widens to the eight directory-copying kinds:

```rust
matches!(kind, Claude | Codex | Copilot | Grok | Kimi | Agy | Pi | Fx)
```

Per-kind skills root:

```rust
fn sync_skills_root(kind: HarnessKind, home: &Path) -> PathBuf {
    let dir = kind.harness_dir(home);
    if kind == HarnessKind::Agy { dir.join("config").join("skills") }
    else { dir.join("skills") }
}
```

Empty desired tree keeps the existing "no managed skills tree present"
NotVerified path; drift still fails with exit code 6 via the shared tail.

## Error mapping

| Condition | Error | Exit |
| --- | --- | --- |
| Unsupported harness name in state.json reaches re-registration | `CeError::Runtime` | 1 |
| Managed-skills copy IO failure (any arm) | `CeError::Runtime` | 1 |
| Post-sync hash drift on any checked surface | `CeError::Verification` | 6 |

## Testing

CLI integration (`tests/cli.rs`), one test per fixed harness, all hermetic:

1. Install opencode (manifest anchor) + target harness with its relocation
   env pinned (`PI_CODING_AGENT_DIR`, `KIMI_CODE_HOME`,
   `ANTIGRAVITY_CONFIG_DIR`, `FX_HOME`).
2. Snapshot the native config file bytes.
3. Delete a managed skill file under that vendor's root (Agy:
   `config/skills/...`).
4. Run `sync`.
5. Assert: skill restored; config bytes byte-identical (no `plugin` /
   `skills.paths` injection); stdout contains `✓ <kind>`.
