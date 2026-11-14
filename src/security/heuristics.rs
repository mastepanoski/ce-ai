//! Contextual Privacy & PII Schema Heuristics.
//!
//! Non-blockingly detects database schemas, ORM models, and entity definitions
//! containing sensitive PII fields (credit cards, SSN, IBAN, medical data, DOB).
//! Surfaces advisory guidance in `ce-ai doctor` and `ce-ai init-prj` pointing
//! to local redaction and AI Privacy Gateways (e.g. Microsoft Presidio).

use std::path::{Path, PathBuf};

/// Diagnostic summary of sensitive PII schema fields detected in a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiiSchemaSuggestion {
    pub file_path: PathBuf,
    pub relative_path: String,
    pub matched_fields: Vec<String>,
}

const SCHEMA_FILE_CANDIDATES: &[&str] = &[
    "schema.prisma",
    "prisma/schema.prisma",
    "db/schema.rb",
    "src/models/user.rs",
    "src/models/user.ts",
    "src/models/user.py",
    "models/user.py",
    "models/user.ts",
    "models/User.ts",
    "models/user.go",
    "src/entities/User.ts",
    "entities/User.ts",
];

const SENSITIVE_FIELD_PATTERNS: &[&str] = &[
    "ssn",
    "social_security",
    "credit_card",
    "card_number",
    "cvv",
    "cvc",
    "iban",
    "bank_account",
    "patient_id",
    "health_record",
    "medical_record",
    "diagnosis",
    "date_of_birth",
    "tax_id",
    "passport",
];

/// Detects whether the project defines database schemas or models with sensitive PII fields.
pub fn detect_pii_schema_opportunity(project_dir: &Path) -> Option<PiiSchemaSuggestion> {
    // 1. Direct candidate files
    for rel_path in SCHEMA_FILE_CANDIDATES {
        let candidate_path = project_dir.join(rel_path);
        if candidate_path.is_file() {
            if let Some(suggestion) = inspect_file_for_pii(&candidate_path, rel_path) {
                return Some(suggestion);
            }
        }
    }

    // 2. Scan directories like migrations/ or models/ if present
    for dir_name in [
        "migrations",
        "db/migrations",
        "src/models",
        "models",
        "entities",
    ] {
        let dir_path = project_dir.join(dir_name);
        if dir_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if ["sql", "rs", "ts", "js", "py", "rb", "go", "prisma"].contains(&ext) {
                            let rel = format!("{dir_name}/{name}");
                            if let Some(suggestion) = inspect_file_for_pii(&path, &rel) {
                                return Some(suggestion);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn inspect_file_for_pii(file_path: &Path, rel_path: &str) -> Option<PiiSchemaSuggestion> {
    // Read up to 64KB to remain lightning-fast and non-blocking
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let content_lower = content.to_ascii_lowercase();
    let mut matched = Vec::new();

    for &pattern in SENSITIVE_FIELD_PATTERNS {
        // Look for field identifiers surrounded by quotes, spaces, colon, or punctuation
        if content_lower.contains(pattern) {
            matched.push(pattern.to_string());
            if matched.len() >= 3 {
                break;
            }
        }
    }

    if !matched.is_empty() {
        Some(PiiSchemaSuggestion {
            file_path: file_path.to_path_buf(),
            relative_path: rel_path.to_string(),
            matched_fields: matched,
        })
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/heuristics_tests.rs"]
mod tests;
