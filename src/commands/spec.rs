//! Living system specifications and delta promotion engine (`ce-ai spec`).
//!
//! Manages evergreen domain contracts in `openspec/specs/` and promotes validated
//! requirements from transient delta change specifications (`openspec/changes/<feat>/`).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::commands::Context;
use crate::error::CeError;

#[derive(Args, Debug, Clone)]
pub struct SpecArgs {
    #[command(subcommand)]
    pub command: SpecCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SpecCommand {
    /// List all living domain specifications in openspec/specs/.
    List {
        /// Emit JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Show the complete specification for a specific domain.
    Show {
        /// Domain identifier (e.g. harnesses, doctor, workflow).
        domain: String,
    },
    /// Validate all domain specifications against schema rules.
    Validate {
        /// Emit JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Promote delta requirements from an OpenSpec change into its target domain specification.
    Promote {
        /// OpenSpec change name to promote from.
        change: String,
        /// Target domain spec override (defaults to domain in spec.md frontmatter).
        #[arg(long)]
        domain: Option<String>,
        /// Dry run preview without mutating disk assets.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainSpecMetadata {
    pub title: String,
    pub domain: String,
    pub version: String,
    pub last_updated: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainSpecSummary {
    pub domain: String,
    pub title: String,
    pub version: String,
    pub last_updated: String,
    pub requirement_count: usize,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecFindingSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecValidationFinding {
    pub file: PathBuf,
    pub domain: Option<String>,
    pub severity: SpecFindingSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecValidationReport {
    pub valid: bool,
    pub specs_count: usize,
    pub findings: Vec<SpecValidationFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromoteOutcome {
    pub change: String,
    pub domain: String,
    pub requirements_promoted: usize,
    pub spec_path: PathBuf,
    pub dry_run: bool,
}

/// Dispatches `ce-ai spec` subcommands.
pub fn run_spec(ctx: &Context, args: &SpecArgs) -> Result<(), CeError> {
    let repo_root = ctx.repo_root();
    match &args.command {
        SpecCommand::List { json } => run_spec_list(&repo_root, *json),
        SpecCommand::Show { domain } => run_spec_show(&repo_root, domain),
        SpecCommand::Validate { json } => run_spec_validate(&repo_root, *json),
        SpecCommand::Promote {
            change,
            domain,
            dry_run,
        } => run_spec_promote(&repo_root, change, domain.as_deref(), *dry_run),
    }
}

/// Lists all domain specifications in `openspec/specs/`.
pub fn list_domain_specs(repo_root: &Path) -> Result<Vec<DomainSpecSummary>, CeError> {
    let specs_dir = repo_root.join("openspec").join("specs");
    if !specs_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut summaries = Vec::new();
    let entries = fs::read_dir(&specs_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Ok(meta) = parse_domain_spec_metadata(&content) {
                let req_count = count_requirements(&content);
                summaries.push(DomainSpecSummary {
                    domain: meta.domain,
                    title: meta.title,
                    version: meta.version,
                    last_updated: meta.last_updated,
                    requirement_count: req_count,
                    file_path: path,
                });
            }
        }
    }

    summaries.sort_by(|a, b| a.domain.cmp(&b.domain));
    Ok(summaries)
}

fn run_spec_list(repo_root: &Path, json: bool) -> Result<(), CeError> {
    let summaries = list_domain_specs(repo_root)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&summaries)?);
        return Ok(());
    }

    if summaries.is_empty() {
        println!("spec: no domain specifications found in openspec/specs/");
        return Ok(());
    }

    println!(
        "{:<18} {:<8} {:<12} {:<6} TITLE",
        "DOMAIN", "VERSION", "UPDATED", "REQS"
    );
    println!("{:-<75}", "");
    for s in &summaries {
        println!(
            "{:<18} {:<8} {:<12} {:<6} {}",
            s.domain, s.version, s.last_updated, s.requirement_count, s.title
        );
    }
    println!("{:-<75}", "");
    println!("Total: {} domain specification(s)", summaries.len());

    Ok(())
}

fn run_spec_show(repo_root: &Path, domain: &str) -> Result<(), CeError> {
    let spec_path = get_domain_spec_path(repo_root, domain);
    if !spec_path.is_file() {
        return Err(CeError::Usage(format!(
            "domain specification '{domain}' not found in openspec/specs/"
        )));
    }

    let content = fs::read_to_string(&spec_path)?;
    print!("{content}");
    Ok(())
}

fn run_spec_validate(repo_root: &Path, json: bool) -> Result<(), CeError> {
    let report = validate_domain_specs(repo_root);

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        if !report.valid {
            return Err(CeError::Verification(
                "one or more domain specifications failed validation".to_string(),
            ));
        }
        return Ok(());
    }

    if report.specs_count == 0 {
        println!("spec validate: warning: openspec/specs/ is empty");
        return Ok(());
    }

    if report.valid {
        println!(
            "spec validate: all {} domain specification(s) valid",
            report.specs_count
        );
        Ok(())
    } else {
        eprintln!(
            "spec validate: {} validation finding(s) detected across {} specifications:",
            report.findings.len(),
            report.specs_count
        );
        for f in &report.findings {
            let level = match f.severity {
                SpecFindingSeverity::Error => "error",
                SpecFindingSeverity::Warning => "warn",
            };
            eprintln!(
                "  [{level}] {}: {}",
                f.file.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                f.message
            );
        }
        Err(CeError::Verification(
            "one or more domain specifications failed validation".to_string(),
        ))
    }
}

fn run_spec_promote(
    repo_root: &Path,
    change: &str,
    domain_override: Option<&str>,
    dry_run: bool,
) -> Result<(), CeError> {
    let outcome = promote_delta_to_domain_spec(repo_root, change, domain_override, dry_run)?;

    if dry_run {
        println!(
            "dry-run: would promote {} requirement(s) from change '{}' into domain spec '{}' ({})",
            outcome.requirements_promoted,
            outcome.change,
            outcome.domain,
            outcome.spec_path.display()
        );
    } else {
        println!(
            "spec promote: successfully promoted {} requirement(s) from '{}' into '{}' ({})",
            outcome.requirements_promoted,
            outcome.change,
            outcome.domain,
            outcome.spec_path.display()
        );
    }

    Ok(())
}

/// Returns the path to `openspec/specs/<domain>.md`.
pub fn get_domain_spec_path(repo_root: &Path, domain: &str) -> PathBuf {
    repo_root
        .join("openspec")
        .join("specs")
        .join(format!("{domain}.md"))
}

/// Extracts YAML frontmatter between `---` boundaries and remainder content.
pub fn extract_frontmatter(content: &str) -> Option<(&str, &str)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let rest = &trimmed[3..];
    let end_idx = rest.find("\n---")?;
    let yaml_str = &rest[..end_idx].trim();
    let body_str = &rest[end_idx + 4..].trim_start();
    Some((yaml_str, body_str))
}

/// Parses YAML frontmatter into `DomainSpecMetadata`.
pub fn parse_domain_spec_metadata(content: &str) -> Result<DomainSpecMetadata, String> {
    let (yaml_str, _) = extract_frontmatter(content)
        .ok_or_else(|| "missing YAML frontmatter delimiters (---)".to_string())?;

    let mut title = None;
    let mut domain = None;
    let mut version = None;
    let mut last_updated = None;
    let mut dependencies = Vec::new();

    for line in yaml_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            match key {
                "title" => title = Some(val),
                "domain" => domain = Some(val),
                "version" => version = Some(val),
                "last_updated" => last_updated = Some(val),
                "dependencies" => {
                    let inside = v.trim().trim_start_matches('[').trim_end_matches(']');
                    for item in inside.split(',') {
                        let dep = item.trim().trim_matches('"').trim_matches('\'');
                        if !dep.is_empty() {
                            dependencies.push(dep.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(DomainSpecMetadata {
        title: title.ok_or_else(|| "missing required field 'title'".to_string())?,
        domain: domain.ok_or_else(|| "missing required field 'domain'".to_string())?,
        version: version.unwrap_or_else(|| "1.0.0".to_string()),
        last_updated: last_updated.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string()),
        dependencies,
    })
}

/// Counts requirements formatted as `### R` or `### ` under requirements sections.
pub fn count_requirements(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let t = line.trim();
            t.starts_with("### R") || t.starts_with("### Requirement")
        })
        .count()
}

/// Validates all specifications in `openspec/specs/`.
pub fn validate_domain_specs(repo_root: &Path) -> SpecValidationReport {
    let specs_dir = repo_root.join("openspec").join("specs");
    let mut findings = Vec::new();

    if !specs_dir.is_dir() {
        return SpecValidationReport {
            valid: true,
            specs_count: 0,
            findings,
        };
    }

    let entries = match fs::read_dir(&specs_dir) {
        Ok(e) => e,
        Err(err) => {
            findings.push(SpecValidationFinding {
                file: specs_dir,
                domain: None,
                severity: SpecFindingSeverity::Error,
                message: format!("cannot read specs directory: {err}"),
            });
            return SpecValidationReport {
                valid: false,
                specs_count: 0,
                findings,
            };
        }
    };

    let mut specs_count = 0;
    let mut known_domains = HashSet::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            specs_count += 1;
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(err) => {
                    findings.push(SpecValidationFinding {
                        file: path.clone(),
                        domain: None,
                        severity: SpecFindingSeverity::Error,
                        message: format!("cannot read file: {err}"),
                    });
                    continue;
                }
            };

            match parse_domain_spec_metadata(&content) {
                Ok(meta) => {
                    if meta.domain != file_stem {
                        findings.push(SpecValidationFinding {
                            file: path.clone(),
                            domain: Some(meta.domain.clone()),
                            severity: SpecFindingSeverity::Error,
                            message: format!(
                                "frontmatter domain '{}' does not match filename '{file_stem}.md'",
                                meta.domain
                            ),
                        });
                    }
                    known_domains.insert(meta.domain.clone());
                }
                Err(err) => {
                    findings.push(SpecValidationFinding {
                        file: path.clone(),
                        domain: None,
                        severity: SpecFindingSeverity::Error,
                        message: format!("invalid frontmatter: {err}"),
                    });
                }
            }

            if !content.contains("## Capabilities & Requirements")
                && !content.contains("## 2. Capabilities & Requirements")
                && !content.contains("## Requirements")
            {
                findings.push(SpecValidationFinding {
                    file: path.clone(),
                    domain: Some(file_stem.to_string()),
                    severity: SpecFindingSeverity::Warning,
                    message: "missing section '## Capabilities & Requirements'".to_string(),
                });
            }
        }
    }

    let has_errors = findings
        .iter()
        .any(|f| f.severity == SpecFindingSeverity::Error);

    SpecValidationReport {
        valid: !has_errors,
        specs_count,
        findings,
    }
}

/// Extracts RFC 2119 requirement blocks from an OpenSpec `spec.md` file.
pub fn extract_requirements_from_spec_md(content: &str) -> Vec<(String, String)> {
    let mut requirements = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_body = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("### R") || trimmed.starts_with("### Requirement") {
            if let Some(title) = current_title.take() {
                let body = current_body.join("\n").trim().to_string();
                if !body.is_empty() {
                    requirements.push((title, body));
                }
                current_body.clear();
            }
            current_title = Some(trimmed.trim_start_matches('#').trim().to_string());
        } else if trimmed.starts_with("## ") && current_title.is_some() {
            if let Some(title) = current_title.take() {
                let body = current_body.join("\n").trim().to_string();
                if !body.is_empty() {
                    requirements.push((title, body));
                }
                current_body.clear();
            }
        } else if current_title.is_some() {
            current_body.push(line);
        }
    }

