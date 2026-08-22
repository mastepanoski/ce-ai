# Issue #96: Multi-Harness Skill Registry Engine

## 1. Overview & Problem

Prior to Issue #96, `ce-ai` lacked a central, harness-neutral index for skill discovery and resolution across AI coding agent harnesses. As `ce-ai` expanded support to 12 distinct agent harnesses—including OpenCode, Claude Code, Cursor, Windsurf, Roo Code, Goose, and others—skills were stored and scanned across fragmented directory structures. Without a unified indexing layer, skill discovery behavior was inconsistent across host agents, skill metadata could drift undetected, and sub-agents had no standardized, secure mechanism to query available skills or verify file integrity before injecting instructions into prompts.

To address these architectural limitations, Issue #96 introduces the Multi-Harness Skill Registry Engine (`ce-ai skills`). The engine establishes a central master JSON index stored at `~/.ce-ai/skills-registry.json`, which indexes, parses, and validates `SKILL.md` instruction files across all 12 supported harnesses. Operating under a strict 4-tier precedence hierarchy and R3 path canonicalization security boundaries, the Skill Registry Engine guarantees deterministic skill resolution, resolution-time SHA256 integrity validation, explicit status degradation tagging, and complete lifecycle integration across `install`, `sync`, `upgrade`, `init-prj`, `deinit-prj`, and `uninstall` commands.

## 2. Key Design Decisions

The Skill Registry Engine is built around six foundational architectural decisions (`DEC-01` through `DEC-06`):

| Decision ID | Title | Key Details & Rationale |
| :--- | :--- | :--- |
| **DEC-01** | Central Master Index Schema & POSIX Atomic Persistence | Store master catalog at `~/.ce-ai/skills-registry.json` using `crate::state::write_atomic` with explicit Unix permission mode `0644` after writes. Ensures thread-safe, power-loss-resilient writes without risking zero-byte catalog corruption or improper access permissions. |
| **DEC-02** | 4-Tier Precedence Resolution Hierarchy | Layer skill discovery across 4 distinct priority tiers: Tier 1 (`.ce-ai/skills/`), Tier 2 (`.opencode/skills/`), Tier 3 (`~/.config/<harness>/skills/`), and Tier 4 (`~/.ce-ai/skills/` global managed). Workspace overrides always take precedence over global defaults. |
| **DEC-03** | Strict R3 Security Canonicalization Boundary | Validate candidate skill file paths via `canonicalize_and_validate_path`. Ensure canonical paths reside strictly within authorized root boundaries, preventing path traversal attacks (`../`) and symlink escapes (`R3`). |
| **DEC-04** | Robust Frontmatter YAML Parser | Implement a native, zero-panic frontmatter parser capable of extracting metadata (`name`, `description`, `scope`, `triggers`) from `SKILL.md` headers, supporting both inline comma-separated triggers and YAML bullet list items (`- trigger`). |
| **DEC-05** | Dual-Format Prompt Resolution & Integrity Tags | Provide human-readable Markdown prompt blocks (`## Skills to load...`) and machine-readable JSON outputs via `ce-ai skills resolve`. Validate SHA256 file hashes at resolution time, assigning explicit degradation tags (`paths-injected`, `fallback-fuzzy`, `none`). |
| **DEC-06** | Lifecycle Integration & Dry-Run Invariants | Integrate registry building across `install`, `sync`, `upgrade`, and `init-prj`. Enforce dry-run safety by gating disk writes behind `if !ctx.dry_run`, and maintain clean workspace state during `deinit-prj` and `uninstall` via sentinel-bounded `.gitignore` block management. |

## 3. 4-Tier Precedence & Security Boundary

### 4-Tier Precedence Resolution Hierarchy

Skill resolution processes candidate `SKILL.md` files from lowest to highest priority, allowing higher-tier directories to overwrite lower-tier entries in the registry map:

1. **Tier 4: Global Managed Baseline (Lowest Priority)**
   - Paths: `~/.ce-ai/skills/`, `~/.config/opencode/compound-engineering/skills/`, `~/.config/opencode/skills/`
   - Purpose: Central default catalog installed and managed by `ce-ai`.
2. **Tier 3: Global User Harness Roots**
   - Paths: `~/.config/<harness>/skills/` or `~/.ce-ai/harness-<kind>/skills/` for each harness kind (e.g., `claude`, `cursor`, `pi`, `windsurf`).
   - Purpose: User-defined global skill overrides specific to individual agent harnesses.
3. **Tier 2: Workspace Harness Roots**
   - Path: `<cwd>/.opencode/skills/`
   - Purpose: Project-scoped OpenCode skill overrides for repository-specific workflows.
4. **Tier 1: Workspace Central Roots (Highest Priority)**
   - Path: `<cwd>/.ce-ai/skills/`
   - Purpose: Project-scoped `ce-ai` skill overrides that take absolute precedence over all other global and harness-specific definitions.

