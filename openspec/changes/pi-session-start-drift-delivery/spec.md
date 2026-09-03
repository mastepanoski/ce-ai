# Specification: Guaranteed Turn-0 Drift Delivery for Pi Coding Agent

## Requirements

### Requirement 1: Native Pi Extension Management
- **WHEN** `ensure_session_start_hook` is called with the target path to `.pi/extensions/compound-engineering.ts`,
- **THEN** it MUST ensure the file exists containing the canonical TypeScript extension subscribing to `session_start` and `before_agent_start`, executed atomically via `write_atomic`.
- **WHEN** the extension file already exists and contains `ce-ai workflow resume`,
- **THEN** `ensure_session_start_hook` MUST return `Ok(false)` without rewriting the file.

### Requirement 2: Surgical Deinstallation
- **WHEN** `remove_session_start_hook` is called on a project containing `.pi/extensions/compound-engineering.ts`,
- **THEN** it MUST remove `compound-engineering.ts` if it contains the managed hook code.
- **WHEN** `.pi/extensions` and `.pi` become empty as a result,
- **THEN** it MUST prune the empty directories cleanly.

### Requirement 3: Health Diagnostics in `ce-ai doctor`
- **WHEN** an adopted project in `state.projects` contains a `.pi` directory,
- **THEN** `ce-ai doctor` MUST verify that `.pi/extensions/compound-engineering.ts` exists and contains the valid hook code.
- **WHEN** the extension is missing or unconfigured,
- **THEN** `ce-ai doctor` MUST emit a finding prompting the user to re-run `ce-ai init-prj`.
