# Specification: Upstream Coupling Remediation & CE Contract Alignment

## Requirements & Acceptance Criteria

### REQ-1: Configurable `docs_root` Support
- **WHEN** `.compound-engineering/config.yaml` contains `docs_root: <custom_dir>`,
- **THEN** `ce-ai` MUST locate plans, brainstorms, and solutions under `<custom_dir>` rather than hardcoded `docs/`.
- **WHEN** no configuration exists,
- **THEN** `ce-ai` MUST fall back to `docs/`.

### REQ-2: Solution Frontmatter Upstream Schema Conformance
- **WHEN** a solution document is evaluated by `check_solution_frontmatter`,
- **THEN** it MUST accept `module`, `date`, `problem_type`, `component`, and `severity` as standard fields.
- **WHEN** `problem_type` is `bugfix` or `bug`,
- **THEN** `applies_when` MUST be treated as optional and MUST NOT trigger a lint error if omitted.
- **WHEN** `tags` is omitted,
- **THEN** it MUST NOT trigger a lint error.

### REQ-3: Component Field Parity
- **WHEN** parsing frontmatter in `parse_solution_file`,
- **THEN** the parser MUST accept `component: <name>`, `related_components: [...]`, and `components: [...]`, merging them into the metadata components list.

### REQ-4: Modern Brainstorming Plan Discovery
- **WHEN** probing active ideation/brainstorming artifacts,
- **THEN** `ce-ai` MUST recognize requirements-only plans (`*-requirements.md`, `*-requirements.html`) located under `<docs_root>/plans/` as ideation artifacts, in addition to legacy `<docs_root>/brainstorms/` and `<docs_root>/ideation/`.

### REQ-5: Multi-Language Dead-Path Validation
- **WHEN** extracting code paths in `clean_code_path`,
- **THEN** candidate paths matching `.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`, `.go`, `.rb`, `.c`, `.cpp`, `.cc`, `.h`, `.hpp`, `.java`, `.kt`, `.swift`, `.sh`, `.bash`, `.zsh`, `.yaml`, `.yml`, `.json`, or `.toml` MUST be recognized and checked for existence.

### REQ-6: Pi Companion Extension Namespacing
- **WHEN** `ensure_session_start_hook` writes a Pi extension,
- **THEN** it MUST write to `.pi/extensions/ce-ai-companion.ts`, never clobbering `.pi/extensions/compound-engineering.ts`.
- **WHEN** `remove_session_start_hook` or `deinit_prj` cleans up Pi extensions,
- **THEN** it MUST remove both `ce-ai-companion.ts` and legacy `compound-engineering.ts` if they contain `"ce-ai workflow resume"`.

### REQ-7: Precise `ce-debug` Gate Exemption
- **WHEN** gate check evaluates a task string for `ce-debug` direct entry,
- **THEN** it MUST only grant exemption if the task string matches `ce-debug` as a prefix (`ce-debug:`, `ce-debug `, `ce-debug/`) or exact string, rejecting arbitrary substring occurrences.