### R3 Security Boundary Canonicalization

To protect host environments against malicious skill packages containing path traversal sequences or deceptive symlinks, `SkillRegistry::build` enforces R3 security canonicalization:

1. **Root Boundary Collection**: `collect_authorized_roots` compiles a list of allowed directories (`ctx.config_dir`, workspace root `<cwd>`, `.ce-ai`, `.opencode`, and user home configuration paths).
2. **Path Canonicalization**: `candidate.canonicalize()` resolves all symbolic links, relative references (`.`, `..`), and path aliases into absolute physical paths.
3. **Boundary Verification**: `canonicalize_and_validate_path` verifies that `canonical_candidate.starts_with(&canonical_root)` for at least one authorized root.
4. **Rejection Handling**: If a symlink targets a file outside authorized directories (e.g., `/etc/passwd` or `~/.ssh/id_rsa`), or if path traversal breaks out of the root, the path is rejected with a `CeError::Runtime` security error.

## 4. Lifecycle Hooks & Dry-Run Invariants

The Skill Registry Engine is fully integrated into `ce-ai`'s command lifecycle, ensuring the central index remains consistent with filesystem state without violating dry-run invariants:

- **`install` / `sync` / `upgrade`**:
  - After executing core plugin updates or harness synchronizations, `SkillRegistry::build(ctx)` scans active skill locations.
  - If `!ctx.dry_run`, `registry.save()` writes the updated catalog to `~/.ce-ai/skills-registry.json` using atomic temporary file substitution (`write_atomic`) and sets POSIX `0644` permissions.
  - When `--dry-run` is active, catalog persistence is skipped completely.
- **`init-prj` (Project Adoption)**:
  - Scans workspace skill directories and updates `skills-registry.json`.
  - Appends a sentinel-bounded block to the project's `.gitignore` to prevent committing local registry indices:
    ```gitignore
    # BEGIN CE-AI MANAGED BLOCK
    .ce-ai/skills-registry.json
    # END CE-AI MANAGED BLOCK
    ```
  - All modifications respect `if !ctx.dry_run`.
- **`deinit-prj` (Project De-Adoption)**:
  - Locates `# BEGIN CE-AI MANAGED BLOCK` and `# END CE-AI MANAGED BLOCK` within `.gitignore` and removes the managed section cleanly.
  - If `.gitignore` becomes empty after removal, the file is automatically cleaned up.
- **`uninstall` (Uninstallation Parity)**:
  - Removes `~/.ce-ai/skills-registry.json` when `!ctx.dry_run`.
  - Scans `~/.ce-ai/` for lingering atomic temp files (`.skills-registry.json.tmp.*`) and removes them to guarantee zero residual file clutter.
- **`doctor` (Integrity Diagnostics)**:
  - Exposes `check_skill_registry_health(ctx)` probe shared between `ce-ai doctor` and `ce-ai skills doctor`.
  - Verifies registry file presence, parses index structure, confirms underlying skill file existence, and detects SHA256 digest drift.

## 5. Code Examples

### 5.1 Skill Entry and Registry Data Structures (`src/source/registry.rs`)

