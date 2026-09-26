# Design: Managed Adoption Block Turn-0 Directives & Progressive OpenSpec Accuracy

## Architectural Overview
This change modifies the text template rendered for `AdoptionTier::Full` and coordinates the system version constant `BLOCK_VERSION`.

## 1. Managed Block Template (`src/commands/init_prj.rs`)

### Turn-0 Directive Replacement:
```markdown
### ⚡ Turn-0 Session Directives (Zero-Step Drift Recovery)
`ce-ai` auto-installs a SessionStart hook on supported harnesses that runs `ce-ai workflow resume --json` and injects live FSM state (Git working tree, active branch, manifest SHA256 integrity, OpenSpec progress) into context before your first turn. If that state is already present at session start, treat it as current — do NOT re-run the command. Only run `ce-ai workflow resume` yourself when no such context was injected (harness without hook support, or the hook is disabled) or when you suspect the injected state has gone stale mid-session (e.g. after a `git` operation you didn't make through your own tool calls).
```

### Stage 2 OpenSpec Replacement:
```markdown
### Stage 2 OpenSpec Enforcement Requirements
OpenSpec is authored progressively. Before writing feature code, agents MUST verify `openspec/changes/<feature_name>/` contains the frozen Stage 2 contract:
- `proposal.md`: Problem statement, in-scope/out-of-scope boundaries, and success criteria.
- `exploration.md`: Technical investigation and architectural tradeoffs.
- `design.md`: Technical design, system architecture, structs, and API/CLI contracts.
- `spec.md`: Formal requirements using `WHEN ... THEN ...` format and explicit acceptance criteria.

`tasks.md` (the executable task checklist) is generated from that frozen contract in Stage 3 — do not require it before Stage 2 is complete, but do not open a PR without it.
```

## 2. Version Bump
- In `src/commands/init_prj.rs`:
  ```rust
  pub const BLOCK_VERSION: u32 = 7;
  ```
- In `tests/cli.rs`:
  ```rust
  const CUR_BLOCK_VERSION: u32 = 7;
  ```

## 3. Test Fixture Synchronization
- `src/commands/tests/init_prj.rs`:
  - Add test asserting exact phrases of new Turn-0 text and Stage 2 progressive OpenSpec text.
- `tests/cli.rs`:
  - Replace hardcoded `6` in `state_val["projects"][0]["block_version"]` with `CUR_BLOCK_VERSION`.
  - Add test verifying upgrade from stale `v=6` block to `v=7` with updated directives.
