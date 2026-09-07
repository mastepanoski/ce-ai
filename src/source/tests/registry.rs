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

#[test]
fn test_sequential_thinking_frontmatter_parsing_and_compatibility() {
    use crate::source::builtin_skills::BUILTIN_SEQUENTIAL_THINKING_SKILL;

    let fm = parse_skill_frontmatter(BUILTIN_SEQUENTIAL_THINKING_SKILL);
    assert_eq!(fm.name, "sequential-thinking");
    assert_eq!(fm.scope, "global");
    assert!(!fm.description.is_empty());
    assert!(fm.triggers.contains(&"complex reasoning".to_string()));
    assert!(fm.triggers.contains(&"sequential thought".to_string()));
    assert!(fm.triggers.contains(&"hypothesis testing".to_string()));
}

#[test]
fn test_sequential_thinking_indexing_resolution_and_degradation() {
    use crate::source::builtin_skills::BUILTIN_SEQUENTIAL_THINKING_SKILL;

    let temp = tempfile::tempdir().unwrap();
    let ce_ai_dir = temp.path().join(".ce-ai");
    let skills_dir = ce_ai_dir.join("skills");
    let seq_dir = skills_dir.join("sequential-thinking");
    fs::create_dir_all(&seq_dir).unwrap();

    let skill_path = seq_dir.join("SKILL.md");
    fs::write(&skill_path, BUILTIN_SEQUENTIAL_THINKING_SKILL).unwrap();

    let ctx = Context {
        config_dir: ce_ai_dir.clone(),
        opencode_config_dir: temp.path().join(".config/opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let registry = SkillRegistry::build(&ctx).unwrap();
    let entry = registry
        .skills
        .iter()
        .find(|s| s.name == "sequential-thinking")
        .expect("sequential-thinking must be indexed in registry");

    assert_eq!(
        entry.sha256,
        compute_file_sha256(&skill_path).unwrap(),
        "indexed sha256 must match file sha256"
    );
    assert!(
        entry.harness_paths.contains_key("pi"),
        "entry must map to pi harness"
    );
    assert!(
        entry.harness_paths.contains_key("opencode"),
        "entry must map to opencode harness"
    );

    // Initial resolution with valid SHA256
    let (status, matched, md) = registry.resolve(HarnessKind::Pi, "sequential-thinking");
    assert_eq!(status, "paths-injected");
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].name, "sequential-thinking");
    assert!(md.contains("status=paths-injected"));
    let canonical_skill_path = skill_path.canonicalize().unwrap();
    assert!(md.contains(&format!("Path: `file://{}", canonical_skill_path.display())));

    // Check is_skill_configured auto-detection
    assert!(
        !crate::source::tools_registry::is_skill_configured(&ctx, "sequential-thinking"),
        "before saving registry, is_skill_configured must be false"
    );
    registry
        .save(&ce_ai_dir.join("skills-registry.json"))
        .unwrap();
    assert!(
        crate::source::tools_registry::is_skill_configured(&ctx, "sequential-thinking"),
        "after saving registry, is_skill_configured must be true"
    );

    // Tamper with file to verify degradation
    fs::write(&skill_path, "tampered content without frontmatter").unwrap();
    let (deg_status, _, deg_md) = registry.resolve(HarnessKind::Pi, "sequential-thinking");
    assert_eq!(deg_status, "fallback-fuzzy");
    assert!(deg_md.contains("status=fallback-fuzzy"));
}
