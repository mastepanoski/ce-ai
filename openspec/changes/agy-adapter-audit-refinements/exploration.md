# Exploration: Antigravity (AGY) Adapter Audit Refinements

## Technical Investigation
1. **Environment Variables**:
   - Google Antigravity defaults to `~/.gemini`.
   - `ce-ai` supports `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` for custom directory relocation (useful in testing, containerized environments, and custom user setups).
   - Documenting these as `ce-ai` extension conventions prevents ambiguity regarding upstream Google documentation versus `ce-ai` extensions.
2. **Project Rules**:
   - `ce-ai init-prj` writes project rules to `.agents/rules/compound-engineering.md` (when `.agents/` pre-exists) and `GEMINI.md` (project root). `GEMINI.md` guarantees root instruction loading across Antigravity tools.
3. **Name Collision Policy**:
   - When registering a local stdio tool (e.g. `codegraph`, `engram`) with `register_agy_mcp_server`, if an existing server entry under the same name previously had `serverUrl` set (remote SSE/HTTP), `register_agy_mcp_server` updates `command`, `args`, `env` and resets `server_url` to `None`. This guarantees valid stdio server JSON structure without dual `command` and `serverUrl` fields.
