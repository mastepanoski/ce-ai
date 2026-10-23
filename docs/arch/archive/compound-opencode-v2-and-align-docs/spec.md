# Specification: Compound Capture & Doc Styling Alignment

## Requirements

### R1. Compound Solution Documentation
- **WHEN** `docs/solutions/bugfixes/opencode-v2-dual-loader.md` is inspected
- **THEN** it MUST contain valid YAML frontmatter matching `references/schema.yaml` with fields `module`, `problem_type`, `tags`, and `date`.
- **AND** it MUST detail the two root causes (key rename, export schema), the dual loader resolution, native command materialization, and verification steps.

### R2. Monotonic Concept Accretion
- **WHEN** `CONCEPTS.md` is updated
- **THEN** zero existing concepts shall be removed or truncated.
- **AND** the additions MUST include `OpenCode Dual Plugin Loader`, `Native Command Materialization`, and `OpenCode Config Key Agnosticism`.
- **AND** `cargo run -- doc lint --strict` MUST exit with code 0.

### R3. README Styling Compliance
- **WHEN** `README.md` is evaluated against `docs/references/docs-styling.md`
- **THEN** total line count MUST be ≤ 100 lines.
- **AND** the Quick Path (`## Try it in two minutes`) MUST immediately follow the Title/What & Why block without intervening explanatory sections.
- **AND** the Documentation Map MUST maintain Audience and Intent columns for all user guide documents.

### R4. Diátaxis Intent Purity
- **WHEN** `docs/user-guide/project-adoption-guide.md` is inspected
- **THEN** its declared Diátaxis intent MUST be singularly `How-to`.
