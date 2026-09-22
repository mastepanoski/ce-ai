//! Risk-Aware Tool Execution & Intelligent Permission Evaluation Engine.
//!
//! Implements a defense-in-depth security model:
//! 1. Deterministic Security Rules (pre-filter, categorical denials/allows).
//! 2. Semantic Risk Dimension Evaluation via System 1 Decision Engine.
//! 3. Conservative Fail-Closed Fallback (RequireConfirmation/Deny on error).
//! 4. Audit Recording & Sensitive Content Redaction.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::decisions::types::{DecisionAnswer, DecisionContext, DecisionQuestion, DecisionRequest};
use crate::decisions::DecisionEngine;
use crate::error::CeError;

/// Outcome policy for tool execution authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPolicy {
    /// Safe to execute automatically without intervention.
    Allow,
    /// Ambiguous or sensitive; requires explicit human confirmation.
    RequireConfirmation,
    /// Categorically prohibited or exceeding risk ceiling; blocked outright.
    Deny,
}

impl ExecutionPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::RequireConfirmation => "require_confirmation",
            Self::Deny => "deny",
        }
    }
}

/// Fallback policy applied when decision provider is offline or errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskFallbackPolicy {
    #[default]
    RequireConfirmation,
    Deny,
}

impl RiskFallbackPolicy {
    pub fn to_execution_policy(&self) -> ExecutionPolicy {
        match self {
            Self::RequireConfirmation => ExecutionPolicy::RequireConfirmation,
            Self::Deny => ExecutionPolicy::Deny,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RequireConfirmation => "require_confirmation",
            Self::Deny => "deny",
        }
    }
}

/// Configurable integer-percentage thresholds for risk escalation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskThresholds {
    #[serde(default = "default_confirmation_threshold")]
    pub confirmation_threshold_pct: u32,
    #[serde(default = "default_deny_threshold")]
    pub deny_threshold_pct: u32,
}

fn default_confirmation_threshold() -> u32 {
    60
}

fn default_deny_threshold() -> u32 {
    90
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            confirmation_threshold_pct: default_confirmation_threshold(),
            deny_threshold_pct: default_deny_threshold(),
        }
    }
}

/// Configuration for risk evaluation in `state.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RiskConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub thresholds: RiskThresholds,
    #[serde(default)]
    pub fallback: RiskFallbackPolicy,
}

/// Standardized risk dimension question definitions.
pub const RECOGNIZED_RISK_DIMENSIONS: &[(&str, &str)] = &[
    (
        "destructive",
        "Does this command delete, truncate, or overwrite critical files, databases, or state?",
    ),
    (
        "credential_sensitive",
        "Does this command read, expose, or transfer secrets, credentials, or private tokens?",
    ),
    (
        "external_side_effect",
        "Does this command make outbound network requests or modify external cloud resources?",
    ),
    (
        "privilege_escalation",
        "Does this command attempt administrative, root, or elevated privileges?",
    ),
    (
        "irreversible",
        "Can the effects of this operation NOT be undone via Git history or local rollback?",
    ),
    (
        "scope_exceeds_task",
        "Does this operation perform actions completely outside the scope of the stated task?",
    ),
];

/// Evaluation confidence for an individual risk dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskDimensionScore {
    pub dimension: String,
    pub confidence: f64,
    pub elevated: bool,
}

/// Structured outcome of a risk evaluation query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskEvaluationResult {
    pub tool: String,
    pub command: String,
    pub task: Option<String>,
    pub policy: ExecutionPolicy,
    pub reason: String,
    pub composite_risk_score: f64,
    pub dimensions: Vec<RiskDimensionScore>,
    pub fallback_applied: bool,
    pub latency_ms: u64,
}

