use std::fs;
use tempfile::tempdir;

use super::*;

#[test]
fn test_parse_domain_spec_metadata_valid() {
    let content = r#"---
title: "Harness Integration"
domain: harnesses
version: 1.2.0
last_updated: "2026-09-15"
dependencies: [state, doctor]
---

# Specification Content
"#;

    let meta = parse_domain_spec_metadata(content).expect("should parse valid frontmatter");
    assert_eq!(meta.title, "Harness Integration");
    assert_eq!(meta.domain, "harnesses");
    assert_eq!(meta.version, "1.2.0");
    assert_eq!(meta.last_updated, "2026-09-15");
    assert_eq!(meta.dependencies, vec!["state", "doctor"]);
}

#[test]
fn test_parse_domain_spec_metadata_missing_title() {
    let content = r#"---
domain: harnesses
---
"#;
    let err = parse_domain_spec_metadata(content).unwrap_err();
    assert!(err.contains("missing required field 'title'"));
}

#[test]
fn test_parse_domain_spec_metadata_missing_delimiters() {
    let content = "title: no delimiters\n";
    let err = parse_domain_spec_metadata(content).unwrap_err();
    assert!(err.contains("missing YAML frontmatter delimiters"));
}

#[test]
fn test_count_requirements() {
    let content = r#"
## Capabilities & Requirements

### R1. First Requirement
WHEN something happens
THEN do this.

### R2. Second Requirement
WHEN other thing happens
THEN do that.

### Not a requirement header
Just a subsection.
"#;
    assert_eq!(count_requirements(content), 2);
}

#[test]
fn test_extract_requirements_from_spec_md() {
    let content = r#"---
title: "Sample Feature"
domain: workflow
---

# Specification

## Requirements

### R1. First Requirement
WHEN condition A
THEN system MUST behave accordingly.

### R2. Second Requirement
WHEN condition B
THEN system SHALL reject operation.
"#;

    let reqs = extract_requirements_from_spec_md(content);
    assert_eq!(reqs.len(), 2);
    assert_eq!(reqs[0].0, "R1. First Requirement");
    assert!(reqs[0].1.contains("WHEN condition A"));
    assert_eq!(reqs[1].0, "R2. Second Requirement");
    assert!(reqs[1].1.contains("WHEN condition B"));
}

#[test]
fn test_validate_domain_specs_clean() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let specs_dir = root.join("openspec").join("specs");
    fs::create_dir_all(&specs_dir).unwrap();

    let valid_spec = r#"---
title: "Doctor Diagnostic Engine"
domain: doctor
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Doctor

## 2. Capabilities & Requirements

### R1. Health Probing
WHEN doctor runs
THEN probes evaluate system.
"#;

    fs::write(specs_dir.join("doctor.md"), valid_spec).unwrap();

    let report = validate_domain_specs(root);
    assert!(report.valid);
    assert_eq!(report.specs_count, 1);
    assert!(report.findings.is_empty());
}

#[test]
fn test_validate_domain_specs_mismatched_filename() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let specs_dir = root.join("openspec").join("specs");
    fs::create_dir_all(&specs_dir).unwrap();

    let invalid_spec = r#"---
title: "Doctor Diagnostic Engine"
domain: doctor
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Doctor
## 2. Capabilities & Requirements
"#;

    // File name is workflow.md but domain in frontmatter is doctor
    fs::write(specs_dir.join("workflow.md"), invalid_spec).unwrap();

    let report = validate_domain_specs(root);
    assert!(!report.valid);
    assert_eq!(report.specs_count, 1);
    assert!(report
        .findings
        .iter()
        .any(|f| f.message.contains("does not match filename")));
}

#[test]
fn test_promote_delta_to_domain_spec_dry_run_and_apply() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    // Create change folder with spec.md
    let change_dir = root.join("openspec").join("changes").join("feat-x");
    fs::create_dir_all(&change_dir).unwrap();
    let change_spec = r#"---
title: "Feature X"
domain: workflow
---

# Specification

## Requirements

### R1. New Capability
WHEN feature X is enabled
THEN execute workflow.
"#;
    fs::write(change_dir.join("spec.md"), change_spec).unwrap();

    // Create existing domain spec
    let specs_dir = root.join("openspec").join("specs");
    fs::create_dir_all(&specs_dir).unwrap();
    let existing_domain_spec = r#"---
title: "Workflow Engine"
domain: workflow
version: 1.0.0
last_updated: "2026-01-01"
---

# Specification: Workflow

## 2. Capabilities & Requirements

### R0. Existing Requirement
Original requirement.
"#;
    fs::write(specs_dir.join("workflow.md"), existing_domain_spec).unwrap();

    // 1. Dry run
    let dry_res =
        promote_delta_to_domain_spec(root, "feat-x", None, true).expect("dry run should succeed");
    assert_eq!(dry_res.domain, "workflow");
    assert_eq!(dry_res.requirements_promoted, 1);
    assert!(dry_res.dry_run);

    // Verify disk was NOT mutated during dry run
    let content_after_dry = fs::read_to_string(specs_dir.join("workflow.md")).unwrap();
    assert!(!content_after_dry.contains("R1. New Capability"));

    // 2. Real application
    let apply_res =
        promote_delta_to_domain_spec(root, "feat-x", None, false).expect("apply should succeed");
    assert_eq!(apply_res.domain, "workflow");
    assert_eq!(apply_res.requirements_promoted, 1);
    assert!(!apply_res.dry_run);

    // Verify disk WAS mutated
    let content_after_apply = fs::read_to_string(specs_dir.join("workflow.md")).unwrap();
    assert!(content_after_apply.contains("R1. New Capability"));
    assert!(content_after_apply.contains("WHEN feature X is enabled"));
    assert!(content_after_apply.contains("<!-- promoted-from: change:feat-x"));
}
