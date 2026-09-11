# Specification: Antigravity PreInvocation Turn-0 Drift Delivery

## Requirements

### R1: Hook Detection (`has_pre_invocation_hook`)
- WHEN `hooks_path` does not exist OR contains invalid JSON, THEN `has_pre_invocation_hook` returns `false`.
- WHEN `hooks_path` contains an entry with `command == "ce-ai workflow resume --pre-invocation"`, THEN `has_pre_invocation_hook` returns `true`.

### R2: Hook Injection (`ensure_pre_invocation_hook`)
- WHEN `has_pre_invocation_hook` is true, THEN `ensure_pre_invocation_hook` returns `Ok(false)` without modifying the file.
- WHEN `hooks_path` exists with pre-existing user hooks or extra groups, THEN `ensure_pre_invocation_hook` preserves them, adds `"compound-engineering".PreInvocation`, and writes atomically.

### R3: Hook Removal (`remove_pre_invocation_hook`)
- WHEN `remove_pre_invocation_hook` is called, THEN the managed command is stripped from `"compound-engineering".PreInvocation`.
- WHEN `"compound-engineering"` has no other hooks, THEN it is removed.
- WHEN `hooks.json` becomes empty, THEN the file is deleted.
- WHEN the `.agents` parent directory becomes empty, THEN it is pruned.

### R4: Deduplication in `workflow resume --pre-invocation`
- WHEN invoked on a new `conversationId`, THEN `workflow resume --pre-invocation` records the session marker and emits `injectSteps` containing `ephemeralMessage`.
- WHEN invoked again with the same `conversationId`, THEN it emits `{}` without duplicate injection.