/// Sanitizes sensitive secrets (API keys, private keys, passwords) from text.
pub fn redact_sensitive_content(content: &str) -> String {
    let mut result = String::new();
    let mut in_private_key = false;

    for line in content.lines() {
        if line.contains("-----BEGIN") && line.contains("PRIVATE KEY-----") {
            in_private_key = true;
            result.push_str("[REDACTED_PRIVATE_KEY]\n");
            continue;
        }
        if in_private_key {
            if line.contains("-----END") && line.contains("PRIVATE KEY-----") {
                in_private_key = false;
            }
            continue;
        }

        let mut redacted_line = line.to_string();

        for sensitive_keyword in [
            "password",
            "token",
            "secret",
            "api_key",
            "apikey",
            "auth_token",
        ] {
            for delim in [':', '='] {
                let lower = redacted_line.to_lowercase();
                let pattern = format!("{sensitive_keyword}{delim}");
                let mut start_search = 0;
                while let Some(idx) = lower[start_search..].find(&pattern) {
                    let actual_idx = start_search + idx;
                    let val_start = actual_idx + pattern.len();
                    let val_end = redacted_line[val_start..]
                        .find(|c: char| c.is_whitespace() || c == ',' || c == '\'' || c == '"')
                        .map(|i| val_start + i)
                        .unwrap_or(redacted_line.len());
                    if val_end > val_start {
                        let prefix = &redacted_line[..val_start];
                        let suffix = &redacted_line[val_end..];
                        redacted_line = format!("{prefix}[REDACTED]{suffix}");
                        start_search = val_start + "[REDACTED]".len();
                    } else {
                        break;
                    }
                }
            }
        }

        for prefix in ["sk-", "ghp_", "ts_"] {
            let mut start_search = 0;
            while let Some(idx) = redacted_line[start_search..].find(prefix) {
                let actual_idx = start_search + idx;
                let val_end = redacted_line[actual_idx..]
                    .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                    .map(|i| actual_idx + i)
                    .unwrap_or(redacted_line.len());
                let token = &redacted_line[actual_idx..val_end];
                if token.len() > 8 {
                    redacted_line = format!(
                        "{}[REDACTED]{}",
                        &redacted_line[..actual_idx],
                        &redacted_line[val_end..]
                    );
                    start_search = actual_idx + "[REDACTED]".len();
                } else if val_end > actual_idx {
                    start_search = val_end;
                } else {
                    break;
                }
            }
        }

        result.push_str(&redacted_line);
        result.push('\n');
    }

    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

/// Evaluates deterministic security rules that categorically deny execution.
pub fn check_deterministic_denial(tool: &str, command: &str) -> Option<&'static str> {
    let lower_tool = tool.to_lowercase();
    let lower_cmd = command.to_lowercase();
    let trimmed = lower_cmd.trim();

    if lower_tool == "forbidden" || lower_tool == "blocked" {
        return Some("Tool is categorically forbidden by security policy");
    }

    // Root filesystem wipes
    if trimmed.contains("rm -rf /")
        || trimmed.contains("rm -fr /")
        || trimmed.contains("rm -r /")
        || trimmed.contains("rm -rf /*")
        || trimmed.contains("rm -fr /*")
    {
        return Some("Destructive root filesystem wipe detected");
    }

    // Direct device/formatting wipe
    if trimmed.starts_with("mkfs")
        || trimmed.contains("mkfs.")
        || trimmed.contains("dd if=/dev/zero")
        || trimmed.contains("dd if=/dev/urandom")
    {
        return Some("Direct block device wipe or format detected");
    }

    // Fork bomb
    if trimmed.contains(":(){ :|:& };:") {
        return Some("Denial of service fork bomb detected");
    }

    // Privilege escalation
    if trimmed.starts_with("sudo ")
        || trimmed.contains(" sudo ")
        || trimmed.starts_with("doas ")
        || trimmed.contains(" doas ")
        || trimmed.starts_with("su -")
        || trimmed == "su"
        || trimmed.starts_with("su root")
    {
        return Some("Privilege escalation attempt detected");
    }

    // System-wide permission modification
    if trimmed.contains("chmod -r 777 /")
        || trimmed.contains("chmod 777 /")
        || trimmed.contains("chown -r root /")
    {
        return Some("Dangerous system permission modification detected");
    }

    // Protected system directories
    for path in [
        "/etc/shadow",
        "/etc/sudoers",
        "/etc/passwd",
        "/boot/",
        "/proc/",
        "/sys/",
    ] {
        if trimmed.contains(path)
            && (trimmed.contains("rm ")
                || trimmed.contains("echo ")
                || trimmed.contains('>')
                || trimmed.contains("mv "))
        {
            return Some("Modification of protected system path detected");
        }
    }

    // Protected cryptographic key material
    for cred in [".ssh/id_rsa", ".ssh/id_ed25519", ".ssh/id_ecdsa", ".gnupg/"] {
        if trimmed.contains(cred)
            && (trimmed.contains("rm ")
                || trimmed.contains("cat ")
                || trimmed.contains("curl ")
                || trimmed.contains('>'))
        {
            return Some("Access or exfiltration of protected cryptographic key material detected");
        }
    }

    // Force push to primary branches
    if trimmed.contains("git push")
        && (trimmed.contains("-f") || trimmed.contains("--force"))
        && (trimmed.contains("main") || trimmed.contains("master"))
    {
        return Some("Force push to protected branch detected");
    }

    None
}

