# Exploration: Phase 2 — CE Compatibility Layer & Fleet Subsystem

## Context & Investigation

In CE-AI v1, package management and upstream synchronization were tightly bound to raw tarball downloads from GitHub Releases. When running `ce-ai install` or `ce-ai sync`:
1. The binary fetched `compound-engineering-<tag>.tar.gz`.
2. It extracted the entire `skills/` tree into `~/.config/opencode/compound-engineering/` or harness config dirs.
3. It calculated SHA256 hashes of every file and maintained a `manifest.json`.

Upstream Compound Engineering maintainers pointed out that this approach conflicts directly with host native package managers:
- Claude Code uses `claude plugin` marketplace/install commands.
- OpenCode uses `opencode.json` plugin configurations.
- Pi uses `.pi/extensions/` npm modules.
- Copying files bypasses the Bun converter and native transformations, causing duplicates and broken paths.

## Evaluated Alternatives

### Option A: Retain File Scraping with Smarter Deduplication
- **Concept**: Continue downloading tarballs and copying files, but attempt to detect when a native plugin is also present and suppress errors.
- **Trade-offs**:
  - *Pros*: Minimal code changes to existing `src/source/archive.rs` and `src/state/diff.rs`.
  - *Cons*: Still maintains dual sources of truth; does not address the Bun converter transformations; continues to suffer from `external-duplicate` confusion.
- **Conclusion**: **Ruled Out**. Violates the architectural principle of letting hosts own native execution and packaging.

### Option B: Raw CLI Shell-Out Only
- **Concept**: Rely entirely on executing `claude plugin install`, `npm install -g`, etc., with no local state or metadata inspection.
- **Trade-offs**:
  - *Pros*: Extremely simple wrappers.
  - *Cons*: Fails completely in non-interactive environments, CI runners, or when the host CLI binary is not globally on `PATH`. Provides no offline inspection, dry-run capability, or corporate pinning auditing.
- **Conclusion**: **Ruled Out**. Lacks resilience, cross-platform portability, and auditability.

### Option C (Selected): Hybrid Native Driver Trait Architecture (`src/fleet/`)
- **Concept**: Define a strongly-typed `FleetHarnessDriver` trait implemented for each supported harness:
  - **Inspection**: Inspects host-native configuration files (`claude.json`, `settings.json`, `opencode.json`, `.pi/`) to determine the installed CE plugin version without requiring a live subprocess invocation.
  - **Execution / Action Planning**: If a version divergence or missing plugin is detected, generates a declarative `FleetAction` (e.g., RunCommand, UpdateConfig) that can be previewed via `--dry-run` or executed.
  - **Centralized Version Pinning**: Stores a single `pinned_version` in `state.json` under `fleet.pinned_version`, allowing fleet-wide compliance checks.
- **Trade-offs**:
  - *Pros*: Works offline and in CI; supports `--dry-run` previews; respects native configuration schemas; gives developers a unified status dashboard across all agents.
  - *Cons*: Requires maintaining driver adapters for each supported host.
- **Conclusion**: **Selected**. Aligns cleanly with the v2 PRD and provides clear separation between observation and execution.

## Architectural Tradeoffs for Compatibility Layer (`src/compat/`)

Previously, `resolve_docs_root`, `clean_code_path`, and `check_solution_frontmatter` were implemented directly inside `src/commands/workflow.rs` and `src/commands/doc.rs`.
Moving them into `src/compat/`:
- **Encapsulation**: Keeps all upstream assumptions in a single compile-time module.
- **Testability**: Independent unit tests for schema changes without needing dummy CLI commands.
- **Reusability**: Accessible by both `doc`, `workflow`, `doctor`, and future fleet commands.