    if let Some(title) = current_title {
        let body = current_body.join("\n").trim().to_string();
        if !body.is_empty() {
            requirements.push((title, body));
        }
    }

    requirements
}

/// Resolves the target domain for a change folder.
pub fn resolve_change_target_domain(
    repo_root: &Path,
    change_name: &str,
    override_domain: Option<&str>,
) -> Result<String, CeError> {
    if let Some(dom) = override_domain {
        return Ok(dom.to_string());
    }

    // Check active changes
    let active_spec = repo_root
        .join("openspec")
        .join("changes")
        .join(change_name)
        .join("spec.md");

    let target_spec = if active_spec.is_file() {
        active_spec
    } else {
        // Fallback to archive
        repo_root
            .join("openspec")
            .join("changes")
            .join("archive")
            .join(change_name)
            .join("spec.md")
    };

    if !target_spec.is_file() {
        return Err(CeError::Usage(format!(
            "cannot resolve spec.md for change '{change_name}'"
        )));
    }

    let content = fs::read_to_string(&target_spec)?;
    if let Ok(meta) = parse_domain_spec_metadata(&content) {
        return Ok(meta.domain);
    }

    // Attempt simple regex / key scan if full metadata parse fails
    if let Some((yaml_str, _)) = extract_frontmatter(&content) {
        for line in yaml_str.lines() {
            if let Some((k, v)) = line.split_once(':') {
                if k.trim() == "domain" {
                    let d = v.trim().trim_matches('"').trim_matches('\'');
                    if !d.is_empty() {
                        return Ok(d.to_string());
                    }
                }
            }
        }
    }

    Err(CeError::Usage(format!(
        "change '{change_name}' does not specify 'domain: <name>' in spec.md frontmatter; use --domain <name> to specify target"
    )))
}