/// Evaluates if a tool invocation is categorically safe and read-only.
pub fn is_safe_read_only(tool: &str, command: &str) -> bool {
    let lower_tool = tool.to_lowercase();
    let lower_cmd = command.to_lowercase();
    let trimmed = lower_cmd.trim();

    if matches!(
        lower_tool.as_str(),
        "read_file" | "view_file" | "list_dir" | "grep_search"
    ) {
        return true;
    }

    if trimmed.contains('|')
        || trimmed.contains('>')
        || trimmed.contains('&')
        || trimmed.contains(';')
        || trimmed.contains('$')
    {
        return false;
    }

    let first_token = trimmed.split_whitespace().next().unwrap_or("");
    matches!(
        first_token,
        "ls" | "pwd" | "cat" | "head" | "tail" | "which" | "whereis" | "whoami" | "uname"
    ) || (first_token == "git"
        && matches!(
            trimmed.split_whitespace().nth(1).unwrap_or(""),
            "status" | "diff" | "log" | "branch" | "show"
        ))
        || (first_token == "cargo"
            && matches!(
                trimmed.split_whitespace().nth(1).unwrap_or(""),
                "--version" | "check" | "clippy"
            ))
}

/// Risk evaluation engine combining deterministic pre-filtering with System 1 micro-decisions.
pub struct RiskEvaluator<'a> {
    config: &'a RiskConfig,
    engine: Option<&'a DecisionEngine>,
    config_dir: Option<&'a Path>,
    workflow_id: Option<String>,
}

impl<'a> RiskEvaluator<'a> {
    pub fn new(config: &'a RiskConfig, engine: Option<&'a DecisionEngine>) -> Self {
        Self {
            config,
            engine,
            config_dir: None,
            workflow_id: None,
        }
    }

    pub fn with_config_dir(mut self, config_dir: &'a Path) -> Self {
        self.config_dir = Some(config_dir);
        self
    }

    pub fn with_workflow(mut self, workflow_id: Option<impl Into<String>>) -> Self {
        self.workflow_id = workflow_id.map(Into::into);
        self
    }

