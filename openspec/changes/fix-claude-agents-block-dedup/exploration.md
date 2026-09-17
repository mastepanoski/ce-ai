# Exploration: Preventing Duplicate Managed Block Injection into CLAUDE.md

## Context & Issue Investigation
GitHub issue #377 identifies that `ce-ai init-prj` injects the managed 7-Stage block twice when Claude Code is used:
1. `AGENTS.md` receives `<!-- ce-ai:block begin v=4 ... --> ... <!-- ce-ai:block end -->`.
2. `CLAUDE.md` has `@AGENTS.md` on line 1, but also receives `<!-- CE-AI MANAGED BLOCK BEGIN --> ... <!-- CE-AI MANAGED BLOCK END -->`.

Claude Code treats `@filepath` as an inline inclusion. When Claude Code initializes, it evaluates `CLAUDE.md`, which reads `AGENTS.md`, and then reads its own body. This wastes ~500 prompt tokens on every session.

## Code Path Analysis
In `src/commands/init_prj.rs`:
```rust
// 1. Initial write to AGENTS.md
write_atomic(&agents_file, new_content.as_bytes())?;

// 2. Create derived stub if missing
let claude_stub = target_dir.join("CLAUDE.md");
if !claude_stub.exists() {
    let stub_content = format!("@AGENTS.md{}", newline);
    crate::state::write_atomic(&claude_stub, stub_content.as_bytes())?;
}
...
// 3. Harness reconciliation
reconcile_project_harness_hooks(&target_dir, inner_body)?;
```
Inside `reconcile_project_harness_hooks`:
```rust
let claude_dir = target_dir.join(".claude");
let claude_md_root = target_dir.join("CLAUDE.md");
let has_user_claude_md = claude_md_root.exists() && {
    let text = fs::read_to_string(&claude_md_root).unwrap_or_default();
    text.trim() != "@AGENTS.md"
};
if claude_dir.exists() || has_user_claude_md {
    let claude_rule_path = if claude_md_root.exists() {
        claude_md_root
    } else {
        claude_dir.join("CLAUDE.md")
    };
    crate::harness::claude::update_claude_md(&claude_rule_path, inner_body)?;
    ...
}
```

If `.claude/` exists, `claude_dir.exists()` is true. Even if `CLAUDE.md` only had `@AGENTS.md`, `update_claude_md` was called, appending the managed block to `CLAUDE.md`.
Furthermore, if a user had `@AGENTS.md` plus any comment or custom directive in `CLAUDE.md`, `text.trim() != "@AGENTS.md"` was true (`has_user_claude_md`), triggering `update_claude_md`.
Once written, `CLAUDE.md` has `CE_MANAGED_BEGIN`, making `text.trim() != "@AGENTS.md"` permanently true on every future `ce-ai sync`.

## Options Evaluated

### Option 1: Only check `text.trim() == "@AGENTS.md"`
If `text.trim() == "@AGENTS.md"`, do nothing.
- *Drawback*: If the user adds ANY custom guideline to `CLAUDE.md` (e.g. `@AGENTS.md\n\n# Project Notes`), `text.trim() != "@AGENTS.md"` is true and the duplicate block is injected anyway.
- *Verdict*: Fragile symptom patch.

### Option 2: Check for `@AGENTS.md` delegation anywhere in the file and heal existing duplicates (Recommended)
1. Provide a robust parser `delegates_to_agents_md(content)` that checks if any line outside code blocks begins with `@` and points to `AGENTS.md` (e.g. `@AGENTS.md`, `@./AGENTS.md`, `@../AGENTS.md`).
2. In `reconcile_project_harness_hooks`, resolve whether `AGENTS.md` exists and whether the candidate Claude file delegates to `AGENTS.md`.
3. If delegation is present and `AGENTS.md` exists:
   - Skip injecting the managed block into `CLAUDE.md`.
   - If `CLAUDE.md` already contains `CE_MANAGED_BEGIN`, strip it so existing repos automatically heal on next `sync` or `init-prj`.
4. If delegation is absent, retain existing behavior: inject the block into `CLAUDE.md`.
5. Ensure hooks (`.claude/settings.json`) are maintained independently whenever `.claude/` exists.
- *Verdict*: Fully resolves root cause, handles all user configurations, and auto-heals legacy duplication.

## Decision
Adopt **Option 2**.
