# Specification: OpenCode V2 Plugin Loader

## WHEN a location loads with the CE plugin present
- **THEN** OpenCode V2 accepts the default export as a plugin definition with
  `id === "compound-engineering"` and a callable `setup`, and the plugin status is not
  `failed` (no err_c926271d-class validation error).

## WHEN `setup(ctx)` runs
- **THEN** every `SKILL.md` discovered under `<pluginDir>/../../skills` is registered via
  `ctx.skill.transform` with `id`/`name` from the frontmatter `name` field, optional
  `description`, the absolute file path, and full file content; registration is
  idempotent when the same skill id already exists.
- **AND** every discovered skill whose frontmatter does not set `user-invocable: false`
  is registered via `ctx.command.transform` as `<name>` with the V1 template semantics:
  executing the command submits
  `Load and execute the \`<name>\` skill.\n\n<arguments>` as a session prompt carrying
  the invocation delivery mode; names that already exist in `ctx.command.list()` are
  skipped.

## WHEN a `session.created` event arrives on the subscribed event stream
- **THEN** the plugin runs `ce-ai workflow resume` in `ctx.location.directory` and, on
  success, injects the output into that session via `ctx.session.synthetic` without
  triggering a model reply; failures never throw.

## WHEN a `session.idle` event arrives
- **THEN** the plugin runs `ce-ai workflow resume` in `ctx.location.directory` for FSM
  checkpoint evaluation, discarding the output; failures never throw.

## WHEN any agent-loop model request is assembled (`context` hook)
- **THEN** the current `ce-ai workflow resume` output is appended to `event.system` as a
  text part; if the command fails or returns empty, the request is left unmodified.

## WHEN a checkpoint-summary model request is assembled (`compaction` hook)
- **THEN** the current `ce-ai workflow resume` output is appended to `event.system` the
  same way, keeping the summarization request state-aware.

## WHEN the plugin unloads
- **THEN** the event subscription is aborted (no pending async work survives).

## WHEN ce-ai evaluates an installed/source loader's content
- **THEN** content is valid only if it contains `session.created` AND either `setup` or
  the legacy `export default function ceLoader` function-export prefix.
- **AND** the current V1-format canonical loader is classified invalid, so `install`,
  `sync`, and `ensure_session_start_plugin` converge it to the embedded V2
  `BUILTIN_LOADER`, and `doctor` reports the actionable "SessionStart plugin missing or
  outdated" finding.
- **AND** `ceLoader*` function-export fixtures (including the v9 upgrade tarball loader)
  remain valid so sync/upgrade never regress a newer loader (#325).

## Acceptance Criteria
- Node ESM import check passes: `default.id === "compound-engineering"`,
  `typeof default.setup === "function"`.
- Isolated OpenCode v2.0.18 instance loads the plugin without failure and exposes the
  registered skills/commands.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test`, and `make e2e` all pass.
