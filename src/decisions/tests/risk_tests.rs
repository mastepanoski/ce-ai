use super::*;
use crate::decisions::mock::MockDecisionProvider;
use crate::decisions::types::DecisionAnswer;
use crate::decisions::DecisionMode;
use tempfile::tempdir;

#[test]
fn test_risk_config_defaults_and_serde_and_eq() {
    let config = RiskConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.thresholds.confirmation_threshold_pct, 60);
    assert_eq!(config.thresholds.deny_threshold_pct, 90);
    assert_eq!(config.fallback, RiskFallbackPolicy::RequireConfirmation);

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: RiskConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}

#[test]
fn test_deterministic_denials_cannot_be_overridden() {
    let denied_commands = [
        ("run_command", "rm -rf /"),
        ("run_command", "rm -fr /*"),
        ("run_command", "rm -r /"),
        ("run_command", "mkfs.ext4 /dev/sda1"),
        ("run_command", "dd if=/dev/zero of=/dev/sda"),
        ("run_command", ":(){ :|:& };:"),
        ("run_command", "sudo rm -rf /var/log"),
        ("run_command", "doas reboot"),
        ("run_command", "su -"),
        ("run_command", "chmod 777 /"),
        ("run_command", "rm /etc/shadow"),
        ("run_command", "cat .ssh/id_rsa > /tmp/key"),
        ("run_command", "git push --force origin main"),
        ("forbidden", "echo hello"),
        ("blocked", "ls"),
    ];

    for (tool, cmd) in denied_commands {
        let reason = check_deterministic_denial(tool, cmd);
        assert!(
            reason.is_some(),
            "Expected deterministic denial for tool='{tool}', cmd='{cmd}'"
        );
    }

    // Even if a mock decision provider answers completely safe with 100% confidence,
    // deterministic security pre-filter must still unconditionally deny!
    let mock = MockDecisionProvider::new()
        .with_canned_answer(
            "destructive",
            DecisionAnswer::Boolean {
                value: false,
                confidence: 1.0,
            },
        )
        .with_canned_answer(
            "overall_risk",
            DecisionAnswer::Choice {
                selected: "safe".into(),
                confidence: 1.0,
                probabilities: std::collections::BTreeMap::new(),
            },
        );
    let engine = DecisionEngine::with_mock(mock, DecisionMode::Active);
    let config = RiskConfig {
        enabled: true,
        thresholds: RiskThresholds::default(),
        fallback: RiskFallbackPolicy::RequireConfirmation,
    };
    let evaluator = RiskEvaluator::new(&config, Some(&engine));
    let result = evaluator.evaluate("run_command", "sudo rm -rf /", Some("Fix system"));

    assert_eq!(result.policy, ExecutionPolicy::Deny);
    assert!(result.reason.contains("Deterministic denial"));
    assert_eq!(result.composite_risk_score, 1.0);
    assert_eq!(result.latency_ms, 0);
}

#[test]
fn test_safe_read_only_commands() {
    let safe_tools = ["read_file", "view_file", "list_dir", "grep_search"];
    for tool in safe_tools {
        assert!(is_safe_read_only(tool, "any content or argument"));
    }

    let safe_commands = [
        ("run_command", "ls"),
        ("run_command", "ls -la src/"),
        ("run_command", "pwd"),
        ("run_command", "cat Cargo.toml"),
        ("run_command", "head -n 20 src/main.rs"),
        ("run_command", "git status"),
        ("run_command", "git diff HEAD~1"),
        ("run_command", "cargo check"),
        ("run_command", "cargo clippy"),
    ];

    for (tool, cmd) in safe_commands {
        assert!(
            is_safe_read_only(tool, cmd),
            "Expected '{cmd}' to be considered safe read-only"
        );
    }

    let unsafe_commands = [
        ("run_command", "ls | rm -rf"),
        ("run_command", "cat file > overwrite.txt"),
        ("run_command", "cargo build && rm file"),
        ("run_command", "echo $(whoami)"),
    ];

    for (tool, cmd) in unsafe_commands {
        assert!(
            !is_safe_read_only(tool, cmd),
            "Expected '{cmd}' with chaining or redirection NOT to be safe read-only"
        );
    }

    let config = RiskConfig {
        enabled: true,
        ..Default::default()
    };
    let evaluator = RiskEvaluator::new(&config, None);
    let result = evaluator.evaluate("run_command", "git status", None);
    assert_eq!(result.policy, ExecutionPolicy::Allow);
    assert!(result.reason.contains("safe read-only"));
    assert_eq!(result.composite_risk_score, 0.0);
}

#[test]
fn test_redact_sensitive_content() {
    let input =
        "curl -H 'Authorization: Bearer sk-ant-api03-abcdef123456789' https://api.anthropic.com";
    let redacted = redact_sensitive_content(input);
    assert!(!redacted.contains("sk-ant-api03-abcdef123456789"));
    assert!(redacted.contains("[REDACTED]"));

    let input_pass = "postgres://user:password=super_secret_db_pass@localhost:5432/db";
    let redacted_pass = redact_sensitive_content(input_pass);
    assert!(!redacted_pass.contains("super_secret_db_pass"));
    assert!(redacted_pass.contains("[REDACTED]"));

    let input_ghp = "git clone https://ghp_0123456789abcdef0123456789@github.com/repo";
    let redacted_ghp = redact_sensitive_content(input_ghp);
    assert!(!redacted_ghp.contains("ghp_0123456789abcdef0123456789"));
    assert!(redacted_ghp.contains("[REDACTED]"));

    let key_block = "header\n-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0...\n-----END RSA PRIVATE KEY-----\nfooter";
    let redacted_key = redact_sensitive_content(key_block);
    assert!(!redacted_key.contains("MIIEowIBAAKCAQEA0..."));
    assert!(redacted_key.contains("[REDACTED_PRIVATE_KEY]"));
    assert!(redacted_key.contains("header"));
    assert!(redacted_key.contains("footer"));
}