/// Promotes delta requirements from a change into its target domain specification.
pub fn promote_delta_to_domain_spec(
    repo_root: &Path,
    change_name: &str,
    domain_override: Option<&str>,
    dry_run: bool,
) -> Result<PromoteOutcome, CeError> {
    let target_domain = resolve_change_target_domain(repo_root, change_name, domain_override)?;

    let active_spec = repo_root
        .join("openspec")
        .join("changes")
        .join(change_name)
        .join("spec.md");

    let source_spec = if active_spec.is_file() {
        active_spec
    } else {
        repo_root
            .join("openspec")
            .join("changes")
            .join("archive")
            .join(change_name)
            .join("spec.md")
    };

    let spec_content = fs::read_to_string(&source_spec)?;
    let requirements = extract_requirements_from_spec_md(&spec_content);

    if requirements.is_empty() {
        return Err(CeError::Usage(format!(
            "change '{change_name}' spec.md contains no extractable requirements (### R...)"
        )));
    }

    let domain_spec_path = get_domain_spec_path(repo_root, &target_domain);
    let today = Utc::now().format("%Y-%m-%d").to_string();

    let mut existing_content = if domain_spec_path.is_file() {
        fs::read_to_string(&domain_spec_path)?
    } else {
        // Initialize template
        format!(
            "---\ntitle: \"{} Domain Specification\"\ndomain: {}\nversion: 1.0.0\nlast_updated: \"{}\"\n---\n\n# Specification: {}\n\n## 1. Overview & Architectural Boundaries\n\nLiving specification for the {} subsystem.\n\n## 2. Capabilities & Requirements\n\n",
            target_domain, target_domain, today, target_domain, target_domain
        )
    };

    // Update last_updated date in frontmatter
    if let Some((yaml, body)) = extract_frontmatter(&existing_content) {
        let mut new_yaml_lines = Vec::new();
        for line in yaml.lines() {
            if line.trim_start().starts_with("last_updated:") {
                new_yaml_lines.push(format!("last_updated: \"{today}\""));
            } else {
                new_yaml_lines.push(line.to_string());
            }
        }
        existing_content = format!("---\n{}\n---\n\n{}", new_yaml_lines.join("\n"), body);
    }

    // Append promoted requirements under capabilities section
    let mut additions = String::new();
    for (title, body) in &requirements {
        // Only append if not already in document
        if !existing_content.contains(title) {
            additions.push_str(&format!(
                "\n### {}\n<!-- promoted-from: change:{} date:{} -->\n{}\n",
                title, change_name, today, body
            ));
        }
    }

    let updated_content = if additions.is_empty() {
        existing_content
    } else {
        let mut final_content = existing_content;
        final_content.push_str(&additions);
        final_content
    };

    if !dry_run {
        crate::state::write_atomic(&domain_spec_path, updated_content.as_bytes())?;
    }

    Ok(PromoteOutcome {
        change: change_name.to_string(),
        domain: target_domain,
        requirements_promoted: requirements.len(),
        spec_path: domain_spec_path,
        dry_run,
    })
}