    /// Evaluates tool execution risk and yields an authoritative `ExecutionPolicy`.
    pub fn evaluate(&self, tool: &str, command: &str, task: Option<&str>) -> RiskEvaluationResult {
        let sanitized_command = redact_sensitive_content(command);

        // Tier 1: Deterministic Denial Pre-filter (Hard Security Boundaries)
        if let Some(reason) = check_deterministic_denial(tool, command) {
            return RiskEvaluationResult {
                tool: tool.to_string(),
                command: sanitized_command,
                task: task.map(String::from),
                policy: ExecutionPolicy::Deny,
                reason: format!("Deterministic denial: {reason}"),
                composite_risk_score: 1.0,
                dimensions: Vec::new(),
                fallback_applied: false,
                latency_ms: 0,
            };
        }

        // Tier 1b: Safe Read-Only Commands
        if is_safe_read_only(tool, command) {
            return RiskEvaluationResult {
                tool: tool.to_string(),
                command: sanitized_command,
                task: task.map(String::from),
                policy: ExecutionPolicy::Allow,
                reason: "Categorically safe read-only operation".into(),
                composite_risk_score: 0.0,
                dimensions: Vec::new(),
                fallback_applied: false,
                latency_ms: 0,
            };
        }

        // Tier 2: Probabilistic Decision Engine Evaluation
        if !self.config.enabled {
            return RiskEvaluationResult {
                tool: tool.to_string(),
                command: sanitized_command,
                task: task.map(String::from),
                policy: ExecutionPolicy::Allow,
                reason: "Risk-aware execution is disabled in configuration.".into(),
                composite_risk_score: 0.0,
                dimensions: Vec::new(),
                fallback_applied: true,
                latency_ms: 0,
            };
        }

        let engine = match self.engine {
            Some(e) if e.is_enabled() => e,
            _ => {
                let policy = self.config.fallback.to_execution_policy();
                return RiskEvaluationResult {
                    tool: tool.to_string(),
                    command: sanitized_command,
                    task: task.map(String::from),
                    policy,
                    reason: format!(
                        "Decision engine unconfigured; conservative fallback applied ({})",
                        policy.as_str()
                    ),
                    composite_risk_score: 0.5,
                    dimensions: Vec::new(),
                    fallback_applied: true,
                    latency_ms: 0,
                };
            }
        };

        let context_prompt = match task {
            Some(t) => format!("Task: {t}\nTool: {tool}\nCommand: {sanitized_command}"),
            None => format!("Tool: {tool}\nCommand: {sanitized_command}"),
        };

        let mut req = DecisionRequest::new(DecisionContext::new(&context_prompt));
        for (dim, q) in RECOGNIZED_RISK_DIMENSIONS {
            req = req.with_question(DecisionQuestion::boolean(*dim, *q));
        }
        req = req.with_question(DecisionQuestion::choice(
            "overall_risk",
            "What is the overall risk of this tool invocation?",
            vec!["safe", "sensitive", "destructive"],
        ));

        let resp = match engine.evaluate(req) {
            Ok(r) => r,
            Err(e) => {
                let policy = self.config.fallback.to_execution_policy();
                return RiskEvaluationResult {
                    tool: tool.to_string(),
                    command: sanitized_command,
                    task: task.map(String::from),
                    policy,
                    reason: format!(
                        "Decision provider error ({e}); conservative fallback applied ({})",
                        policy.as_str()
                    ),
                    composite_risk_score: 0.5,
                    dimensions: Vec::new(),
                    fallback_applied: true,
                    latency_ms: 0,
                };
            }
        };

        if resp.fallback_used && !resp.shadow_mode {
            let policy = self.config.fallback.to_execution_policy();
            let res = RiskEvaluationResult {
                tool: tool.to_string(),
                command: sanitized_command,
                task: task.map(String::from),
                policy,
                reason: format!(
                    "Decision provider returned fallback; conservative fallback applied ({})",
                    policy.as_str()
                ),
                composite_risk_score: 0.5,
                dimensions: Vec::new(),
                fallback_applied: true,
                latency_ms: resp.latency_ms,
            };
            if let Some(cd) = self.config_dir {
                let event = crate::decisions::analytics::DecisionEvent::new(
                    crate::decisions::analytics::DecisionType::RiskClassification,
                    &resp.provider,
                    &resp.model,
                    resp.latency_ms,
                    policy.as_str(),
                )
                .with_workflow(self.workflow_id.clone())
                .with_fallback(true)
                .with_shadow(false)
                .with_metadata("tool", tool.to_string())
                .with_metadata(
                    "command_summary",
                    crate::decisions::analytics::sanitize_task_summary(&res.command),
                );
                let _ = crate::decisions::analytics::log_decision_event(cd, &event);
            }
            return res;
        }

        let mut dimensions = Vec::new();
        let mut max_dimension_confidence: f64 = 0.0;

        for (dim, _) in RECOGNIZED_RISK_DIMENSIONS {
            let (val, conf) = match resp.get_answer(dim) {
                Some(DecisionAnswer::Boolean { value, confidence }) => (*value, *confidence),
                _ => (false, 0.0),
            };

            let score = if val { conf } else { 0.0 };
            if score > max_dimension_confidence {
                max_dimension_confidence = score;
            }

            dimensions.push(RiskDimensionScore {
                dimension: dim.to_string(),
                confidence: conf,
                elevated: val
                    && conf >= (self.config.thresholds.confirmation_threshold_pct as f64 / 100.0),
            });
        }

        let overall_score = match resp.get_answer("overall_risk") {
            Some(DecisionAnswer::Choice {
                selected,
                confidence,
                ..
            }) => match selected.as_str() {
                "destructive" => *confidence,
                "sensitive" => *confidence * 0.7,
                _ => 0.0,
            },
            _ => 0.0,
        };

        let composite_score = max_dimension_confidence.max(overall_score);
        let deny_thresh = self.config.thresholds.deny_threshold_pct as f64 / 100.0;
        let confirm_thresh = self.config.thresholds.confirmation_threshold_pct as f64 / 100.0;

        let (evaluated_policy, evaluated_reason) = if composite_score >= deny_thresh {
            (
                ExecutionPolicy::Deny,
                format!(
                    "Risk score ({:.0}%) exceeds deny threshold ({:.0}%)",
                    composite_score * 100.0,
                    deny_thresh * 100.0
                ),
            )
        } else if composite_score >= confirm_thresh {
            (
                ExecutionPolicy::RequireConfirmation,
                format!(
                    "Risk score ({:.0}%) exceeds confirmation threshold ({:.0}%)",
                    composite_score * 100.0,
                    confirm_thresh * 100.0
                ),
            )
        } else {
            (
                ExecutionPolicy::Allow,
                format!(
                    "Risk score ({:.0}%) is below confirmation threshold ({:.0}%)",
                    composite_score * 100.0,
                    confirm_thresh * 100.0
                ),
            )
        };

        let (policy, reason) = if resp.shadow_mode {
            (
                ExecutionPolicy::Allow,
                format!(
                    "Shadow mode active: suggested policy was {} (score {:.0}%), but execution policy remains unconstrained.",
                    evaluated_policy.as_str(),
                    composite_score * 100.0
                ),
            )
        } else {
            (evaluated_policy, evaluated_reason)
        };

        if let Some(cd) = self.config_dir {
            let event = crate::decisions::analytics::DecisionEvent::new(
                crate::decisions::analytics::DecisionType::RiskClassification,
                &resp.provider,
                &resp.model,
                resp.latency_ms,
                evaluated_policy.as_str(),
            )
            .with_workflow(self.workflow_id.clone())
            .with_confidence(Some(composite_score))
            .with_shadow(resp.shadow_mode)
            .with_fallback(false)
            .with_estimated_cost(resp.estimated_cost_usd)
            .with_metadata("tool", tool.to_string())
            .with_metadata(
                "command_summary",
                crate::decisions::analytics::sanitize_task_summary(&sanitized_command),
            );
            let _ = crate::decisions::analytics::log_decision_event(cd, &event);
        }

        RiskEvaluationResult {
            tool: tool.to_string(),
            command: sanitized_command,
            task: task.map(String::from),
            policy,
            reason,
            composite_risk_score: composite_score,
            dimensions,
            fallback_applied: false,
            latency_ms: resp.latency_ms,
        }
    }
}

/// Canonical path to `risk-events.jsonl` within the configuration directory.
pub fn risk_events_log_path(config_dir: &Path) -> PathBuf {
    config_dir.join("risk-events.jsonl")
}

/// Appends a structured, sanitized risk evaluation event record to `risk-events.jsonl`.
pub fn log_risk_event(config_dir: &Path, record: &RiskEvaluationResult) -> Result<(), CeError> {
    let log_path = risk_events_log_path(config_dir);
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let serialized = serde_json::to_string(record).map_err(|e| {
        CeError::Runtime(format!("failed to serialize risk evaluation record: {e}"))
    })?;
    writeln!(file, "{serialized}")?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/risk_tests.rs"]
mod tests;