#[test]
fn test_risk_evaluator_disabled_allows() {
    let config = RiskConfig {
        enabled: false,
        ..Default::default()
    };
    let evaluator = RiskEvaluator::new(&config, None);
    let result = evaluator.evaluate("run_command", "npm install", Some("Setup project"));
    assert_eq!(result.policy, ExecutionPolicy::Allow);
    assert!(result.reason.contains("disabled"));
}

#[test]
fn test_risk_evaluator_probabilistic_thresholds() {
    let config = RiskConfig {
        enabled: true,
        thresholds: RiskThresholds {
            confirmation_threshold_pct: 60,
            deny_threshold_pct: 90,
        },
        fallback: RiskFallbackPolicy::RequireConfirmation,
    };

    // Case 1: Low risk -> Allow
    let mock_safe = MockDecisionProvider::new()
        .with_canned_answer(
            "destructive",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.30, // below 60%
            },
        )
        .with_canned_answer(
            "overall_risk",
            DecisionAnswer::Choice {
                selected: "safe".into(),
                confidence: 0.85,
                probabilities: std::collections::BTreeMap::new(),
            },
        );
    let engine_safe = DecisionEngine::with_mock(mock_safe, DecisionMode::Active);
    let evaluator_safe = RiskEvaluator::new(&config, Some(&engine_safe));
    let result_safe = evaluator_safe.evaluate("run_command", "cargo build", None);
    assert_eq!(result_safe.policy, ExecutionPolicy::Allow);
    assert_eq!(result_safe.composite_risk_score, 0.30);

    // Case 2: Medium risk -> RequireConfirmation (e.g. 75% destructive)
    let mock_confirm = MockDecisionProvider::new()
        .with_canned_answer(
            "destructive",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.75,
            },
        )
        .with_canned_answer(
            "overall_risk",
            DecisionAnswer::Choice {
                selected: "sensitive".into(),
                confidence: 0.70,
                probabilities: std::collections::BTreeMap::new(),
            },
        );
    let engine_confirm = DecisionEngine::with_mock(mock_confirm, DecisionMode::Active);
    let evaluator_confirm = RiskEvaluator::new(&config, Some(&engine_confirm));
    let result_confirm =
        evaluator_confirm.evaluate("run_command", "DROP DATABASE test_db;", Some("Clean tests"));
    assert_eq!(result_confirm.policy, ExecutionPolicy::RequireConfirmation);
    assert_eq!(result_confirm.composite_risk_score, 0.75);
    assert!(result_confirm.reason.contains("confirmation threshold"));

    // Case 3: High risk -> Deny (e.g. 95% destructive)
    let mock_deny = MockDecisionProvider::new()
        .with_canned_answer(
            "destructive",
            DecisionAnswer::Boolean {
                value: true,
                confidence: 0.95,
            },
        )
        .with_canned_answer(
            "overall_risk",
            DecisionAnswer::Choice {
                selected: "destructive".into(),
                confidence: 0.95,
                probabilities: std::collections::BTreeMap::new(),
            },
        );
    let engine_deny = DecisionEngine::with_mock(mock_deny, DecisionMode::Active);
    let evaluator_deny = RiskEvaluator::new(&config, Some(&engine_deny));
    let result_deny = evaluator_deny.evaluate(
        "run_command",
        "rm -rf ./production_data",
        Some("Clear cache"),
    );
    assert_eq!(result_deny.policy, ExecutionPolicy::Deny);
    assert_eq!(result_deny.composite_risk_score, 0.95);
    assert!(result_deny.reason.contains("deny threshold"));
}

#[test]
fn test_risk_evaluator_conservative_fail_closed() {
    let config_confirm = RiskConfig {
        enabled: true,
        thresholds: RiskThresholds::default(),
        fallback: RiskFallbackPolicy::RequireConfirmation,
    };
    // Provider unconfigured or offline
    let evaluator = RiskEvaluator::new(&config_confirm, None);
    let res = evaluator.evaluate("run_command", "npm publish", None);
    assert_eq!(res.policy, ExecutionPolicy::RequireConfirmation);
    assert!(res.fallback_applied);

    let config_deny = RiskConfig {
        enabled: true,
        thresholds: RiskThresholds::default(),
        fallback: RiskFallbackPolicy::Deny,
    };
    let evaluator_deny = RiskEvaluator::new(&config_deny, None);
    let res_deny = evaluator_deny.evaluate("run_command", "npm publish", None);
    assert_eq!(res_deny.policy, ExecutionPolicy::Deny);
    assert!(res_deny.fallback_applied);
}

#[test]
fn test_risk_evaluation_audit_logging() {
    let temp = tempdir().unwrap();
    let config_dir = temp.path();

    let record = RiskEvaluationResult {
        tool: "run_command".into(),
        command: "rm -rf / [REDACTED]".into(),
        task: Some("Wipe system".into()),
        policy: ExecutionPolicy::Deny,
        reason: "Deterministic denial: root wipe".into(),
        composite_risk_score: 1.0,
        dimensions: vec![RiskDimensionScore {
            dimension: "destructive".into(),
            confidence: 1.0,
            elevated: true,
        }],
        fallback_applied: false,
        latency_ms: 0,
    };

    log_risk_event(config_dir, &record).unwrap();

    let log_path = risk_events_log_path(config_dir);
    assert!(log_path.exists());
    let content = std::fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("\"policy\":\"deny\""));
    assert!(content.contains("\"tool\":\"run_command\""));
    assert!(content.contains("\"composite_risk_score\":1.0"));
}
