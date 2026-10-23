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

#[test]
fn test_list_domain_specs_from_docs_arch_system() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let system_dir = root.join("docs").join("arch").join("system");
    fs::create_dir_all(&system_dir).unwrap();

    let spec_content = r#"---
title: "Doctor Diagnostic Engine"
domain: doctor
version: 1.1.0
last_updated: "2026-10-23"
---

# Specification: Doctor

## 2. Capabilities & Requirements

### R1. Health Probing
WHEN doctor runs
THEN probes evaluate system.
"#;
    fs::write(system_dir.join("doctor.md"), spec_content).unwrap();

    let specs = list_domain_specs(root).expect("should list specs from docs/arch/system");
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].domain, "doctor");
    assert_eq!(specs[0].title, "Doctor Diagnostic Engine");
    assert_eq!(specs[0].version, "1.1.0");
    assert_eq!(specs[0].requirement_count, 1);
    assert_eq!(specs[0].file_path, system_dir.join("doctor.md"));
}

#[test]
fn test_get_domain_spec_path_dual_path() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    // 1. In empty repo -> defaults to modern docs/arch/system
    assert_eq!(
        get_domain_spec_path(root, "doctor"),
        root.join("docs")
            .join("arch")
            .join("system")
            .join("doctor.md")
    );

    // 2. In legacy repo with openspec/specs
    let legacy_specs = root.join("openspec").join("specs");
    fs::create_dir_all(&legacy_specs).unwrap();
    assert_eq!(
        get_domain_spec_path(root, "doctor"),
        legacy_specs.join("doctor.md")
    );

    // 3. When docs/arch/system exists, it takes precedence
    let modern_system = root.join("docs").join("arch").join("system");
    fs::create_dir_all(&modern_system).unwrap();
    assert_eq!(
        get_domain_spec_path(root, "doctor"),
        modern_system.join("doctor.md")
    );
}

#[test]
fn test_promote_delta_to_docs_arch_system() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let deltas_dir = root.join("docs").join("arch").join("deltas").join("feat-y");
    fs::create_dir_all(&deltas_dir).unwrap();
    let delta_spec = r#"---
title: "Feature Y"
domain: harnesses
---

# Specification

## Requirements

### R1. New Harness
WHEN harness initialized
THEN configure isolation.
"#;
    fs::write(deltas_dir.join("spec.md"), delta_spec).unwrap();

    let system_dir = root.join("docs").join("arch").join("system");
    fs::create_dir_all(&system_dir).unwrap();
    let base_spec = r#"---
title: "Harness Integration"
domain: harnesses
version: 1.0.0
last_updated: "2026-10-01"
---

# Specification: Harnesses

## 2. Capabilities & Requirements

### R0. Core Engine
Initial requirement.
"#;
    fs::write(system_dir.join("harnesses.md"), base_spec).unwrap();

    let res = promote_delta_to_domain_spec(root, "feat-y", None, false)
        .expect("should promote into docs/arch/system");
    assert_eq!(res.domain, "harnesses");
    assert_eq!(res.requirements_promoted, 1);
    assert_eq!(res.spec_path, system_dir.join("harnesses.md"));

    let content = fs::read_to_string(system_dir.join("harnesses.md")).unwrap();
    assert!(content.contains("R1. New Harness"));
    assert!(content.contains("<!-- promoted-from: change:feat-y"));
}

#[test]
fn test_security_spec_schema_valid() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let report = validate_domain_specs(&repo_root);
    assert!(
        report.valid,
        "docs/arch/system specifications should be valid, findings: {:?}",
        report.findings
    );
    assert!(
        report.specs_count >= 1,
        "should find at least 1 domain specification in docs/arch/system"
    );

    let summaries = list_domain_specs(&repo_root).expect("should list domain specs");
    assert!(
        summaries.iter().any(|s| s.domain == "security"),
        "expected domain 'security' to be found in docs/arch/system"
    );
}
