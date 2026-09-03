# Specification: Cursor sessionStart Lifecycle Hook Integration

## Requirements

### R1: Detection (`has_session_start_hook`)
- WHEN `hooks_path` does not exist OR is invalid JSON, THEN `has_session_start_hook` returns `false`.
- WHEN `hooks_path` contains `hooks.sessionStart` array with an entry where `command == "ce-ai workflow resume --json"`, THEN `has_session_start_hook` returns `true`.

### R2: Injection (`ensure_session_start_hook`)
- WHEN `has_session_start_hook` is true, THEN `ensure_session_start_hook` returns `Ok(false)` without modifying the file.
- WHEN `hooks_path` exists with pre-existing user hooks or extra settings, THEN `ensure_session_start_hook` preserves them, appends the managed hook under `hooks.sessionStart`, and writes atomically.

### R3: Removal (`remove_session_start_hook`)
- WHEN `remove_session_start_hook` is called, THEN the managed entry is stripped from `hooks.sessionStart`.
- WHEN `hooks` contains no other hooks and the file has no extra custom keys beyond `"version": 1`, THEN the file is deleted.
- WHEN the `.cursor` parent directory becomes empty, THEN it is cleanly pruned.

### R4: CLI Commands
- WHEN `ce-ai init-prj` is executed in a repository with `.cursor/`, THEN `.cursor/hooks.json` is ensured.
- WHEN `ce-ai doctor` is executed in an adopted repo with `.cursor/` missing the hook, THEN a warning finding is reported.
- WHEN `ce-ai deinit-prj` is executed, THEN the managed hook is removed from `.cursor/hooks.json`.
