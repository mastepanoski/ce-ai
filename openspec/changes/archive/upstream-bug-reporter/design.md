# Design: Intelligent Upstream Bug Detection, Deduplication & Reporting

## Architecture & Workflow Flowchart

```
                 [Error / ce-ai report-bug]
                             │
                             ▼
         [Domain Check: Is this a host app bug?]
               /                           \
         (Yes: Host App)             (No: ce-ai Internal)
             │                               │
             ▼                               ▼
    [Route to Compound            [Compile Diagnostic Bundle]
     Engineering: ce-debug]        - ce-ai version & git commit
                                   - OS & CPU Architecture
                                   - Target Harness
                                   - Sanitized Error / Backtrace
                                             │
                                             ▼
                               [Multi-Pass Redaction Filter]
                               - Strip API keys, tokens, secrets
                               - Anonymize home paths (~)
                               - Anonymize project paths (<project-root>)
                                             │
                                             ▼
                             [Deduplication Query to mastepanoski/ce-ai]
                                             │
                                 ┌───────────┴───────────┐
                             (Match Found)          (No Match)
                                 │                       │
                                 ▼                       ▼
                        [Present Match & Link]    [Draft Bug Report]
                                 │                       │
                                 └───────────┬───────────┘
                                             │
                                             ▼
                               [User Consent & Options Prompt]
                               [1] Submit to GitHub
                               [2] Open prefilled browser URL
                               [3] Show sanitized draft
                               [4] Dismiss / Ignore
                                             │
                                             ▼
                                     [Action Executed]
```

## Data Structures

### 1. Diagnostic Bundle (`BugReportBundle`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BugReportBundle {
    pub ce_version: String,
    pub os: String,
    pub arch: String,
    pub target_harness: String,
    pub command_invoked: String,
    pub error_message: String,
    pub logs_sanitized: String,
}
```

### 2. Formatted Markdown Issue Body (`format_github_issue_body`)
Conforms to `.github/ISSUE_TEMPLATE/bug_report.yml`:
```markdown
### Bug Description
<error_message>

### Operating System
<os> (<arch>)

### Target Harness
<target_harness>

### Steps To Reproduce
1. Invoked command: `<command_invoked>`
2. Encountered error: `<error_message>`

### Expected Behavior
Command completes successfully without errors or crashes.

### CLI Logs & Error Output
```shell
<logs_sanitized>
```
```

### 3. Redaction Engine (`sanitize_text`)
- Replaces absolute home directory with `~`.
- Replaces workspace root directory with `<project-root>`.
- Redacts GitHub tokens (`ghp_*`, `github_pat_*`).
- Redacts generic API keys, tokens, and authorization bearer tokens.
- Redacts private key blocks.

### 4. GitHub CLI (`gh`) Integration (`GhCliHelper`)
- `check_gh_status() -> GhStatus`: Returns `Ready`, `Unauthenticated`, or `NotInstalled`.
- `install_instructions() -> &'static str`: Returns platform-specific installation command (`brew install gh`, `sudo apt install gh`, `winget install --id GitHub.cli`).
- `generate_web_issue_url(title: &str, body: &str) -> String`: Generates URL encoded `https://github.com/mastepanoski/ce-ai/issues/new?title=...&body=...`.

### 5. CLI Command Interface: `ce-ai report-bug`
```
Usage: ce-ai report-bug [OPTIONS]

Options:
      --title <TITLE>       Optional custom issue title
      --error <ERROR>       Specific error message or trace to report
      --harness <HARNESS>   Target harness affected (opencode, claude, cursor, etc.)
      --dry-run             Preview sanitized report without submitting or prompting
      --json                Output diagnostic bundle as JSON
      --web                 Open prefilled issue in browser directly
  -y, --yes                 Skip confirmation and submit directly via gh if ready
  -h, --help                Print help
```

## Configuration in `state.json`
```json
{
  "bug_reporter": {
    "enabled": true,
    "auto_prompt": true
  }
}
```
Environment variables:
- `CE_NO_BUG_REPORT=1` or `CE_DISABLE_BUG_PROMPT=1`: Suppress automatic prompt after unhandled error.
