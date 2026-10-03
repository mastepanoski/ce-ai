use serde::{Deserialize, Serialize};

/// Upstream Compound Engineering solution frontmatter schema specification.
/// Conforms to `skills/ce-compound/references/schema.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CeSolutionFrontmatter {
    #[serde(default)]
    pub schema_version: Option<String>,
    pub module: String,
    pub date: String,
    pub problem_type: String,
    pub component: String,
    #[serde(default)]
    pub related_components: Vec<String>,
    pub severity: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub applies_when: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

impl CeSolutionFrontmatter {
    /// Returns true if the problem type represents a bugfix or bug diagnosis track.
    pub fn is_bug_track(&self) -> bool {
        self.problem_type.eq_ignore_ascii_case("bugfix")
            || self.problem_type.eq_ignore_ascii_case("bug")
    }

    /// Validates compliance with upstream Compound Engineering schema rules:
    /// - `module`, `date`, `problem_type`, `component`, `severity` must be non-empty.
    /// - Non-bug tracks (architecture, pattern, discovery, config, preference) must provide `applies_when`.
    pub fn validate_upstream_compliance(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.module.trim().is_empty() {
            errors.push("missing required field: 'module'".to_string());
        }
        if self.date.trim().is_empty() {
            errors.push("missing required field: 'date'".to_string());
        }
        if self.problem_type.trim().is_empty() {
            errors.push("missing required field: 'problem_type'".to_string());
        }
        if self.component.trim().is_empty() {
            errors.push("missing required field: 'component'".to_string());
        }
        if self.severity.trim().is_empty() {
            errors.push("missing required field: 'severity'".to_string());
        }

        if !self.is_bug_track() && self.applies_when.as_deref().unwrap_or("").trim().is_empty() {
            errors.push(format!(
                "knowledge-track problem_type '{}' requires 'applies_when'",
                self.problem_type
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Validates raw frontmatter from a Markdown document against upstream schema rules.
/// Returns a list of missing required keys (e.g. "category", "problem_type", "applies_when").
pub fn check_solution_frontmatter(content: &str) -> Vec<String> {
    let mut missing = Vec::new();
    let header_lines = match extract_yaml_frontmatter(content) {
        Some(lines) => lines,
        None => {
            return vec![
                "category".into(),
                "problem_type".into(),
                "applies_when".into(),
            ];
        }
    };

    let mut has_category_or_module = false;
    let mut problem_type_val: Option<String> = None;
    let mut has_applies_when = false;

    for line in header_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some((k, v)) = trimmed.split_once(':') {
                let key = k.trim().to_lowercase();
                let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
                match key.as_str() {
                    "category" | "module" => has_category_or_module = true,
                    "problem_type" => problem_type_val = Some(val),
                    "applies_when" => has_applies_when = true,
                    _ => {}
                }
            }
        }
    }

    if !has_category_or_module {
        missing.push("category".into());
    }
    let is_bugfix = match &problem_type_val {
        Some(pt) => {
            let clean = pt.to_lowercase();
            clean == "bugfix" || clean == "bug"
        }
        None => false,
    };
    if problem_type_val.is_none() {
        missing.push("problem_type".into());
    }
    if !is_bugfix && !has_applies_when {
        missing.push("applies_when".into());
    }

    missing
}

fn extract_yaml_frontmatter(content: &str) -> Option<Vec<&str>> {
    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut header = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            return Some(header);
        }
        header.push(line);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bug_track_validation_without_applies_when() {
        let frontmatter = CeSolutionFrontmatter {
            schema_version: Some("2.0".to_string()),
            module: "harness::claude".to_string(),
            date: "2026-10-02".to_string(),
            problem_type: "bugfix".to_string(),
            component: "claude".to_string(),
            related_components: vec![],
            severity: "standard".to_string(),
            tags: vec!["bug".to_string()],
            applies_when: None,
            title: Some("Fix Claude Hook".to_string()),
        };

        assert!(frontmatter.is_bug_track());
        assert!(frontmatter.validate_upstream_compliance().is_ok());
    }

    #[test]
    fn test_knowledge_track_requires_applies_when() {
        let mut frontmatter = CeSolutionFrontmatter {
            schema_version: Some("2.0".to_string()),
            module: "architecture::v2".to_string(),
            date: "2026-10-02".to_string(),
            problem_type: "architecture".to_string(),
            component: "architecture".to_string(),
            related_components: vec!["compat".to_string()],
            severity: "standard".to_string(),
            tags: vec!["v2".to_string()],
            applies_when: None,
            title: Some("V2 Design".to_string()),
        };

        assert!(!frontmatter.is_bug_track());
        let res = frontmatter.validate_upstream_compliance();
        assert!(res.is_err());
        assert!(res.unwrap_err()[0].contains("requires 'applies_when'"));

        frontmatter.applies_when = Some("When planning v2".to_string());
        assert!(frontmatter.validate_upstream_compliance().is_ok());
    }
}