/// Probes living domain specifications health for `ce-ai doctor`.
pub fn probe_specs_health(repo_root: &Path) -> Vec<String> {
    let mut warnings = Vec::new();
    let specs_dir = repo_root.join("openspec").join("specs");

    if !specs_dir.is_dir() {
        warnings.push("openspec/specs/ directory does not exist".to_string());
        return warnings;
    }

    let report = validate_domain_specs(repo_root);
    if report.specs_count == 0 {
        warnings.push(
            "openspec/specs/ is empty — run 'ce-ai spec validate' or seed domain specs".to_string(),
        );
    } else {
        for f in &report.findings {
            let name = f.file.file_name().and_then(|n| n.to_str()).unwrap_or("");
            warnings.push(format!("domain spec '{name}': {}", f.message));
        }
    }

    // Check active unarchived changes for domain frontmatter
    let changes_dir = repo_root.join("openspec").join("changes");
    if let Ok(entries) = fs::read_dir(&changes_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if p.is_dir() && name != "archive" && !name.starts_with('.') {
                let spec_path = p.join("spec.md");
                if spec_path.is_file() {
                    if let Ok(content) = fs::read_to_string(&spec_path) {
                        if extract_frontmatter(&content).is_none()
                            || parse_domain_spec_metadata(&content).is_err()
                        {
                            warnings.push(format!(
                                "active change '{name}' lacks target domain mapping in spec.md"
                            ));
                        }
                    }
                }
            }
        }
    }

    warnings
}

#[cfg(test)]
#[path = "tests/spec.rs"]
mod tests;
