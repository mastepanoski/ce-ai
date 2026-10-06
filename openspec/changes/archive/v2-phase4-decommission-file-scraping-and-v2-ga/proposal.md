# Proposal: Phase 4 — Full Decommissioning of File Scraping & CE-AI v2.0 Release (v2.0.0 GA)

## Problem Statement

In CE-AI v1, the tool extracted raw `skills/` out of GitHub release tarballs and copied them directly into harness directories (e.g. `~/.config/opencode/compound-engineering`), enforcing SHA256 integrity checks and a continuous watcher loop (`sync --watch`). As highlighted in upstream review by the maintainer of Compound Engineering:
1. **Packaging Divergence:** Raw file scraping bypasses host-specific packaging transformations (such as native marketplace manifests or compiler rewrites).
2. **Dual Sources of Truth:** Copying files alongside native host installs causes `external-duplicate` conflicts and risks clobbering user customizations.
3. **Watcher Friction:** Restoring file edits in a background polling loop fights against host-native plugin systems.

Following the successful delivery of **Phase 1** (upstream coupling fixes in v1.75.1), **Architecture Refinement** (No Semantic Authority in v1.75.2), **Phase 2** (CE Compatibility Layer & Fleet Subsystem in v1.76.0), and **Phase 3** (Advisory Observation Engine & OpenSpec Decoupling in v1.77.0), CE-AI is ready for its final transformation step: **Phase 4 (v2.0.0 GA)**.

## Scope Boundaries

### In-Scope
- **Decommission Plugin Tarball Scraping:** Retire `extract_to_source` in `src/source/archive.rs` that unpacked GitHub release tarballs of Compound Engineering into local cache trees.
- **Decommission Diff Watcher Loop:** Deprecate and neutralize the aggressive background file-restoration loop in `ce-ai sync --watch`, directing users to `ce-ai fleet sync`.
- **Evolve CLI Command Surfaces:** Gracefully deprecate `ce-ai install` and file-level `ce-ai sync` in favor of `ce-ai init-prj` (project adoption) and `ce-ai fleet pin` / `sync` (native harness management).
- **Update Identity & Documentation:** Rewrite `README.md` (≤ 100 lines) and user documentation to present CE-AI as the *Cross-Host Operational Companion for Compound Engineering*.
- **Accrete Domain Concepts:** Monotonically record v2 GA domain concepts in `CONCEPTS.md`.
- **Document Solution:** Capture the architectural migration in `docs/solutions/architecture/`.
- **Release CE-AI v2.0.0 GA:** Bump SemVer to `2.0.0` in `Cargo.toml` and document all changes in `CHANGELOG.md`.

### Out-of-Scope
- Breaking `ce-ai self-update`: The binary self-updater will retain safe extraction utilities (`is_safe_relative_path` and `extract_safe`) for downloading and verifying CE-AI binary releases.
- Altering model profile management (`ce-ai models`) or token telemetry (`ce-ai usage`).

## Risk Evaluation & Mitigation
- **Risk:** Existing scripts or CI calling `ce-ai install` or `ce-ai sync` could break if commands are deleted.
- **Mitigation:** Retain commands with non-zero deprecation guidance; provide clear operational hints redirecting to `init-prj` and `fleet sync`.

## Success Criteria
- Zero raw skill copying from upstream release tarballs into harness plugin directories.
- Deprecation warnings emitted when legacy file-scraping surfaces are invoked.
- `README.md` stays strictly ≤ 100 lines and explains the new v2 companion role.
- 100% green CI matrix across Linux, macOS, and Windows.
- `make e2e` passes.
