# Technical Design: Claude MD Delegation & Deduplication

## 1. Syntax Detection (`src/harness/claude.rs`)

We introduce a dedicated parser function:
```rust
/// Returns true if the given markdown content delegates to an `AGENTS.md` file via Claude Code's `@` import syntax.
pub fn delegates_to_agents_md(content: &str) -> bool {
    let mut in_code_fence = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }

        // Support optional bullet prefixes: "- @AGENTS.md" or "* @AGENTS.md"
        let unbulleted = trimmed
            .strip_prefix('-')
            .or_else(|| trimmed.strip_prefix('*'))
            .map(|s| s.trim())
            .unwrap_or(trimmed);

        if let Some(rest) = unbulleted.strip_prefix('@') {
            let target_token = rest.split_whitespace().next().unwrap_or("");
            let clean_target = target_token.trim_matches(['"', '\'', '`']);
            if let Some(file_name) = std::path::Path::new(clean_target).file_name() {
                if file_name.to_string_lossy().eq_ignore_ascii_case("AGENTS.md") {
                    return true;
                }
            }
        }
    }
    false
}
```

## 2. Harness Reconciliation Logic (`src/commands/init_prj.rs`)

In `reconcile_project_harness_hooks`:
```rust
    // 2. Claude
    let claude_dir = target_dir.join(".claude");
    let claude_md_root = target_dir.join("CLAUDE.md");
    let claude_md_nested = claude_dir.join("CLAUDE.md");
    let target_agents_md = target_dir.join("AGENTS.md");

    let claude_rule_path_opt = if claude_md_root.exists() {
        Some(claude_md_root)
    } else if claude_md_nested.exists() {
        Some(claude_md_nested)
    } else if claude_dir.exists() {
        Some(claude_dir.join("CLAUDE.md"))
    } else {
        None
    };

    if let Some(claude_rule_path) = claude_rule_path_opt {
        let existing_content = if claude_rule_path.exists() {
            fs::read_to_string(&claude_rule_path).unwrap_or_default()
        } else {
            String::new()
        };

        let delegates = target_agents_md.exists()
            && crate::harness::claude::delegates_to_agents_md(&existing_content);

        if delegates {
            // Claude Code already imports AGENTS.md, which contains the managed block.
            // If CLAUDE.md has a duplicate managed block (from prior versions or edits), strip it.
            if existing_content.contains(crate::harness::claude::CE_MANAGED_BEGIN)
                || existing_content.contains(BLOCK_BEGIN_MARKER)
            {
                let stripped = crate::harness::claude::strip_managed_block(&existing_content);
                let mut cleaned = stripped.trim_end().to_string();
                if !cleaned.is_empty() {
                    let newline = if existing_content.contains("\r\n") { "\r\n" } else { "\n" };
                    cleaned.push_str(newline);
                }
                if cleaned != existing_content {
                    crate::state::write_atomic(&claude_rule_path, cleaned.as_bytes())?;
                }
            }
        } else {
            // No delegation to AGENTS.md: only update if .claude exists or user has custom instructions
            let has_user_instructions = !existing_content.trim().is_empty();
            if claude_dir.exists() || has_user_instructions {
                crate::harness::claude::update_claude_md(&claude_rule_path, inner_body)?;
            }
        }
    }

    if claude_dir.exists() {
        let settings_path = claude_dir.join("settings.json");
        let _ = crate::harness::claude::ensure_session_start_hook(&settings_path);
        let _ = crate::harness::claude::ensure_claude_gate_hook(&settings_path);
    }
```

## 3. De-adoption Consistency (`src/commands/deinit_prj.rs`)
In `deinit_prj.rs`, when cleaning up Claude rule files:
If `claude_rule` exists and has `c_text.trim() == "@AGENTS.md"`, it is recognized as a tool-generated stub and cleanly removed upon deinit, just like when `created_file` is true.

## 4. Invariants Preserved
- `crate::state::write_atomic` is used for all file updates.
- User comments and custom rules in `CLAUDE.md` are preserved.
- When `CLAUDE.md` does not delegate to `AGENTS.md`, it continues to receive the managed block.
