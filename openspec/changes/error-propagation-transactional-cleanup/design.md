# OpenSpec Design: Error Propagation & Transactional Order

- **Change:** `error-propagation-transactional-cleanup`
- **Issue:** #162 (P1)

---

## 📐 1. Execution Flow for `uninstall.rs`

```rust
// 1. Perform required filesystem mutations, propagating errors via `?`
for target in &targets {
    if let Ok(harness_kind) = target.parse::<HarnessKind>() {
        let config_dir = harness_kind.harness_dir(&home_dir);
        let target_config = harness_kind.config_path(&config_dir);
        let backups = ctx.config_dir.join("backups");
        if let Some(backup) = newest_backup_for_harness(&backups, target)? {
            restore_backup_by_id(&backups, &backup.id, &target_config)?;
        } else if target_config.exists() {
            std::fs::remove_file(&target_config)?;
        }
        let managed_dir = config_dir.join(MANAGED_DIR);
        if managed_dir.exists() {
            std::fs::remove_dir_all(&managed_dir)?;
        }
    }
    state.installed_harnesses.retain(|h| h["name"].as_str() != Some(target.as_str()));
}

// 2. Best-effort non-critical registry cleanup
if let Err(e) = crate::source::registry::SkillRegistry::remove(ctx) {
    if !ctx.quiet {
        eprintln!("warning: skill registry cleanup failed: {e}");
    }
}

// 3. Commit state ONLY after required operations succeed
state.save(&state_path)?;
```

---

## 📐 2. Execution Flow for `deinit_prj.rs`

```rust
// 1. Required file removal / clean write of AGENTS.md / CLAUDE.md
if created_file && is_empty_now {
    fs::remove_file(&agents_file)?;
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

// 2. Required .gitignore block cleanup (MUST execute BEFORE state.save)
let gitignore_file = target_dir.join(".gitignore");
if gitignore_file.exists() {
    let gi_text = fs::read_to_string(&gitignore_file)?;
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

// 3. Commit state ONLY after required operations succeed
if let Some(idx) = registry_pos {
    state.projects.remove(idx);
    state.save(&global_state_path)?;
}
```

---

## 📐 3. Execution Flow for `init_prj.rs`

```rust
// 1. Write AGENTS.md and CLAUDE.md (required)
write_atomic(&agents_file, block.as_bytes())?;

// 2. Inject .gitignore sentinel block (required; MUST execute BEFORE state.save)
let gitignore_file = target_dir.join(".gitignore");
let gitignore_block = format!(
    "{}\n.ce-ai/skills-registry.json\n{}\n",
    GITIGNORE_BEGIN_MARKER, GITIGNORE_END_MARKER
);
let gitignore_text = if gitignore_file.exists() {
    fs::read_to_string(&gitignore_file)?
} else {
    String::new()
};
if !gitignore_text.contains(GITIGNORE_BEGIN_MARKER) {
    let mut updated_gi = gitignore_text;
    if !updated_gi.is_empty() && !updated_gi.ends_with('\n') {
        updated_gi.push('\n');
    }
    updated_gi.push_str(&gitignore_block);
    crate::state::write_atomic(&gitignore_file, updated_gi.as_bytes())?;
}

// 3. Best-effort skill registry sync
if let Err(e) = crate::source::registry::SkillRegistry::sync_registry(ctx) {
    if !ctx.quiet {
        eprintln!("warning: skill registry sync failed: {e}");
    }
}

// 4. Commit state ONLY after required operations succeed
state.projects.push(...);
state.save(&global_state_path)?;
```
