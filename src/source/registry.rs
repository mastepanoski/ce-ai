//! Multi-Harness Skill Registry Engine (`ce-ai skills`).
//!
//! Indexes, parses, validates, and resolves `SKILL.md` instruction files
//! across 12 AI coding agent harnesses with SHA256 integrity and boundary security.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;

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

impl Default for SkillRegistry {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            skills: Vec::new(),
        }
    }
}

/// Parsed YAML frontmatter from `SKILL.md`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub scope: String,
}

impl SkillRegistry {
    /// Loads `SkillRegistry` from disk, returning default if file is missing.
    pub fn load(path: &Path) -> Result<Self, CeError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let registry: Self = serde_json::from_str(&content).map_err(|e| {
            CeError::Runtime(format!(
                "failed to parse skill registry at '{}': {}",
                path.display(),
                e
            ))
        })?;
        Ok(registry)
    }

    /// Saves `SkillRegistry` atomically using `write_atomic` and sets POSIX `0644` mode bits.
    pub fn save(&self, path: &Path) -> Result<(), CeError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| CeError::Runtime(format!("failed to serialize skill registry: {}", e)))?;
        crate::state::write_atomic(path, content.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o644)).map_err(|e| {
                CeError::Runtime(format!(
                    "failed to set permissions on '{}': {}",
                    path.display(),
                    e
                ))
            })?;
        }

        Ok(())
    }

    /// Helper encapsulating index build and save lifecycle integration (MAINT-003).
    pub fn sync_registry(ctx: &Context) -> Result<(), CeError> {
        if !ctx.dry_run {
            let registry = Self::build(ctx)?;
            let _ = registry.save(&ctx.config_dir.join("skills-registry.json"));
        }
        Ok(())
    }

    /// Helper encapsulating registry index and residual temporary file removal (MAINT-004).
    pub fn remove(ctx: &Context) -> Result<(), CeError> {
        if !ctx.dry_run {
            let registry_path = ctx.config_dir.join("skills-registry.json");
            if registry_path.exists() {
                let _ = fs::remove_file(&registry_path);
            }
            if let Ok(entries) = fs::read_dir(&ctx.config_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(".skills-registry.json.tmp") {
                        if let Ok(meta) = fs::symlink_metadata(entry.path()) {
                            if meta.is_file() && !meta.file_type().is_symlink() {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Scans global and workspace skill roots across all harnesses to build the registry.
    pub fn build(ctx: &Context) -> Result<Self, CeError> {
        let mut registry = Self::default();
        let mut skill_map: BTreeMap<String, SkillEntry> = BTreeMap::new();

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Collect authorized root boundaries (R3 security canonicalization)
        let authorized_roots = collect_authorized_roots(ctx, &cwd);

        // Precedence Tier 4 (Lowest): Global Managed (~/.ce-ai/skills/ and managed opencode skills)
        let global_managed = ctx.config_dir.join("skills");
        scan_skill_directory(
            &global_managed,
            "global",
            None,
            &authorized_roots,
            &mut skill_map,
        )?;

        let opencode_managed = ctx
            .opencode_config_dir
            .join("compound-engineering")
            .join("skills");
        scan_skill_directory(
            &opencode_managed,
            "global",
            None,
            &authorized_roots,
            &mut skill_map,
        )?;

        let opencode_skills = ctx.opencode_config_dir.join("skills");
        scan_skill_directory(
            &opencode_skills,
            "global",
            None,
            &authorized_roots,
            &mut skill_map,
        )?;

        // Precedence Tier 3: Global User Harness Roots (~/.config/<harness>/skills/)
        for harness in HarnessKind::all() {
            let harness_dir = ctx.config_dir.join(format!("harness-{}", harness.as_str()));
            let harness_skills = harness_dir.join("skills");
            scan_skill_directory(
                &harness_skills,
                "global",
                Some(harness),
                &authorized_roots,
                &mut skill_map,
            )?;
        }

        // Precedence Tier 2: Workspace (.opencode/skills/)
        let ws_opencode = cwd.join(".opencode").join("skills");
        scan_skill_directory(
            &ws_opencode,
            "project",
            None,
            &authorized_roots,
            &mut skill_map,
        )?;

        // Precedence Tier 1 (Highest): Workspace (.ce-ai/skills/)
        let ws_ce_ai = cwd.join(".ce-ai").join("skills");
        scan_skill_directory(
            &ws_ce_ai,
            "project",
            None,
            &authorized_roots,
            &mut skill_map,
        )?;

        registry.skills = skill_map.into_values().collect();
        Ok(registry)
    }

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
            if has_degradation {
                "fallback-fuzzy".to_string()
            } else {
                "none".to_string()
            }
        } else if has_degradation {
            "fallback-fuzzy".to_string()
        } else {
            "paths-injected".to_string()
        };

        // No wall-clock data here: identical registry state must produce
        // byte-identical output so harness-facing prompts stay reproducible.
        let mut markdown = String::new();
        markdown.push_str(&format!(
            "<!-- ce-ai:skill_resolution status={} -->\n",
            status_tag
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

/// Collects all authorized root directories for security boundary canonicalization (R3).
pub fn collect_authorized_roots(ctx: &Context, cwd: &Path) -> Vec<PathBuf> {
    let mut roots = vec![
        ctx.config_dir.clone(),
        ctx.config_dir.join("skills"),
        ctx.opencode_config_dir.clone(),
        cwd.to_path_buf(),
        cwd.join(".ce-ai"),
        cwd.join(".opencode"),
    ];

    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let home_path = PathBuf::from(home);
        roots.push(home_path.join(".ce-ai"));
        roots.push(home_path.join(".config").join("opencode"));
        for harness in HarnessKind::all() {
            roots.push(home_path.join(format!(".{}", harness.as_str())));
            roots.push(home_path.join(".config").join(harness.as_str()));
        }
    }

    roots
}

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

/// Scans a directory for `SKILL.md` files or skill subdirectories.
fn scan_skill_directory(
    dir: &Path,
    default_scope: &str,
    target_harness: Option<HarnessKind>,
    roots: &[PathBuf],
    skill_map: &mut BTreeMap<String, SkillEntry>,
) -> Result<(), CeError> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                process_skill_file(&skill_md, default_scope, target_harness, roots, skill_map)?;
            }
        } else if path.is_file() && path.file_name() == Some(std::ffi::OsStr::new("SKILL.md")) {
            process_skill_file(&path, default_scope, target_harness, roots, skill_map)?;
        }
    }

    Ok(())
}

/// Processes an individual `SKILL.md` file, parsing frontmatter and updating entry.
fn process_skill_file(
    skill_path: &Path,
    default_scope: &str,
    target_harness: Option<HarnessKind>,
    roots: &[PathBuf],
    skill_map: &mut BTreeMap<String, SkillEntry>,
) -> Result<(), CeError> {
    // Validate path security (R3)
    let valid_path = match canonicalize_and_validate_path(skill_path, roots) {
        Ok(p) => p,
        Err(_) => return Ok(()),
    };

    let content = match fs::read_to_string(&valid_path) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    let sha256 = match compute_file_sha256(&valid_path) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };

    let fm = parse_skill_frontmatter(&content);
    let name = if fm.name.is_empty() {
        skill_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    } else {
        fm.name
    };

    let scope = if !fm.scope.is_empty() {
        fm.scope
    } else {
        default_scope.to_string()
    };

    let path_str = valid_path.to_string_lossy().to_string();

    let entry = skill_map.entry(name.clone()).or_insert_with(|| SkillEntry {
        name: name.clone(),
        description: fm.description.clone(),
        scope: scope.clone(),
        triggers: fm.triggers.clone(),
        sha256: sha256.clone(),
        harness_paths: BTreeMap::new(),
    });

    entry.scope = scope;
    if !fm.description.is_empty() {
        entry.description = fm.description;
    }
    if !fm.triggers.is_empty() {
        entry.triggers = fm.triggers;
    }
    entry.sha256 = sha256;

    if let Some(target) = target_harness {
        entry
            .harness_paths
            .insert(target.as_str().to_string(), path_str);
    } else {
        for harness in HarnessKind::all() {
            entry
                .harness_paths
                .insert(harness.as_str().to_string(), path_str.clone());
        }
    }

    Ok(())
}

/// Computes SHA256 digest of a file.
pub fn compute_file_sha256(path: &Path) -> Result<String, CeError> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Parses YAML frontmatter `---\n...\n---` headers from `SKILL.md` (MAINT-005).
pub fn parse_skill_frontmatter(content: &str) -> SkillFrontmatter {
    let mut fm = SkillFrontmatter::default();

    if !content.starts_with("---") {
        return fm;
    }

    let rest = &content[3..];
    let end_idx = match rest.find("\n---") {
        Some(idx) => idx,
        None => return fm,
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
                fm.triggers.push(item);
            }
            continue;
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim().to_lowercase();
            let val = v.trim().trim_matches('"').trim_matches('\'');
            current_key = key.clone();

            match key.as_str() {
                "name" => fm.name = val.to_string(),
                "description" => fm.description = val.to_string(),
                "scope" => fm.scope = val.to_string(),
                "triggers" if !val.is_empty() => {
                    let clean_val = val.trim_start_matches('[').trim_end_matches(']');
                    fm.triggers.extend(
                        clean_val
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                            .filter(|s| !s.is_empty()),
                    );
                }
                _ => {}
            }
        }
    }

    fm
}

