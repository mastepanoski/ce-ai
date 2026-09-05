use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;

use tempfile::tempdir;

use crate::error::CeError;
use crate::harness::registration::{registration_spec, RegistrationSpec};
use crate::harness::HarnessKind;

#[test]
fn table_driven_harnesses_have_valid_mcp_registrars() {
    use HarnessKind::*;
    for kind in [Claude, Codex, Copilot, Cursor, Grok, Kimi, Agy, Fx] {
        let spec = registration_spec(kind)
            .unwrap_or_else(|| panic!("expected registration spec for table-driven kind {kind:?}"));
        assert!(
            spec.register_mcp.is_some(),
            "expected Some(register_mcp) for kind {kind:?}"
        );
    }
}

#[test]
fn pi_is_explicitly_no_mcp_by_design() {
    let pi = registration_spec(HarnessKind::Pi).expect("pi registration spec");
    assert!(
        pi.register_mcp.is_none(),
        "Pi must have None for register_mcp (No-MCP by design)"
    );

    // Calling register_companions on No-MCP spec is a silent no-op
    let temp = tempdir().unwrap();
    let dummy_path = temp.path().join("dummy.json");
    assert!(pi.register_companions(&dummy_path).is_ok());
    assert!(!dummy_path.exists());
}

#[test]
fn dedicated_arm_kinds_and_descope_return_none() {
    use HarnessKind::*;
    for kind in [Opencode, Custom, Deepseek] {
        assert!(
            registration_spec(kind).is_none(),
            "kind {kind:?} must return None from registration_spec (handled in dedicated arms or de-scoped)"
        );
    }
}

static SPY_CALLS: Mutex<Vec<(String, String, Vec<String>)>> = Mutex::new(Vec::new());

fn spy_mcp_registrar(
    _target_config: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    _env: &BTreeMap<String, String>,
) -> Result<(), CeError> {
    SPY_CALLS.lock().unwrap().push((
        name.to_string(),
        command.to_string(),
        args.iter().map(|s| s.to_string()).collect(),
    ));
    Ok(())
}

#[test]
fn register_companions_registers_codegraph_and_engram() {
    SPY_CALLS.lock().unwrap().clear();

    let spec = RegistrationSpec {
        register_mcp: Some(spy_mcp_registrar),
    };

    let temp = tempdir().unwrap();
    let config = temp.path().join("config.json");
    spec.register_companions(&config).unwrap();

    let calls = SPY_CALLS.lock().unwrap().clone();
    assert_eq!(calls.len(), 2, "must register exactly 2 companions");

    assert_eq!(calls[0].0, "codegraph");
    assert_eq!(calls[0].1, "codegraph");
    assert_eq!(calls[0].2, vec!["mcp".to_string()]);

    assert_eq!(calls[1].0, "engram");
    assert_eq!(calls[1].1, "engram");
    assert_eq!(calls[1].2, vec!["serve".to_string()]);
}
