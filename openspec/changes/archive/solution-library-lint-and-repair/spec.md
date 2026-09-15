# Spec: Solution Library Drift Repair & Frontmatter Schema Normalization

## Requirements & Acceptance Criteria

### Requirement 1: Complete YAML Frontmatter on All Solution Documents
WHEN `ce-ai doctor` or `probe_solution_drift` inspects any file under `docs/solutions/**/*.md`
THEN the file MUST contain a valid YAML frontmatter header delimited by `---`
AND the header MUST contain `title`, either `category` or `module`, `problem_type`, `tags`, and `applies_when`
AND `probe_solution_drift` MUST emit zero missing frontmatter warnings across all 74 solution files.

### Requirement 2: Zero Dead Source Code References in Solution Documents
WHEN `ce-ai doctor` or `probe_solution_drift` checks for backticked paths outside code blocks
THEN every path candidate starting with `src/` or `tests/` and ending in `.rs` (after stripping line numbers and anchors) MUST exist as an actual file in the repository root
AND `probe_solution_drift` MUST emit zero dead path warnings across all 74 solution files.

### Requirement 3: Idempotent & Lossless Repair
WHEN the solution normalization and repair process is executed on `docs/solutions/`
THEN all existing solution markdown prose, headings, bullet points, and code examples MUST be preserved verbatim
AND no file formatting or non-frontmatter text shall be corrupted or lost.

### Requirement 4: Clean Doctor Health Report
WHEN `ce-ai doctor` is executed on the repository
THEN the diagnostic engine MUST output `doctor: ok`
AND `doc debt` summary in `workflow status` MUST report 0 dead solution links and 0 solutions missing frontmatter.
