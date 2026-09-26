# Exploration: Intelligent Upstream Bug Detection, Deduplication & Reporting

## Technical Investigation

### 1. Existing Infrastructure
- **Exit Codes (`src/error.rs`)**:
  - `CeError::Usage` (exit 2)
  - `CeError::Runtime` (exit 1)
  - `CeError::State` (exit 3)
  - `CeError::Io` (exit 4)
  - `CeError::Network` (exit 5)
  - `CeError::Verification` (exit 6)
  Internal errors that warrant upstream reporting are typically `Runtime`, `State`, `Verification`, unexpected `Io`, or panics.
- **GitHub API Integration (`src/source/binary_release.rs`, `src/source/release.rs`)**:
  - `CLI_REPO`: `"mastepanoski/ce-ai"`.
  - Token resolution: `resolve_github_token()` inspects `CE_AI_GITHUB_TOKEN`, `GITHUB_TOKEN`, `GH_TOKEN`.
- **System Metadata (`std::env::consts`)**:
  - `OS` (`linux`, `macos`, `windows`)
  - `ARCH` (`x86_64`, `aarch64`, etc.)
- **Issue Template**:
  - `.github/ISSUE_TEMPLATE/bug_report.yml` defines the canonical schema:
    - Bug Description
    - Operating System
    - Target Harness
    - Steps To Reproduce
    - Expected Behavior
    - CLI Logs & Error Output

### 2. Evaluated Options

#### Option A: Automatic Silent Issue Creation in Background
- Description: Automatically POST to GitHub API upon unhandled error.
- Pros: Frictionless data collection.
- Cons: **Critical violation of ISO/IEC 27001**, GDPR, user sovereignty, and NIST AI RMF. Potential leakage of sensitive error strings without user inspection; creates spam.
- Decision: **Rejected**. Explicit user consent and review are mandatory.

#### Option B: Standalone Web Link Generator Only
- Description: Only print a static link to GitHub.
- Pros: Simple to implement.
- Cons: Does not deduplicate issues, does not compile structured environment bundles or sanitize paths/secrets, and requires manual copy-pasting into issue forms.
- Decision: **Rejected**.

#### Option C: Intelligent Triage Engine + gh CLI Integration + Sanitized Web Fallback (Selected)
- Description:
  1. Compile sanitized diagnostic bundle (OS, arch, ce-ai version, target harness, sanitized error trace).
  2. Redact sensitive patterns (secrets, tokens, absolute home/project paths).
  3. Query `mastepanoski/ce-ai` to detect duplicates. If a matching issue is found, offer to link or add diagnostics.
  4. Prompt user with choices (submit, edit, open web URL, or dismiss).
  5. Check `gh auth status`; if present and authenticated, allow direct submission via `gh issue create`. If not, provide installation instructions and open a pre-filled browser submission URL.
- Pros:
  - Full user sovereignty (zero silent network transmissions).
  - Robust privacy and ISO 27001 data protection.
  - Reduces duplicate issue noise.
  - Zero-friction web fallback when `gh` is absent.
- Decision: **Accepted**.

## Redaction Strategy (ISO 27001 & NIST AI RMF)
1. **Home Directory Anonymization**: Replace any instance of `home_dir` with `~`.
2. **Project Root Anonymization**: Replace any instance of `workspace_root` with `<project-root>`.
3. **Secret Redaction**:
   - `ghp_[A-Za-z0-9]{36}` -> `[REDACTED_GH_TOKEN]`
   - `github_pat_[A-Za-z0-9_]{82}` -> `[REDACTED_GH_TOKEN]`
   - `Bearer\s+[A-Za-z0-9\-_\.]+` -> `Bearer [REDACTED]`
   - `(?:api[_-]?key|secret|token|password)\s*[:=]\s*["']?([A-Za-z0-9_\-\.]{8,})["']?` -> `[REDACTED]`
   - Private key blocks -> `[REDACTED_PRIVATE_KEY]`
