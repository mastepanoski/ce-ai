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
            let _ = fs::remove_file(&agents_file);

            // Clean up derived CLAUDE.md stub if created and only contains @AGENTS.md
            let claude_stub = target_dir.join("CLAUDE.md");
            if claude_stub.exists() {
                if let Ok(stub_text) = fs::read_to_string(&claude_stub) {
                    if stub_text.trim() == "@AGENTS.md" {
                        let _ = fs::remove_file(&claude_stub);
                    }
                }
            }
        } else {
            crate::state::write_atomic(&agents_file, cleaned_content.as_bytes())?;
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