```rust
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::error::CeError;

/// Indexed skill entry with metadata, SHA256 hash, and harness path mappings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub scope: String,
    pub triggers: Vec<String>,
    pub sha256: String,
    /// Mapping of harness kind (e.g. "opencode", "claude") to absolute path.
    pub harness_paths: BTreeMap<String, String>,
}

/// Central skill registry index schema (`~/.ce-ai/skills-registry.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillRegistry {
    pub version: String,
    pub updated_at: String,
    pub skills: Vec<SkillEntry>,
}
```

### 5.2 Atomic Persistence with POSIX Permissions (`src/source/registry.rs`)

```rust
impl SkillRegistry {
    /// Saves `SkillRegistry` atomically using `write_atomic` and sets POSIX `0644` mode bits.
    pub fn save(&self, path: &Path) -> Result<(), CeError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| CeError::Runtime(format!("failed to serialize skill registry: {}", e)))?;
        crate::state::write_atomic(path, content.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o644));
        }

        Ok(())
    }
}
```

### 5.3 Path Canonicalization & Security Boundary Checks (`src/source/registry.rs`)

```rust
/// Canonicalizes candidate path and root paths, enforcing root boundary constraints (R3).
pub fn canonicalize_and_validate_path(
    candidate: &Path,
    roots: &[PathBuf],
) -> Result<PathBuf, CeError> {
    let canonical_candidate = candidate.canonicalize().map_err(|e| {
        CeError::Runtime(format!(
            "failed to canonicalize path '{}': {}",
            candidate.display(),
            e
        ))
    })?;

    for root in roots {
        if let Ok(canonical_root) = root.canonicalize() {
            if canonical_candidate.starts_with(&canonical_root) {
                return Ok(canonical_candidate);
            }
        }
    }

    Err(CeError::Runtime(format!(
        "security rejection: path '{}' escapes authorized skill root boundaries",
        candidate.display()
    )))
}
```

### 5.4 Frontmatter Parser with YAML Bullet Support (`src/source/registry.rs`)

```rust
/// Parses YAML frontmatter `---\n...\n---` headers from `SKILL.md`.
pub fn parse_skill_frontmatter(content: &str) -> (String, String, Vec<String>, String) {
    let mut name = String::new();
    let mut description = String::new();
    let mut triggers: Vec<String> = Vec::new();
    let mut scope = String::new();

    if !content.starts_with("---") {
        return (name, description, triggers, scope);
    }

    let rest = &content[3..];
    let end_idx = match rest.find("\n---") {
        Some(idx) => idx,
        None => return (name, description, triggers, scope),
    };

    let header_lines = &rest[..end_idx];
    let mut current_key = String::new();

    for line in header_lines.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('-') && (current_key == "triggers" || current_key == "triggers:") {
            let item = trimmed
                .trim_start_matches('-')
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if !item.is_empty() {
                triggers.push(item);
            }
            continue;
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim().to_lowercase();
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            current_key = key.clone();

            match key.as_str() {
                "name" => name = val,
                "description" => description = val,
                "scope" => scope = val,
                "triggers" if !val.is_empty() => {
                    triggers.extend(
                        val.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty()),
                    );
                }
                _ => {}
            }
        }
    }

    (name, description, triggers, scope)
}
```

### 5.5 Dual-Format Resolution & Integrity Check (`src/source/registry.rs`)

```rust
impl SkillRegistry {
    /// Resolves a skill query for a specific harness in dual format.
    pub fn resolve(&self, harness: HarnessKind, query: &str) -> (String, Vec<SkillEntry>, String) {
        let query_lower = query.to_lowercase();
        let mut matched: Vec<SkillEntry> = Vec::new();
        let mut has_degradation = false;

        for entry in &self.skills {
            let name_match = entry.name.to_lowercase().contains(&query_lower);
            let desc_match = entry.description.to_lowercase().contains(&query_lower);
            let trigger_match = entry
                .triggers
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower));

            if name_match || desc_match || trigger_match {
                // Verify file existence & SHA256 integrity at resolution time
                if let Some(raw_path) = entry.harness_paths.get(harness.as_str()) {
                    let path = PathBuf::from(raw_path);
                    if path.exists() {
                        if let Ok(current_sha) = compute_file_sha256(&path) {
                            if current_sha == entry.sha256 {
                                matched.push(entry.clone());
                                continue;
                            }
                        }
                    }
                }
                has_degradation = true;
            }
        }

        let status_tag = if matched.is_empty() {
            "none".to_string()
        } else if has_degradation {
            "fallback-fuzzy".to_string()
        } else {
            "paths-injected".to_string()
        };

        let now_iso = chrono::Utc::now().to_rfc3339();

        let mut markdown = String::new();
        markdown.push_str(&format!(
            "<!-- ce-ai:skill_resolution status={} timestamp={} -->\n",
            status_tag, now_iso
        ));
        markdown.push_str("## Skills to load before work:\n");

        for m in &matched {
            let path_str = m
                .harness_paths
                .get(harness.as_str())
                .cloned()
                .unwrap_or_default();
            markdown.push_str(&format!(
                "- **{}**: {}\n  Path: `file://{}`\n",
                m.name, m.description, path_str
            ));
        }

        (status_tag, matched, markdown)
    }
}
```

### 5.6 CLI Command Dispatcher (`src/commands/skills.rs`)

```rust
pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let registry_path = ctx.config_dir.join("skills-registry.json");
    let registry = SkillRegistry::load(&registry_path)?;

    match &args.action {
        Action::List { harness, json } => {
            // Filter and render catalog in tabular or JSON format
        }
        Action::Resolve { harness, query, json } => {
            let harness_kind = harness.parse::<HarnessKind>()?;
            let (status, skills, markdown) = registry.resolve(harness_kind, query);

            if status == "fallback-fuzzy" {
                eprintln!(
                    "⚠️ Warning: Skill resolution degraded to fallback-fuzzy for query '{}'",
                    query
                );
            }

            if *json {
                let output = serde_json::json!({
                    "resolution_status": status,
                    "query": query,
                    "harness": harness_kind.as_str(),
                    "skills": skills,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                print!("{}", markdown);
            }
        }
        Action::Doctor => {
            let findings = crate::source::registry::check_skill_registry_health(ctx)?;
            if !findings.is_empty() {
                return Err(CeError::Runtime("skill registry integrity check failed".into()));
            }
        }
    }
    Ok(())
}
```
