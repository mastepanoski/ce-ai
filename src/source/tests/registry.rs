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
