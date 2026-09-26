# Tasks: Intelligent Upstream Bug Detection, Deduplication & Reporting

## Work Unit 1: Privacy Redaction & Diagnostic Bundle Engine
- [x] Create `src/source/bug_reporter.rs` defining `BugReportBundle`. (~40 LOC)
- [x] Implement multi-pass sanitization engine `sanitize_text` (home path anonymization, project root anonymization, token/bearer/key/secret redaction). (~70 LOC)
- [x] Implement `format_github_issue_body` matching `.github/ISSUE_TEMPLATE/bug_report.yml` and `generate_web_issue_url`. (~40 LOC)
- [x] Add unit tests in `src/source/tests/bug_reporter.rs` validating redactions, formatting, and URL encoding. (~80 LOC)
*Estimated changed lines: ~230 LOC*

## Work Unit 2: GitHub CLI (`gh`) Integration & Upstream Deduplication
- [x] Implement `check_gh_status`, `install_instructions`, and `submit_issue_via_gh`. (~60 LOC)
- [x] Implement `search_upstream_issues` to query `mastepanoski/ce-ai` for duplicates by error fingerprint. (~60 LOC)
- [x] Add unit tests in `src/source/tests/bug_reporter.rs` for `gh` status parsing, search deduplication, and install commands. (~50 LOC)
*Estimated changed lines: ~170 LOC*

## Work Unit 3: CLI Subcommand `ce-ai report-bug` & Dispatch
- [x] Create `src/commands/report_bug.rs` with `Args` supporting `--title`, `--error`, `--harness`, `--dry-run`, `--json`, `--web`, `-y`. (~110 LOC)
- [x] Wire interactive consent prompt (`[1] Submit`, `[2] Browser URL`, `[3] View draft`, `[4] Dismiss`) with non-interactive detection. (~50 LOC)
- [x] Register `report-bug` in `src/commands/registry.rs` and `src/commands/mod.rs`. (~20 LOC)
*Estimated changed lines: ~180 LOC*

## Work Unit 4: Agent Directives, CLI Integration Tests & Quality Gates
- [x] Update `AGENTS.md` and `CONCEPTS.md` defining the host project vs internal tool bug triage boundary. (~30 LOC)
- [x] Add CLI integration tests in `tests/cli.rs` verifying `ce-ai report-bug --dry-run`, `--json`, and path/token redaction. (~80 LOC)
- [x] Verify `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and full `cargo test`. (~20 LOC)
*Estimated changed lines: ~130 LOC*

*Total PR forecast: ~710 LOC*
