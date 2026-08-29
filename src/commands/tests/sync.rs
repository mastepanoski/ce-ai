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
