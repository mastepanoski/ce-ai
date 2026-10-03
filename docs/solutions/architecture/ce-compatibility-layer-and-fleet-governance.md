---
module: architecture::v2::fleet
tags: [architecture, fleet, compat, v2, drivers, schema]
problem_type: architecture
title: "CE Compatibility Layer and Fleet Version Governance Subsystem"
applies_when: "When coordinating multi-agent coding environments across heterogeneous AI coding hosts with native package managers and centralized upstream CE contracts."
date: 2026-10-03
component: fleet
related_components: [compat, harness, cli]
severity: standard
---

# CE Compatibility Layer and Fleet Version Governance Subsystem

## Context & Problem Statement

In `ce-ai` v1, plugin distribution and version management relied on scraping raw release tarballs from GitHub and extracting skill files directly into host configuration directories (`~/.config/opencode/`, `~/.claude/`, etc.). 

During an architectural boundary review, the upstream maintainer of Compound Engineering highlighted critical architectural friction caused by this approach:
1. **Bypassing Host Native Packaging:** Every supported AI coding host (Claude Code, OpenCode, Pi, Codex, Cursor) possesses its own native plugin or marketplace distribution mechanism. Extracting skills directly from release tarballs bypassed per-target rewrites performed by native packaging, producing `external-duplicate` warnings and leaving hosts with non-standard file structures.
2. **Fragmented Upstream Schema Assumptions:** Hardcoded assumptions regarding documentation paths (`docs/plans/`, `docs/solutions/`) and solution frontmatter rules were scattered across multiple commands (`src/commands/workflow.rs`, `src/commands/doc.rs`), causing desync with upstream rules (e.g. requiring `applies_when` for bug-track docs when upstream treats it as knowledge-track only, or failing to recognize dynamic `docs_root` relocations from `.compound-engineering/config.yaml`).
3. **Lack of Fleet Alignment:** In multi-agent environments where different developers and autonomous subagents run across different AI hosts, there was no declarative way to pin and audit a uniform Compound Engineering version across the fleet without manual file manipulation.

## Architecture & Implementation Solutions

In Phase 2 (v1.76.0), `ce-ai` established two dedicated subsystems to resolve these challenges cleanly:

### 1. Strongly-Typed Centralized CE Compatibility Layer (`src/compat/`)

Rather than scattering upstream assumptions across CLI subcommands, `src/compat/` serves as the centralized boundary isolating upstream Compound Engineering semantics:
- **`CeDocsConfig` (`src/compat/docs.rs`):** Dynamically discovers and respects `docs_root` relocations from `.compound-engineering/config.local.yaml` or `.compound-engineering/config.yaml`, returning standard paths for `plans_dir()` and `solutions_dir()`.
- **`CeSolutionFrontmatter` (`src/compat/schema.rs`):** Validates upstream `schema.yaml` conformance (`module`, `date`, `problem_type`, `component`, `severity`), correctly enforcing `applies_when` only for knowledge-track documents while exempting `bugfix` and `bug` problem tracks.
- **`CeSkillContract` (`src/compat/contracts.rs`):** Models documented skill integration contracts, identifying skills that support `mode:return-to-caller` (such as `ce-work` and `ce-resolve-pr-feedback`).
- **`CeRelease` (`src/compat/release.rs`):** Handles semantic version parsing, tag normalization, and version comparisons across releases.

Both `src/commands/workflow.rs` and `src/commands/doc.rs` were refactored to delegate docs root resolution and frontmatter validation entirely to `src/compat/`.

### 2. Fleet Version Governance Engine (`src/fleet/`)

The fleet subsystem introduces declarative version pinning and drives host-native packaging mechanisms:
- **`FleetState` in `state.json`:** Tracks `pinned_version` (e.g. `"v1.76.0"`) and `last_sync` timestamp within the atomic state store.
- **`FleetHarnessDriver` Trait (`src/fleet/driver.rs`):** A polymorphic driver interface defining readiness probing (`is_installed`), version detection (`detect_version`), and sync action planning (`plan_sync`).
- **Declarative `FleetAction` Plans:** Encapsulates planned sync operations into inspectable actions:
  - `UpToDate`: Host already matches pinned version.
  - `RunCommand`: Drives host-native CLI tools (e.g. `claude plugin update compound-engineering@v1.76.0` or `npm install @everyinc/compound-engineering@v1.76.0`).
  - `UpdateConfig`: Modifies host configuration files atomically (e.g. `opencode.json` plugins array).
- **Concrete Native Drivers (`src/fleet/drivers.rs`):** Out-of-the-box drivers for `OpenCodeDriver`, `ClaudeDriver`, `PiDriver`, `CodexDriver`, and `CursorDriver`.
- **Auditing & Reporting (`src/fleet/status.rs`):** Compares all host versions against the pinned version, categorizing alignment into `Aligned`, `Divergent`, `Missing`, or `Unmanaged`.

### 3. New CLI Surface (`src/commands/fleet.rs`)

A streamlined CLI interface under `ce-ai fleet`:
- `ce-ai fleet status [--json]`: Audits and reports installed harness versions, alignment status, and overall fleet readiness.
- `ce-ai fleet pin <version>`: Atomically records the target semantic version tag in `state.json`.
- `ce-ai fleet sync [--dry-run]`: Evaluates planned actions across all detected hosts, allowing safe previewing (`--dry-run`) before executing native package managers or config updates.

## Verification & Testing Evidence

1. **Unit Tests:**
   - 8 unit tests in `src/compat/` verifying docs root discovery, precedence, frontmatter schema validation, bug-track exemptions, and version parsing.
   - Driver tests in `src/fleet/drivers.rs` verifying OpenCode and Claude Code version detection and action plan synthesis.
   - State persistence test in `src/state/tests/state.rs` verifying `FleetState` roundtrip serialization.
2. **Integration Tests:**
   - `test_cli_fleet_pin_status_sync_lifecycle` in `tests/cli.rs` testing full lifecycle: initial unpinned status, JSON output, sync failure on unpinned state, invalid version rejection, pinning, dry-run sync, real sync, and atomic `opencode.json` modification.
3. **Quality Gates:**
   - `cargo fmt --check`: 100% compliant.
   - `cargo clippy --all-targets --all-features -- -D warnings`: 0 warnings.
   - Full test suite: 846 passing tests with 0 failures.
