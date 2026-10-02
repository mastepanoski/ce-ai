# Exploration: Upstream Coupling Remediation & CE Contract Alignment

## Technical Investigation

### 1. Dynamic `docs_root` Discovery
- Upstream CE stores project configuration in `.compound-engineering/config.yaml` (or `config.local.yaml`).
- Format:
  ```yaml
  docs_root: docs
  ```
  or custom paths like `docs_root: documentation` or `docs_root: internal/docs`.
- Strategy: Implement `resolve_docs_root(repo_root: &Path) -> PathBuf` in `src/commands/workflow.rs` (or a shared compat helper). If `.compound-engineering/config.yaml` exists, parse `docs_root:` line; fallback to `PathBuf::from("docs")`.

### 2. Upstream `schema.yaml` Conformance
- Upstream `skills/ce-compound/references/schema.yaml`:
  - Mandatory keys:
    - `module`: string (e.g. `workflow::resume`, `harness::pi`)
    - `date`: YYYY-MM-DD
    - `problem_type`: enum (`bugfix`, `architecture`, `pattern`, `discovery`, `config`, `preference`)
    - `component`: string (primary component affected)
    - `severity`: enum (`critical`, `major`, `standard`, `minor`)
  - Optional keys:
    - `schema_version`: integer/string
    - `related_components`: list of strings
    - `tags`: list of strings
    - `title`: optional in YAML if top-level `# Title` exists (or vice versa)
    - `applies_when`: required only for knowledge-track (non-bugfix) docs; optional for bugfixes.
- Strategy: Update `check_solution_frontmatter` to validate `module` (or `category`), `date`, `problem_type`, `component`, and `severity`. Only require `applies_when` if `problem_type != "bugfix"` and `problem_type != "bug"`. Do not fail if `tags` is absent.

### 3. Component & Related Components Parsing
- Current `parse_solution_file` in `src/commands/doc.rs` only handles `"components"`.
- Strategy: Handle `"component"` (singular value pushed to `components`) and `"related_components"` (array of values pushed to `components`).

### 4. Brainstorms as Modern Requirements-Only Plans
- Upstream `ce-brainstorm` emits requirements documents directly into `<docs_root>/plans/*-requirements.md` (and `.html`).
- Strategy: Update `probe_active_ideation` and `has_brainstorms` in `src/commands/workflow.rs` to inspect:
  1. `<docs_root>/plans/` for files ending in `-requirements.md` or `-requirements.html`.
  2. Legacy `<docs_root>/brainstorms/` and `<docs_root>/ideation/`.

### 5. Multi-Language Dead-Path Checking
- Current `clean_code_path` checks: `candidate.ends_with(".rs")`.
- Strategy: Expand to:
  `.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`, `.go`, `.rb`, `.c`, `.cpp`, `.cc`, `.h`, `.hpp`, `.java`, `.kt`, `.swift`, `.sh`, `.bash`, `.zsh`, `.yaml`, `.yml`, `.json`, `.toml`.

### 6. Pi Extension Collision Avoidance
- Current `PI_EXTENSION_FILENAME`: `"compound-engineering.ts"`.
- Collision: Overwrites upstream CE's own Pi extension.
- Strategy: Change constant to `"ce-ai-companion.ts"`. In `remove_session_start_hook`, clean up both `ce-ai-companion.ts` and legacy `compound-engineering.ts` (if it contains `"ce-ai workflow resume"`).

### 7. Precise `ce-debug` Gate Exemption
- Current check: `clean.starts_with("ce-debug") || clean.contains("ce-debug")`.
- Strategy: Require structured match:
  `clean == "ce-debug" || clean.starts_with("ce-debug:") || clean.starts_with("ce-debug ") || clean.starts_with("ce-debug/")`.
