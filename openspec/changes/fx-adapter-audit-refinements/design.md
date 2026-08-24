# Design: Vercel Labs fx Adapter Audit Refinements

## Path Resolution Strategy
`FxAdapter::default_config_path` resolves paths purely by inspecting string basenames without filesystem IO:
1. If `home.file_name() == Some("mcp.json")`: Returns `home.to_path_buf()`.
2. If `home.file_name() == Some(".fx")`: Returns `home.join("mcp.json")`.
3. Default: `self.kind().harness_dir(home).join("mcp.json")` (`~/.fx/mcp.json` or `$FX_HOME/mcp.json`).

Note: `FX_HOME` is a custom `ce-ai` extension for harness directory relocation.

## Error Propagation Policy on File Removal
When `unregister_fx_mcp_server` removes the last MCP server entry and config is empty:
- Attempts `std::fs::remove_file(config_path)`.
- If error kind is `io::ErrorKind::NotFound`, returns `Ok(())`.
- Any other IO error is propagated as `CeError::Io`.

## Extra Map Collision Cleanup
When registering a managed MCP server entry in `~/.fx/mcp.json`, `existing_extra.remove("type")` strips stale `type` keys from the `extra` serde map before inserting `"type": "local"`, preventing duplicate or conflicting keys in serialized output.
