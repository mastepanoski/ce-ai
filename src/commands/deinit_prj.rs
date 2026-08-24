//! Project adoption deinit subcommand: `ce-ai deinit-prj`.

use std::fs;
use std::path::PathBuf;

use crate::commands::init_prj::{BLOCK_BEGIN_MARKER, BLOCK_END_MARKER};
use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::State;

/// Executes `ce-ai deinit-prj`.
pub fn run(ctx: &Context, target_path_opt: Option<PathBuf>) -> Result<(), CeError> {
    let raw_target = match target_path_opt {
        Some(p) => p,
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let target_dir = match raw_target.canonicalize() {
        Ok(canonical) => canonical,
        Err(_) => raw_target,
    };

    let agents_file = target_dir.join("AGENTS.md");
    let global_state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&global_state_path)?;

    let registry_pos = state.projects.iter().position(|p| p.path == target_dir);
    let created_file = registry_pos
        .map(|idx| state.projects[idx].created_file)
        .unwrap_or(false);

    if !agents_file.exists() {
        if let Some(idx) = registry_pos {
            if !ctx.dry_run {
                state.projects.remove(idx);
                state.save(&global_state_path)?;
            }
        }
        if !ctx.quiet {
            println!(
                "No 'AGENTS.md' found at '{}'. Project registry entry cleaned up.",
                target_dir.display()
            );
        }
        return Ok(());
    }

    let existing_content = fs::read_to_string(&agents_file)?;

    let cleaned_content = if let Some(start_idx) = existing_content.find(BLOCK_BEGIN_MARKER) {
        if let Some(end_rel_idx) = existing_content[start_idx..].find(BLOCK_END_MARKER) {
            let end_idx = start_idx + end_rel_idx + BLOCK_END_MARKER.len();

            let mut remaining = String::new();
            let before = &existing_content[..start_idx];
            let before_trimmed = if let Some(stripped) = before.strip_suffix("\r\n\r\n") {
                format!("{}\r\n", stripped)
            } else if let Some(stripped) = before.strip_suffix("\n\n") {
                format!("{}\n", stripped)
            } else {
                before.to_string()
            };
            remaining.push_str(&before_trimmed);

            // Skip trailing newlines immediately following the end marker if any
            let after = &existing_content[end_idx..];
            let after_trimmed = if let Some(stripped) = after.strip_prefix("\r\n") {
                stripped
            } else if let Some(stripped) = after.strip_prefix('\n') {
                stripped
            } else {
                after
            };

            remaining.push_str(after_trimmed);
            remaining
        } else {
            return Err(CeError::Runtime(format!(
                "malformed managed block in '{}': found begin marker without end marker",
                agents_file.display()
            )));
        }
    } else {
        existing_content.clone()
    };

    if !ctx.dry_run {
        let is_empty_now = cleaned_content.trim().is_empty();

        if created_file && is_empty_now {
            fs::remove_file(&agents_file)?;

            // Clean up derived CLAUDE.md stub if created and only contains @AGENTS.md
            let claude_stub = target_dir.join("CLAUDE.md");
            if claude_stub.exists() {
                if let Ok(stub_text) = fs::read_to_string(&claude_stub) {
                    if stub_text.trim() == "@AGENTS.md" {
                        fs::remove_file(&claude_stub)?;
                    }
                }
            }
        } else {
            crate::state::write_atomic(&agents_file, cleaned_content.as_bytes())?;
        }

        // Clean up Claude rule files (CLAUDE.md / .claude/CLAUDE.md)
        for claude_rule in &[
            target_dir.join("CLAUDE.md"),
            target_dir.join(".claude").join("CLAUDE.md"),
        ] {
            if claude_rule.exists() {
                if let Ok(c_text) = fs::read_to_string(claude_rule) {
                    if c_text.contains(crate::harness::claude::CE_MANAGED_BEGIN) {
                        let stripped = crate::harness::claude::strip_managed_block(&c_text);
                        if stripped.trim().is_empty() || stripped.trim() == "@AGENTS.md" {
                            let _ = fs::remove_file(claude_rule);
                        } else {
                            let _ = crate::state::write_atomic(claude_rule, stripped.as_bytes());
                        }
                    }
                }
            }
        }
        // Clean up Codex rule files (AGENTS.md / .codex/AGENTS.md)
        for codex_rule in &[
            target_dir.join("AGENTS.md"),
            target_dir.join(".codex").join("AGENTS.md"),
        ] {
            if codex_rule.exists() {
                if let Ok(c_text) = fs::read_to_string(codex_rule) {
                    if c_text.contains(crate::harness::codex::CE_MANAGED_BEGIN) {
                        let stripped = crate::harness::codex::strip_managed_block(&c_text);
                        if stripped.trim().is_empty() {
                            let _ = fs::remove_file(codex_rule);
                        } else {
                            let _ = crate::state::write_atomic(codex_rule, stripped.as_bytes());
                        }
                    }
                }
            }
        }

        // Clean up Copilot rule file (.github/copilot-instructions.md)
        let copilot_rule = target_dir.join(".github").join("copilot-instructions.md");
        if copilot_rule.exists() {
            if let Ok(c_text) = fs::read_to_string(&copilot_rule) {
                if c_text.contains(crate::harness::copilot::CE_MANAGED_BEGIN) {
                    let stripped = crate::harness::copilot::strip_managed_block(&c_text);
                    if stripped.trim().is_empty() {
                        let _ = fs::remove_file(&copilot_rule);
                    } else {
                        let _ = crate::state::write_atomic(&copilot_rule, stripped.as_bytes());
                    }
                }
            }
        }

        // Clean up Grok rule file (.grok/rules/compound-engineering.md)
        let grok_rule = target_dir
            .join(".grok")
            .join("rules")
            .join("compound-engineering.md");
        if grok_rule.exists() {
            if let Ok(c_text) = fs::read_to_string(&grok_rule) {
                if c_text.contains(crate::harness::CE_MANAGED_BEGIN) {
                    let stripped = crate::harness::strip_managed_rule_block(&c_text);
                    if stripped.trim().is_empty() {
                        let _ = fs::remove_file(&grok_rule);
                    } else {
                        let _ = crate::state::write_atomic(&grok_rule, stripped.as_bytes());
                    }
                }
            }
        }

        // Clean up Kimi rule file (.kimi-code/AGENTS.md and legacy .kimi-code/rules/compound-engineering.md)
        for kimi_path in &[
            target_dir.join(".kimi-code").join("AGENTS.md"),
            target_dir
                .join(".kimi-code")
                .join("rules")
                .join("compound-engineering.md"),
        ] {
            if kimi_path.exists() {
                if let Ok(c_text) = fs::read_to_string(kimi_path) {
                    if c_text.contains(crate::harness::CE_MANAGED_BEGIN) {
                        let stripped = crate::harness::strip_managed_rule_block(&c_text);
                        if stripped.trim().is_empty() {
                            let _ = fs::remove_file(kimi_path);
                            let _ = fs::remove_dir(target_dir.join(".kimi-code").join("rules"));
                        } else {
                            let _ = crate::state::write_atomic(kimi_path, stripped.as_bytes());
                        }
                    }
                }
            }
        }

        // Clean up Pi rule file (.pi/AGENTS.md)
        let pi_agents = target_dir.join(".pi").join("AGENTS.md");
        if pi_agents.exists() {
            if let Ok(c_text) = fs::read_to_string(&pi_agents) {
                if c_text.contains(crate::harness::CE_MANAGED_BEGIN) {
                    let stripped = crate::harness::strip_managed_rule_block(&c_text);
                    if stripped.trim().is_empty() {
                        let _ = fs::remove_file(&pi_agents);
                    } else {
                        let _ = crate::state::write_atomic(&pi_agents, stripped.as_bytes());
                    }
                }
            }
        }

        // Clean up fx rule file (.fx/AGENTS.md)
        let fx_agents = target_dir.join(".fx").join("AGENTS.md");
        if fx_agents.exists() {
            if let Ok(c_text) = fs::read_to_string(&fx_agents) {
                if c_text.contains(crate::harness::CE_MANAGED_BEGIN) {
                    let stripped = crate::harness::strip_managed_rule_block(&c_text);
                    if stripped.trim().is_empty() {
                        let _ = fs::remove_file(&fx_agents);
                    } else {
                        let _ = crate::state::write_atomic(&fx_agents, stripped.as_bytes());
                    }
                }
            }
        }

        // Clean up Antigravity rule files (.agents/rules/compound-engineering.md, GEMINI.md)
        let agy_rule = target_dir
            .join(".agents")
            .join("rules")
            .join("compound-engineering.md");
        if agy_rule.exists() {
            if let Ok(c_text) = fs::read_to_string(&agy_rule) {
                if c_text.contains(crate::harness::CE_MANAGED_BEGIN) {
                    let stripped = crate::harness::strip_managed_rule_block(&c_text);
                    if stripped.trim().is_empty() {
                        let _ = fs::remove_file(&agy_rule);
                    } else {
                        let _ = crate::state::write_atomic(&agy_rule, stripped.as_bytes());
                    }
                }
            }
        }
        let gemini_rule = target_dir.join("GEMINI.md");
        if gemini_rule.exists() {
            if let Ok(c_text) = fs::read_to_string(&gemini_rule) {
                if c_text.contains(crate::harness::CE_MANAGED_BEGIN) {
                    let stripped = crate::harness::strip_managed_rule_block(&c_text);
                    if stripped.trim().is_empty() {
                        let _ = fs::remove_file(&gemini_rule);
                    } else {
                        let _ = crate::state::write_atomic(&gemini_rule, stripped.as_bytes());
                    }
                }
            }
        }

        // Clean up sentinel-bounded .gitignore block (DEC-06)
        let gitignore_file = target_dir.join(".gitignore");
        if gitignore_file.exists() {
            let gi_text = fs::read_to_string(&gitignore_file)?;
            use crate::commands::init_prj::{GITIGNORE_BEGIN_MARKER, GITIGNORE_END_MARKER};
            if let Some(start_idx) = gi_text.find(GITIGNORE_BEGIN_MARKER) {
                if let Some(end_rel) = gi_text[start_idx..].find(GITIGNORE_END_MARKER) {
                    let end_idx = start_idx + end_rel + GITIGNORE_END_MARKER.len();
                    let mut cleaned_gi = String::new();
                    cleaned_gi.push_str(&gi_text[..start_idx]);
                    let rest = &gi_text[end_idx..];
                    let rest_trimmed = rest
                        .strip_prefix("\r\n")
                        .unwrap_or_else(|| rest.strip_prefix('\n').unwrap_or(rest));
                    cleaned_gi.push_str(rest_trimmed);
                    if cleaned_gi.trim().is_empty() {
                        fs::remove_file(&gitignore_file)?;
                    } else {
                        crate::state::write_atomic(&gitignore_file, cleaned_gi.as_bytes())?;
                    }
                }
            }
        }

        if let Some(idx) = registry_pos {
            state.projects.remove(idx);
            state.save(&global_state_path)?;
        }
    }

    if !ctx.quiet {
        println!(
            "✓ Removed project adoption block from '{}'",
            target_dir.display()
        );
    }

    Ok(())
}
