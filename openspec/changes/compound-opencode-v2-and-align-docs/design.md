# Technical Design: Compound Capture & Doc Styling Alignment

## Architecture & File Layout

### 1. Solution Artifact: `docs/solutions/bugfixes/opencode-v2-dual-loader.md`
- **Frontmatter**:
  ```yaml
  module: opencode
  problem_type: bugfix
  tags:
    - opencode
    - plugin-loader
    - v2-migration
    - esm
    - native-commands
  date: 2026-09-30
  ```
- **Structure**:
  - Problem Context & Symptoms
  - Root Cause Analysis (Key rename & ESM export validation)
  - Solution Architecture (Dual Export Contract, Dynamic Directory Resolution, Native Command Materialization)
  - Verification Evidence (Local OpenCode Daemon API, Doctor, Docker E2E)
  - Prevention & Best Practices

### 2. `CONCEPTS.md` Monotonic Extension
- Read `CONCEPTS.md` prior to mutation.
- Use surgical insertion to add:
  - `### OpenCode Dual Plugin Loader` under Harness Adapters & Plugin Management
  - `### Native Command Materialization` under Skill & Command Surface
  - `### OpenCode Config Key Agnosticism` under Configuration & State Management
- Verify with `ce-ai doc lint --strict`.

### 3. `README.md` Restructuring
- Target: ≤ 85 lines.
- Sections:
  1. `# CE-AI — Compound Engineering Workflow Orchestration & Governance` + Subtitle (3 lines)
  2. `## Try it in two minutes` (Quick Path)
  3. `## Documentation map` (Diátaxis & Audience table)
  4. `## Acknowledgments & Project Links`

### 4. `docs/references/docs-styling.md` & `docs/user-guide/project-adoption-guide.md`
- Normalize `project-adoption-guide.md` intent to `How-to`.
- Add links to Reference and Explanation examples in `docs-styling.md`.
