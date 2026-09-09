use std::collections::BTreeMap;

use tempfile::tempdir;

use super::{
    failed_detail_lines, guidance_note_lines, matrix_line, reconciliation_line, sync_skills_root,
    verify_tree_against, CheckStatus, TreeDrift,
};
use crate::harness::registration::registration_spec;
use crate::harness::HarnessKind;
use crate::state::diff::sha256_hex;

#[test]
fn registration_specs_cover_the_table_driven_kinds() {
    use HarnessKind::*;
    for kind in [Claude, Codex, Copilot, Grok, Kimi, Agy, Fx] {
        let spec = registration_spec(kind).expect("table-driven kind");
        assert!(spec.register_mcp.is_some());
    }

    // Pi: skills tree only — No-MCP by design (objective 8).
    let pi = registration_spec(Pi).expect("pi spec");
    assert!(pi.register_mcp.is_none());

    // Cursor consumes MCP servers only; copying a skills tree into its
    // directory would pollute user storage (regression pin). Skills-root
    // conventions live in sync_skills_root, not in this table.
    let cursor = registration_spec(Cursor).expect("cursor spec");
    assert!(cursor.register_mcp.is_some());

    for kind in [Opencode, Custom, Deepseek] {
        assert!(registration_spec(kind).is_none(), "dedicated arm kind");
    }
}

#[test]
fn sync_skills_root_nests_agy_under_config() {
    let home = tempdir().unwrap();
    let dir = home.path();
    assert_eq!(
        sync_skills_root(HarnessKind::Agy, dir),
        dir.join(".gemini").join("config").join("skills")
    );
    assert_eq!(
        sync_skills_root(HarnessKind::Pi, dir),
        dir.join(".pi").join("agent").join("skills")
    );
}

fn expected_map(files: &[(&str, &[u8])]) -> BTreeMap<String, String> {
    files
        .iter()
        .map(|(name, bytes)| ((*name).to_string(), sha256_hex(bytes)))
        .collect()
}

#[test]
fn clean_tree_reports_no_drift() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), b"alpha").unwrap();
    let expected = expected_map(&[("a.txt", b"alpha")]);

    let drift = verify_tree_against(dir.path(), &expected);
    assert_eq!(drift, TreeDrift::default());
    assert_eq!(
        CheckStatus::from_drift(1, drift),
        CheckStatus::Verified {
            matched: 1,
            total: 1
        }
    );
}

#[test]
fn hash_mismatch_is_detected_per_file() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), b"tampered").unwrap();
    std::fs::write(dir.path().join("b.txt"), b"beta").unwrap();
    let expected = expected_map(&[("a.txt", b"alpha"), ("b.txt", b"beta")]);

    let drift = verify_tree_against(dir.path(), &expected);
    assert_eq!(drift.mismatched, vec!["a.txt".to_string()]);
    assert!(drift.missing.is_empty());
}

#[test]
fn missing_files_are_reported_separately() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("b.txt"), b"beta").unwrap();
    let expected = expected_map(&[("a.txt", b"alpha"), ("b.txt", b"beta")]);

    let drift = verify_tree_against(dir.path(), &expected);
    assert!(drift.mismatched.is_empty());
    assert_eq!(drift.missing, vec!["a.txt".to_string()]);

    let status = CheckStatus::from_drift(2, drift);
    match status {
        CheckStatus::Failed {
            mismatched,
            missing,
        } => {
            assert!(mismatched.is_empty());
            assert_eq!(missing.len(), 1);
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[test]
fn nested_paths_are_hashed_relative_to_root() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("skills/ce-work")).unwrap();
    std::fs::write(dir.path().join("skills/ce-work/SKILL.md"), b"# skill").unwrap();
    let expected = expected_map(&[("ce-work/SKILL.md", b"# skill")]);

    // The harness skills root maps `skills/<rest>` onto `<root>/<rest>`.
    let skills_root = dir.path().join("skills");
    let drift = verify_tree_against(&skills_root, &expected);
    assert_eq!(drift, TreeDrift::default());
}

