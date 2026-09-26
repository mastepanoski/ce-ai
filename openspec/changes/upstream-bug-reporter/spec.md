# Spec: Intelligent Upstream Bug Detection, Deduplication & Reporting Requirements

## Functional Requirements

### REQ-1: Triage Domain Boundary
- **WHEN** an error or test failure occurs within host application code or business logic,
- **THEN** it MUST be triaged locally via Compound Engineering (`ce-debug` ➔ `ce-plan` ➔ `ce-work`) and MUST NOT be submitted upstream to `mastepanoski/ce-ai`.
- **WHEN** an internal `ce-ai` error occurs (CLI crash, unhandled `CeError`, panic, state desync, harness sync failure),
- **THEN** it qualifies for upstream diagnostic compilation and reporting.

### REQ-2: Zero-Data-Leakage Privacy Sanitization (ISO 27001)
- **WHEN** compiling diagnostic bundles or error traces,
- **THEN** the sanitization filter MUST:
  1. Replace user home directory paths with `~`.
  2. Replace current workspace / repository paths with `<project-root>`.
  3. Redact GitHub personal access tokens (`ghp_*`, `github_pat_*`).
  4. Redact authorization bearer tokens (`Bearer ...`).
  5. Redact generic API keys (`*_API_KEY`, `*_SECRET`, `*_TOKEN`).
  6. Redact private cryptographic keys (`BEGIN ... PRIVATE KEY`).
  7. Exclude all host application source files and customer data.

### REQ-3: Deduplication Query
- **WHEN** `ce-ai report-bug` runs or error reporting is triggered,
- **THEN** the system MUST query `mastepanoski/ce-ai` using an error fingerprint or search query.
- **WHEN** an existing issue matches the defect,
- **THEN** the system MUST display the existing issue URL and title, and offer to view or add a diagnostic comment instead of opening a duplicate issue.

### REQ-4: User Sovereignty & Explicit Consent Gate
- **WHEN** preparing to submit an issue,
- **THEN** the system MUST NEVER transmit data or create issues silently in the background.
- **WHEN** running in an interactive terminal,
- **THEN** the system MUST prompt the user with explicit choices: submit, preview sanitized report, open pre-filled browser URL, or dismiss.
- **WHEN** the user selects dismiss or ignores the prompt,
- **THEN** the command MUST exit cleanly with zero network transmission.

### REQ-5: GitHub CLI (`gh`) Verification & Fallback Guidance
- **WHEN** `gh auth status` indicates GitHub CLI is installed and authenticated,
- **THEN** direct submission via `gh issue create` MUST be offered.
- **WHEN** `gh` is not installed or unauthenticated,
- **THEN** platform-specific installation instructions (macOS: `brew install gh`, Linux: `sudo apt install gh`, Windows: `winget install --id GitHub.cli`) and authentication instructions (`gh auth login`) MUST be displayed.
- **WHEN** `gh` is unavailable or the user chooses web submission,
- **THEN** a pre-filled, properly URL-encoded GitHub web URL MUST be generated and provided.

### REQ-6: Subcommand Interface
- **WHEN** `ce-ai report-bug` is invoked with `--dry-run`,
- **THEN** the sanitized markdown draft MUST be printed to stdout without prompting or submitting.
- **WHEN** `ce-ai report-bug` is invoked with `--json`,
- **THEN** the diagnostic bundle MUST be emitted as machine-readable JSON.
- **WHEN** `ce-ai report-bug` is invoked with `--web`,
- **THEN** the pre-filled web URL MUST be opened in the default browser (or printed if headless).
