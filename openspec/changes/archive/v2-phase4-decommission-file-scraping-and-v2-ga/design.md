# Design: Phase 4 — Full Decommissioning of File Scraping & CE-AI v2.0 Release (v2.0.0 GA)

## Architectural Design

### 1. Decommissioning Plan for File Scraping
In `src/source/archive.rs`:
- Deprecate `extract_to_source` and `find_source_root` (which were only used to extract `.opencode` and `skills/` from upstream release tarballs).
- Retain `is_safe_relative_path` and `extract_safe` for binary release self-updates (`src/source/binary_release.rs`) and test security verifications.

### 2. Deprecation & Guidance for CLI Surfaces
In `src/commands/install.rs`:
- Emit a deprecation advisory notice when executed:
  ```text
  [DEPRECATION] 'ce-ai install' file copying is deprecated in CE-AI v2.
  Use 'ce-ai init-prj' to adopt the project repository and 'ce-ai fleet pin <version>' / 'ce-ai fleet sync' to manage harness plugins natively.
  ```
- Retain existing logic in non-destructive mode / dry-run to prevent crashing automated pipelines, but recommend fleet commands.

In `src/commands/upgrade.rs`:
- Emit deprecation notice:
  ```text
  [DEPRECATION] 'ce-ai upgrade' is deprecated in CE-AI v2.
  Use 'ce-ai fleet pin <version>' followed by 'ce-ai fleet sync' to update plugins across coding agent harnesses.
  ```

In `src/commands/sync.rs`:
- Deprecate the `--watch` polling loop. If `--watch` is invoked, emit:
  ```text
  [DEPRECATION] 'ce-ai sync --watch' continuous file restoration is deprecated in CE-AI v2.
  Native hosts manage plugin lifecycles. Run 'ce-ai fleet sync' for cross-harness synchronization.
  ```
- Provide deprecation banner on standard `ce-ai sync` runs.

### 3. Identity and Documentation Pivot (v2.0 GA)
- **`Cargo.toml`:**
  - `version = "2.0.0"`
  - `description = "Cross-host operational companion for Compound Engineering across AI coding agents"`
- **`README.md`:**
  - Update title and description to *Cross-Host Operational Companion*.
  - Update quick start to reflect `init-prj`, `fleet pin`, `fleet sync`, and `workflow status`.
  - Maintain strictly ≤ 100 lines.
- **`CHANGELOG.md`:**
  - Add `[2.0.0]` entry summarizing all four phases of the v2 pivot.

### 4. Monotonic Concept Accretion (`CONCEPTS.md`)
Add:
- `Cross-Host Operational Companion (v2 GA)`
- `Decommissioned File Scraping`
- `Native Host Packaging Sovereignty`