/// Shared diagnostic probe helper function for `ce-ai doctor` and `ce-ai skills doctor` (DEC-05).
pub fn check_skill_registry_health(ctx: &Context) -> Result<Vec<String>, CeError> {
    let mut findings: Vec<String> = Vec::new();
    let registry_path = ctx.config_dir.join("skills-registry.json");

    if !registry_path.exists() {
        findings.push(format!(
            "Skill registry missing at '{}' (run 'ce-ai sync' to generate)",
            registry_path.display()
        ));
        return Ok(findings);
    }

    let registry = match SkillRegistry::load(&registry_path) {
        Ok(r) => r,
        Err(e) => {
            findings.push(format!(
                "Corrupted skill registry at '{}': {}",
                registry_path.display(),
                e
            ));
            return Ok(findings);
        }
    };

    // Deduplicate file path health checks across harnesses (SKILL-REG-05)
    for skill in &registry.skills {
        let mut checked_paths: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();

        for path_str in skill.harness_paths.values() {
            if checked_paths.insert(path_str.clone()) {
                let path = PathBuf::from(path_str);
                if !path.exists() {
                    findings.push(format!(
                        "Skill '{}' file missing at '{}'",
                        skill.name, path_str
                    ));
                } else if let Ok(current_sha) = compute_file_sha256(&path) {
                    if current_sha != skill.sha256 {
                        findings.push(format!(
                            "Skill '{}' SHA256 digest drift at '{}' (expected {}, found {})",
                            skill.name, path_str, skill.sha256, current_sha
                        ));
                    }
                }
            }
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_markdown_is_byte_stable() {
        let entry = SkillEntry {
            name: "ce-brainstorm".to_string(),
            description: "Explore requirements".to_string(),
            scope: "global".to_string(),
            triggers: vec!["brainstorm".to_string()],
            sha256: "deadbeef".to_string(),
            harness_paths: BTreeMap::from([(
                "opencode".to_string(),
                "/tmp/nonexistent-ce-brainstorm/SKILL.md".to_string(),
            )]),
        };
        let registry = SkillRegistry {
            skills: vec![entry],
            ..SkillRegistry::default()
        };

        let (status_a, _, md_a) = registry.resolve(HarnessKind::Opencode, "brainstorm");
        let (status_b, _, md_b) = registry.resolve(HarnessKind::Opencode, "brainstorm");
        assert_eq!(md_a, md_b);
        assert_eq!(status_a, status_b);
        assert!(md_a.contains("<!-- ce-ai:skill_resolution status="));
        assert!(!md_a.contains("timestamp="));
    }

    #[test]
    fn test_frontmatter_extraction_yaml_lists() {
        let content = r#"---
name: "ce-brainstorm"
description: "Explore vague or ambitious ideas"
scope: "project"
triggers:
  - "brainstorm"
  - "ideate"
---
# Skill Body
"#;
        let fm = parse_skill_frontmatter(content);
        assert_eq!(fm.name, "ce-brainstorm");
        assert_eq!(fm.description, "Explore vague or ambitious ideas");
        assert_eq!(fm.scope, "project");
        assert_eq!(fm.triggers, vec!["brainstorm", "ideate"]);
    }

    #[test]
    fn test_path_traversal_rejection() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("allowed_root");
        fs::create_dir_all(&root).unwrap();

        let normal_file = root.join("SKILL.md");
        fs::write(&normal_file, "---").unwrap();

        let roots = vec![root.clone()];
        assert!(canonicalize_and_validate_path(&normal_file, &roots).is_ok());

        let outside_dir = temp.path().join("outside");
        fs::create_dir_all(&outside_dir).unwrap();
        let outside_file = outside_dir.join("SKILL.md");
        fs::write(&outside_file, "---").unwrap();

        assert!(canonicalize_and_validate_path(&outside_file, &roots).is_err());
    }

    #[test]
    fn test_symlink_escape_rejection() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("allowed_root");
        let outside = temp.path().join("outside_target");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let target_file = outside.join("secret.txt");
        fs::write(&target_file, "secret").unwrap();

        #[cfg(unix)]
        {
            let symlink_file = root.join("symlink_SKILL.md");
            std::os::unix::fs::symlink(&target_file, &symlink_file).unwrap();

            let roots = vec![root.clone()];
            assert!(canonicalize_and_validate_path(&symlink_file, &roots).is_err());
        }
    }

    #[test]
    fn test_registry_4_tier_precedence_override() {
        let temp = tempfile::tempdir().unwrap();
        let global_skills = temp.path().join(".ce-ai/skills/shared-skill");
        let workspace_skills = temp.path().join(".ce-ai/skills/shared-skill");
        fs::create_dir_all(&global_skills).unwrap();
        fs::create_dir_all(&workspace_skills).unwrap();

        fs::write(
            global_skills.join("SKILL.md"),
            "---\nname: shared-skill\ndescription: Global skill\n---\n",
        )
        .unwrap();

        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: true,
        };

        let registry = SkillRegistry::build(&ctx).unwrap();
        assert!(!registry.skills.is_empty());
        let found = registry.skills.iter().find(|s| s.name == "shared-skill");
        assert!(found.is_some());
    }

    #[test]
    fn test_registry_atomic_save_and_load() {
        let temp = tempfile::tempdir().unwrap();
        let registry_path = temp.path().join("skills-registry.json");

        let mut registry = SkillRegistry::default();
        registry.skills.push(SkillEntry {
            name: "test-skill".into(),
            description: "A test skill".into(),
            scope: "global".into(),
            triggers: vec!["test".into()],
            sha256: "abc123sha".into(),
            harness_paths: BTreeMap::from([("opencode".into(), "/path/to/SKILL.md".into())]),
        });

        registry.save(&registry_path).unwrap();
        assert!(registry_path.exists());

        let loaded = SkillRegistry::load(&registry_path).unwrap();
        assert_eq!(loaded.skills.len(), 1);
        assert_eq!(loaded.skills[0].name, "test-skill");
    }
}
