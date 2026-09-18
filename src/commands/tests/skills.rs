use super::*;
use sha2::Digest;
use std::fs;
use tempfile::TempDir;

fn hermetic_ctx(tmp: &TempDir) -> Context {
    Context {
        config_dir: tmp.path().join("ce-ai"),
        opencode_config_dir: tmp.path().join("home/.config/opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    }
}

fn setup_test_registry(ctx: &Context) {
    fs::create_dir_all(&ctx.config_dir).unwrap();
    let skill_file = ctx.config_dir.join("test_SKILL.md");
    fs::write(&skill_file, "security skill content").unwrap();

    let mut hasher = sha2::Sha256::new();
    sha2::Digest::update(&mut hasher, b"security skill content");
    let sha = format!("{:x}", sha2::Digest::finalize(hasher));

    let entry = crate::source::registry::SkillEntry {
        name: "security-audit".into(),
        description: "Audit security tokens and cookies".into(),
        scope: "global".into(),
        triggers: vec!["security".into(), "tokens".into()],
        categories: vec!["security".into()],
        sha256: sha,
        harness_paths: std::collections::BTreeMap::from([(
            "opencode".into(),
            skill_file.to_string_lossy().into(),
        )]),
    };

    let registry = SkillRegistry {
        skills: vec![entry],
        ..Default::default()
    };
    registry
        .save(&ctx.config_dir.join("skills-registry.json"))
        .unwrap();
}

#[test]
fn test_skills_resolve_missing_query_returns_usage_error() {
    let tmp = TempDir::new().unwrap();
    let ctx = hermetic_ctx(&tmp);
    setup_test_registry(&ctx);

    let args = Args {
        action: Action::Resolve {
            harness: "opencode".into(),
            query_pos: None,
            query: None,
            json: false,
            verbose: false,
        },
    };

    let res = run(&ctx, &args);
    assert!(matches!(res, Err(CeError::Usage(_))));
}

#[test]
fn test_skills_resolve_positional_and_flag_query() {
    let tmp = TempDir::new().unwrap();
    let ctx = hermetic_ctx(&tmp);
    setup_test_registry(&ctx);

    // Positional query
    let args_pos = Args {
        action: Action::Resolve {
            harness: "opencode".into(),
            query_pos: Some("tokens".into()),
            query: None,
            json: false,
            verbose: false,
        },
    };
    assert!(run(&ctx, &args_pos).is_ok());

    // Flag query
    let args_flag = Args {
        action: Action::Resolve {
            harness: "opencode".into(),
            query_pos: None,
            query: Some("security".into()),
            json: false,
            verbose: false,
        },
    };
    assert!(run(&ctx, &args_flag).is_ok());
}

#[test]
fn test_skills_resolve_json_and_verbose() {
    let tmp = TempDir::new().unwrap();
    let ctx = hermetic_ctx(&tmp);
    setup_test_registry(&ctx);

    let args_json = Args {
        action: Action::Resolve {
            harness: "opencode".into(),
            query_pos: Some("tokens".into()),
            query: None,
            json: true,
            verbose: true,
        },
    };
    assert!(run(&ctx, &args_json).is_ok());
}

#[test]
fn test_skills_resolve_with_routing_configuration() {
    let tmp = TempDir::new().unwrap();
    let ctx = hermetic_ctx(&tmp);
    setup_test_registry(&ctx);

    let mut state = crate::state::state::State::default();
    let mut decisions = crate::state::state::DecisionsConfig::default();
    decisions.skills.enabled = true;
    decisions.skills.minimum_confidence_pct = 70;
    state.decisions = Some(decisions);
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    let args = Args {
        action: Action::Resolve {
            harness: "opencode".into(),
            query_pos: Some("Audit security and tokens".into()),
            query: None,
            json: true,
            verbose: true,
        },
    };
    assert!(run(&ctx, &args).is_ok());
}