#[test]
fn matrix_line_pins_registered_wording() {
    let line = matrix_line(
        "claude",
        &CheckStatus::NotVerified {
            reason: super::REASON_NO_MANAGED_SKILLS,
        },
    );
    assert_eq!(
        line,
        "  ○ claude: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)"
    );

    let cursor = matrix_line(
        "cursor",
        &CheckStatus::NotVerified {
            reason: super::REASON_CONFIG_ONLY,
        },
    );
    assert_eq!(
        cursor,
        "  ○ cursor: registered — config registration only — no managed assets to hash-verify"
    );
}

#[test]
fn matrix_line_pins_verified_and_failed_wording() {
    assert_eq!(
        matrix_line(
            "opencode",
            &CheckStatus::Verified {
                matched: 1,
                total: 1
            }
        ),
        "  ✓ opencode: verified — 1/1 managed files match SHA256"
    );

    let failed = matrix_line(
        "opencode",
        &CheckStatus::Failed {
            mismatched: vec!["plugins/x.js".to_string()],
            missing: vec![],
        },
    );
    assert_eq!(failed, "  ✗ opencode: FAILED — 1 file(s) drifted");
    assert_eq!(
        failed_detail_lines(&["plugins/x.js".to_string()], &[]),
        vec!["      plugins/x.js".to_string()]
    );
}

#[test]
fn reconciliation_line_uses_registered_not_unverified() {
    let line = reconciliation_line(1, 9, 0);
    assert_eq!(
        line,
        "reconciliation status: 1 verified, 9 registered (nothing to verify), 0 failed"
    );
    assert!(!line.contains("unverified"));
}

#[test]
fn guidance_note_explains_adoption_and_scope() {
    let lines = guidance_note_lines();
    let joined = lines.join("\n");
    assert!(joined.contains("install --harness"));
    assert!(joined.contains("outside ce-ai's verification scope"));
    assert!(joined.contains("managed skills tree"));
    assert!(
        lines
            .iter()
            .all(|l| l.starts_with("note:") || l.starts_with("      ")),
        "note block must be visually grouped"
    );
}

#[test]
fn sync_state_store_port_evaluates_adoption_status_in_memory() {
    use crate::state::state::{SkillSurface, State};
    use crate::state::{InMemoryStateStore, StateStore};
    use std::path::{Path, PathBuf};

    let store = InMemoryStateStore::new();
    let path = Path::new("/virtual/ce-ai/state.json");

    let mut state = State::new();
    state.skill_surfaces.push(SkillSurface {
        harness: "opencode".into(),
        root: PathBuf::from("/virtual/opencode/skills"),
        status: "adopted".into(),
        files: vec![],
        adopted_at: Some("2026-08-27T00:00:00Z".into()),
    });
    store.save(path, &state).unwrap();

    let loaded = store.load(path).unwrap();
    let is_adopted = loaded
        .skill_surfaces
        .iter()
        .any(|s| s.harness == "opencode" && s.status == "adopted");
    assert!(is_adopted);
}

#[test]
fn tree_drift_tracks_mismatches_and_missing_counts() {
    let mut drift = TreeDrift::default();
    assert_eq!(drift.total(), 0);

    drift.mismatched.push("skill_a/SKILL.md".to_string());
    drift.missing.push("skill_b/SKILL.md".to_string());
    drift.missing.push("skill_c/SKILL.md".to_string());

    assert_eq!(drift.total(), 3);
    assert_eq!(drift.mismatched.len(), 1);
    assert_eq!(drift.missing.len(), 2);

    let status = CheckStatus::from_drift(5, drift.clone());
    match status {
        CheckStatus::Failed {
            mismatched,
            missing,
        } => {
            assert_eq!(mismatched, vec!["skill_a/SKILL.md"]);
            assert_eq!(missing, vec!["skill_b/SKILL.md", "skill_c/SKILL.md"]);
        }
        _ => panic!("expected CheckStatus::Failed"),
    }
}

