# Proposal: Phase 2 — CE Compatibility Layer & Fleet Subsystem

## Problem Statement

Historically, `ce-ai` operated as an ad-hoc tarball package unpacker and file synchronizer. It downloaded the upstream release tarball (`EveryInc/compound-engineering-plugin`), recursively extracted the `skills/` directory, and copied files into each AI harness configuration folder while checking per-file SHA256 hashes.

This approach produced major architectural friction highlighted during upstream design review:
1. **Bypassing Native Packaging**: Every supported host (Claude Code, OpenCode, Pi, Codex, Cursor) possesses or is moving toward native plugin and marketplace packaging. Raw file copying bypasses host-specific packaging transformations (e.g., the Bun converter) and generates conflicting `external-duplicate` warnings.
2. **Scattered Upstream Assumptions**: Path resolution (`docs/plans/`, `docs/solutions/`), solution frontmatter schemas, and skill contracts were scattered as heuristics across individual commands (`workflow.rs`, `doc.rs`, `init_prj.rs`), leading to brittle coupling bugs when upstream evolved.
3. **Lack of Centralized Version Governance**: Developers working across multiple AI coding agents lacked a single tool to audit and pin the exact release version of Compound Engineering deployed across their entire toolchain.

## In-Scope

- **Strongly-Typed Upstream Compatibility Layer (`src/compat/`)**:
  - `src/compat/docs.rs`: Centralized `CeDocsConfig` discovering `docs_root` from `.compound-engineering/config.yaml` or `.local.yaml`, providing canonical paths for `plans_dir()`, `solutions_dir()`, and specification paths.
  - `src/compat/schema.rs`: Serde model `CeSolutionFrontmatter` validating upstream `schema.yaml` (`module`, `date`, `problem_type`, `component`, `severity`, optional `tags` and `applies_when`).
  - `src/compat/contracts.rs`: Typed skill invocation contracts, mode tokens (`mode:return-to-caller`), and canonical skill name catalog.
  - `src/compat/release.rs`: Upstream release tag and semver validation without mandatory tarball downloading.
- **Fleet Version Governance Subsystem (`src/fleet/`)**:
  - `FleetHarnessDriver` trait defining native package manager interactions (`detect_installed_version`, `install_version`, `update_version`).
  - Native drivers for supported harnesses:
    - **Claude Code**: Plugin registry / `claude plugin install/update` driver.
    - **OpenCode**: Extension registry driver via `opencode.json` plugin declarations.
    - **Pi (`pi.dev`)**: npm/extension driver targeting `.pi/extensions/`.
    - **Codex / Cursor**: Declarative configuration drivers.
- **CLI Commands (`ce-ai fleet`)**:
  - `ce-ai fleet status`: Tabular and JSON auditing of Compound Engineering versions installed across all active hosts.
  - `ce-ai fleet pin <version>`: Persist desired corporate or workspace CE version in `state.json`.
  - `ce-ai fleet sync`: Drive native installers across all active harnesses to match the pinned version.
- **Deprecation Notice**: Mark legacy raw tarball extraction (`ce-ai install` file-copy mode) with an advisory deprecation warning directing users to `ce-ai fleet`.

## Out-of-Scope

- Decommissioning legacy `ce-ai install` or `src/source/archive.rs` completely (deferred to Phase 4 for backwards compatibility).
- Deleting the FSM stage cursor from `state.json` (Phase 3 milestone).
- Rewriting the upstream Bun converter.

## Risk Evaluation

| Risk | Impact | Likelihood | Mitigation |
| :--- | :--- | :--- | :--- |
| Host CLI not in PATH | Medium | Moderate | Gracefully degrade `detect_installed_version` to report `unsupported-cli` or `unreachable` without aborting fleet operations. |
| Network latency fetching GitHub releases | Low | Moderate | Cache release metadata locally under `~/.config/ce-ai/cache/` with TTL. |
| Incompatible host plugin manager syntax | Medium | Low | Decouple each harness driver into isolated unit tests with mock subprocess runners. |

## Success Criteria

1. `src/compat/` centralizes `CeDocsConfig`, `CeSolutionFrontmatter`, `CeRelease`, and `CeSkillContract`, fully replacing duplicate path discovery logic in `src/commands/doc.rs` and `src/commands/workflow.rs`.
2. `ce-ai fleet status` reliably reports installed CE versions across Claude Code, OpenCode, Pi, and Codex in human-readable and `--json` formats.
3. `ce-ai fleet pin <version>` and `ce-ai fleet sync` update state and invoke native driver hooks cleanly.
4. 100% unit and CLI test passage with zero clippy warnings.
