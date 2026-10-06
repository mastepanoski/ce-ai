---
title: "CE-AI v2.0 GA: Cross-Host Operational Companion Pivot & File Scraping Decommissioning"
module: architecture
date: 2026-10-06
problem_type: architecture
component: core
severity: medium
tags:
  - architecture
  - v2-pivot
  - fleet
  - operational-companion
  - decommissioning
applies_when: "When migrating from CE-AI v1 file-scraping to CE-AI v2 cross-host operational companion and fleet governance"
---

# CE-AI v2.0 GA: Cross-Host Operational Companion Pivot & File Scraping Decommissioning

## Context & Problem
CE-AI was originally built as a workflow governance and harness sync tool that extracted raw skills from GitHub release tarballs and unpacked them into harness directories (`~/.config/opencode/compound-engineering`, etc.), enforcing SHA-256 manifest checks and background watch-loops (`sync --watch`). 

In upstream review, the maintainer of Compound Engineering highlighted critical friction:
1. Copying raw skills out of tarballs bypassed host-specific packaging transformations (Bun converter, native marketplace manifests).
2. Running alongside native installations produced `external-duplicate` conflicts and two sources of truth.
3. Aggressive watch-loop file restoration clobbered developer and host adaptations.

## Architectural Decision & Solution
In CE-AI v2.0 GA, we fully decommissioned the file scraping pipeline and repositioned CE-AI as a **Cross-Host Operational Companion for Compound Engineering**:
- **Domain Separation:** Compound Engineering owns engineering semantics and workflow artifacts; AI hosts own native execution and packaging; CE-AI coordinates environment readiness, multi-harness fleet version governance, project adoption, and advisory workflow observation.
- **Fleet Version Governance (`src/fleet/`):** Introduced in Phase 2, `ce-ai fleet (pin/status/sync)` uses polymorphic `FleetHarnessDriver` implementations to inspect and update native manifests (such as `opencode.json` plugin arrays) rather than scraping raw files.
- **Decommissioning Legacy File Scraping:** In Phase 4, `extract_to_source` was decommissioned in `src/source/archive.rs`. Invocations of `ce-ai install`, `ce-ai upgrade`, and `ce-ai sync --watch` now emit explicit deprecation advisories pointing to `ce-ai init-prj` and `ce-ai fleet pin / sync`.
- **Identity & Docs:** `README.md` was rewritten (maintaining the strict ≤ 100 line limit) to introduce the Cross-Host Operational Companion role and modern Quick Start commands.

## Key Learnings & Verification
- Preserved binary self-update security (`is_safe_relative_path` and `extract_safe`) in `src/source/archive.rs` for `ce-ai self-update` while eliminating plugin tarball scraping.
- Maintained backward compatibility for existing scripts by emitting deprecation notices to stderr without breaking command execution.
- Validated concept accretion in `CONCEPTS.md` (71 entries, was 68, +3 added, 0 scrubbed).