#[test]
fn resolve_sync_source_and_version_fails_fast_with_empty_state() {
    use crate::commands::sync::resolve_sync_source_and_version;
    use crate::commands::Context;
    use crate::state::state::State;

    let tmp = tempdir().unwrap();
    let ctx = Context {
        config_dir: tmp.path().join("config"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    let state = State::new();
    let home_dir = tmp.path().join("home");
    let opencode_dir = ctx.opencode_config_dir.clone();

    let res = resolve_sync_source_and_version(&ctx, &state, &home_dir, &opencode_dir);
    assert!(res.is_err());
    if let Err(crate::error::CeError::Runtime(msg)) = res {
        assert!(msg.contains("no harnesses installed — run ce-ai install first"));
    } else {
        panic!("expected CeError::Runtime with no harnesses installed message");
    }
}

/// Issue #325: `sync` (and therefore `upgrade`, which calls `sync_with`
/// internally) used to copy the OpenCode loader byte-for-byte from the
/// resolved source tree whenever it drifted, with no content validation.
/// A correctly-installed loader (with the SessionStart `session.created`
/// hook) would get silently regressed to a stale upstream release loader
/// that lacks the hook, because the raw source bytes differ from what is
/// already on disk. This pins that `sync_with` never regresses an
/// already-correct loader when the resolved source's own loader is stale.
#[test]
fn sync_with_does_not_regress_opencode_loader_when_source_lacks_session_hook() {
    use crate::commands::sync::sync_with;
    use crate::commands::Context;
    use crate::opencode::manifest::{InstallManifest, ManifestFile};
    use crate::opencode::plugins::LOADER_REL_PATH;

    let tmp = tempdir().unwrap();

    // Stale upstream release source: loader has neither the SessionStart
    // hook nor the legacy stub.
    let source_root = tmp.path().join("ce-source");
    let stale_loader = b"export default async function OldLoader() { return {} }";
    std::fs::create_dir_all(source_root.join(".opencode/plugins")).unwrap();
    std::fs::write(
        source_root.join(".opencode/plugins/compound-engineering.js"),
        stale_loader,
    )
    .unwrap();

    // Already-installed, correct loader (as `install_loader` would have
    // written it), tracked by a matching install-manifest entry.
    let opencode_dir = tmp.path().join("opencode-config");
    let correct_loader = b"export default function ceLoader() { /* session.created */ }";
    let loader_dest = opencode_dir.join("compound-engineering/plugins/compound-engineering.js");
    std::fs::create_dir_all(loader_dest.parent().unwrap()).unwrap();
    std::fs::write(&loader_dest, correct_loader).unwrap();

    InstallManifest {
        version: "v1.0.0".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: "2026-09-07T00:00:00Z".into(),
        source: serde_json::json!({"kind": "github-release", "tag": "v1.0.0"}),
        files: vec![ManifestFile {
            path: LOADER_REL_PATH.to_string(),
            sha256: sha256_hex(correct_loader),
        }],
        config_mutations: vec![],
    }
    .write(&opencode_dir)
    .unwrap();

    let ctx = Context {
        config_dir: tmp.path().join("config"),
        opencode_config_dir: opencode_dir.clone(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    sync_with(
        &ctx,
        &source_root,
        "v1.0.1",
        serde_json::json!({"kind": "local", "path": source_root.display().to_string()}),
    )
    .unwrap();

    let on_disk = std::fs::read_to_string(&loader_dest).unwrap();
    assert!(
        on_disk.contains("session.created") || on_disk.contains("ceLoader"),
        "sync regressed the correct SessionStart loader to the stale source loader: {on_disk}"
    );
}

#[test]
fn resolve_sync_source_and_version_resolves_from_non_opencode_entry() {
    use crate::commands::sync::resolve_sync_source_and_version;
    use crate::commands::Context;
    use crate::state::state::State;

    let tmp = tempdir().unwrap();
    let source_dir = tmp.path().join("source");
    std::fs::create_dir_all(&source_dir).unwrap();

    let ctx = Context {
        config_dir: tmp.path().join("config"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    let mut state = State::new();
    state.installed_harnesses.push(serde_json::json!({
        "name": "claude",
        "version": "1.6.3",
        "source": { "kind": "local", "path": source_dir.display().to_string() },
        "installed_at": "2026-08-22T00:00:00Z"
    }));

    let home_dir = tmp.path().join("home");
    let opencode_dir = ctx.opencode_config_dir.clone();

    let (root, ver, src) =
        resolve_sync_source_and_version(&ctx, &state, &home_dir, &opencode_dir).unwrap();
    assert_eq!(root, source_dir);
    assert_eq!(ver, "1.6.3");
    assert_eq!(src["kind"], "local");
}

/// harness-manifest-sha256-coverage: `sync` used to rewrite the install
/// manifest of every registration-spec harness (claude, kimi, ...) with
/// `files: vec![]`, permanently blinding drift detection outside OpenCode.
/// This pins that sync harvests real SHA256 entries from the on-disk managed
/// tree and preserves the prior manifest's `installed_at`.
#[test]
fn sync_with_harvests_real_hashes_for_registration_harness_manifests() {
    use crate::commands::sync::sync_with;
    use crate::commands::Context;
    use crate::opencode::manifest::InstallManifest;
    use crate::state::state::State;

    // Shield against parallel adapter tests that mutate KIMI_CODE_HOME:
    // hold the harness env lock and force the default (home-derived) paths.
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    let saved_kimi_home = std::env::var_os("KIMI_CODE_HOME");
    std::env::remove_var("KIMI_CODE_HOME");
    let tmp = tempdir().unwrap();
    let home = tmp.path().join("home");
    let kimi_dir = home.join(".kimi-code");

    // Source tree with the CE loader (required by sync's desired-set build).
    let source_root = tmp.path().join("ce-source");
    std::fs::create_dir_all(source_root.join(".opencode/plugins")).unwrap();
    std::fs::write(
        source_root.join(".opencode/plugins/compound-engineering.js"),
        b"export default function ceLoader() { /* session.created */ }",
    )
    .unwrap();

    // Pre-existing EMPTY manifest (the audited broken state) + managed tree.
    let managed = kimi_dir.join("compound-engineering");
    std::fs::create_dir_all(managed.join("plugins")).unwrap();
    std::fs::create_dir_all(managed.join("skills/ce-brainstorm")).unwrap();
    std::fs::write(managed.join("plugins/compound-engineering.js"), b"loader").unwrap();
    std::fs::write(managed.join("skills/ce-brainstorm/SKILL.md"), b"# skill").unwrap();
    InstallManifest {
        version: "compound-engineering-v3.24.0".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: "2026-01-02T03:04:05Z".into(),
        source: serde_json::json!({"kind": "local", "path": source_root.display().to_string()}),
        files: vec![],
        config_mutations: vec![],
    }
    .write(&kimi_dir)
    .unwrap();

    let ctx = Context {
        config_dir: home.join(".ce-ai"),
        opencode_config_dir: home.join(".config").join("opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    std::fs::create_dir_all(&ctx.config_dir).unwrap();

    let mut state = State::new();
    state.installed_harnesses.push(serde_json::json!({
        "name": "kimi",
        "version": "compound-engineering-v3.24.0",
        "scope": "global",
        "installed_at": "2026-08-22T00:00:00Z"
    }));
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    sync_with(
        &ctx,
        &source_root,
        "compound-engineering-v3.24.1",
        serde_json::json!({"kind": "local", "path": source_root.display().to_string()}),
    )
    .unwrap();

    let rewritten = InstallManifest::load(&kimi_dir).unwrap();
    assert_eq!(rewritten.version, "compound-engineering-v3.24.1");
    assert_eq!(
        rewritten.installed_at, "2026-01-02T03:04:05Z",
        "sync must preserve the prior manifest installed_at"
    );
    assert_eq!(
        rewritten.files.len(),
        2,
        "harvested entries: {:?}",
        rewritten.files
    );
    let loader = rewritten
        .files
        .iter()
        .find(|f| f.path == "plugins/compound-engineering.js")
        .expect("loader entry harvested");
    assert_eq!(loader.sha256, sha256_hex(b"loader"));
    let skill = rewritten
        .files
        .iter()
        .find(|f| f.path == "skills/ce-brainstorm/SKILL.md")
        .expect("skill entry harvested");
    assert_eq!(skill.sha256, sha256_hex(b"# skill"));

    if let Some(v) = saved_kimi_home {
        std::env::set_var("KIMI_CODE_HOME", v);
    }
}
