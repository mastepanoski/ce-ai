# Design: Antigravity (AGY) Adapter Audit Refinements

## 1. Environment Variable Extensions
- `$ANTIGRAVITY_CONFIG_DIR`: Primary environment variable override for Google Antigravity configuration directory (defaults to `$HOME/.gemini`).
- `$GEMINI_HOME`: Secondary environment variable override for Google Antigravity configuration directory.
- Both environment variables are `ce-ai` extension conventions for custom directory relocation.

## 2. Project Rules Architecture
- `canonical_instruction_file()` returns `PathBuf::from("GEMINI.md")`.
- `derived_stub_files()` returns `vec![PathBuf::from(".agents/rules/compound-engineering.md")]`.
- `init-prj` adopts `GEMINI.md` and `.agents/rules/compound-engineering.md` if `.agents/` directory pre-exists.

## 3. Server Registration Collision Policy
- When `register_agy_mcp_server` registers a server name that already exists as a remote server entry (`serverUrl: Some(...)`), the entry is updated to a local stdio command server (`command`, `args`, `env`), and `server_url` is explicitly reset to `None`.
- Unmanaged remote server entries with different names are preserved intact with `serverUrl` and `headers`.
