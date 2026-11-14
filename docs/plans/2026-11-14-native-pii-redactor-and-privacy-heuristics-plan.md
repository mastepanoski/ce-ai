# Plan: Native PII & Secrets Redaction Engine and Privacy Schema Heuristics

## Overview
Implement a high-performance, zero-Python, sub-millisecond native Rust PII and secrets redaction engine in `src/security/`, providing mathematical checksum validation (Luhn for credit cards, ISO 7064 mod 97-10 for IBANs, E.164 phone formats, RFC 5322 emails, JWT/Bearer/PAT tokens, and PEM private keys). Unify redaction across `src/decisions/risk.rs` and `src/source/bug_reporter.rs`. Additionally, add contextual privacy schema heuristics in `ce-ai doctor` and `ce-ai init-prj` to non-blockingly advise teams handling sensitive PII on local redaction and AI Privacy Gateways (e.g. Microsoft Presidio).

## Tasks

### Phase 1: Native Security Engine (`src/security/mod.rs` & `src/security/pii.rs`)
- [x] Create `src/security/mod.rs` and declare `pub mod pii;` and `pub mod heuristics;`. Expose `pub mod security;` in `src/lib.rs`.
- [x] Implement mathematical validation:
  - `is_luhn_valid(number: &str) -> bool`: Validates 13-19 digit card numbers using the standard Luhn (mod 10) algorithm with card brand prefix awareness (Visa, Mastercard, Amex, Discover, Diners, JCB).
  - `is_iban_valid(iban: &str) -> bool`: Validates ISO 13616 / ISO 7064 IBAN format and mod 97-10 checksum algorithm.
- [x] Implement native string scanner & redactor in `src/security/pii.rs`:
  - Credit cards (`[REDACTED_CREDIT_CARD]`) with Luhn confirmation.
  - IBANs (`[REDACTED_IBAN]`) with mod 97-10 confirmation.
  - Emails (`[REDACTED_EMAIL]`) matching standard email patterns.
  - International & formatted phone numbers (`[REDACTED_PHONE]`).
  - GitHub PATs (`ghp_`, `github_pat_`, etc.), OpenAI/Anthropic keys (`sk-`), AWS access keys (`AKIA...`), and generic tokens (`[REDACTED]`).
  - JWT tokens (`[REDACTED_JWT]`) and Bearer headers.
  - PEM private keys (`[REDACTED_PRIVATE_KEY]`).
  - Path variants & user home directory anonymization (`~` and `<project-root>`).
- [x] Comprehensive unit tests for `pii.rs` covering valid/invalid Luhn cards, valid/invalid IBANs, emails, phones, tokens, private keys, and path variants.

### Phase 2: Unify Redaction in Decisions & Bug Reporter
- [x] Refactor `src/decisions/risk.rs` to delegate `redact_sensitive_content` to `crate::security::pii::redact_pii_and_secrets`.
- [x] Refactor `src/source/bug_reporter.rs` to delegate `sanitize_text` to `crate::security::pii::redact_pii_and_secrets_with_paths`.
- [x] Verify existing tests in `src/decisions/risk.rs` and `src/source/bug_reporter.rs` continue to pass without regression.

### Phase 3: Privacy & PII Schema Heuristics (`src/security/heuristics.rs`)
- [x] Implement `detect_pii_schema_opportunity(project_dir: &Path) -> Option<PiiSchemaSuggestion>`:
  - Scans schema and model files (`schema.prisma`, `db/schema.rb`, `migrations/`, `models/`, `entities/`, `src/models/`, `types/`).
  - Detects sensitive fields (SSN, credit card, patient ID, medical/diagnosis, tax ID, IBAN, DOB).
  - Returns structured recommendation advising local redaction or enterprise AI Privacy Gateway (Presidio).
- [x] Wire heuristic probe into `ce-ai doctor` as a non-blocking `doctor-info: privacy:` finding.
- [x] Wire heuristic hint into `ce-ai init-prj` adoption summary when sensitive models are detected.
- [x] Unit tests for privacy heuristics in `src/security/heuristics.rs`, `src/commands/tests/doctor.rs`, and `src/commands/tests/init_prj.rs`.

### Phase 4: Verification & Definition of Done
- [x] Add CLI integration test in `tests/cli.rs` validating doctor and risk redaction behavior.
- [x] Verify `cargo fmt --check`.
- [x] Verify `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Verify `cargo test`.
- [x] Run `scripts/validate-concepts.py` after accreting `Native PII Redaction Engine` and `Contextual Privacy Schema Heuristics` into `CONCEPTS.md`.
- [x] Update `CHANGELOG.md` with features and compliance notes.
- [x] Bump version in `Cargo.toml` (`3.12.0`) and sync `Cargo.lock`.
