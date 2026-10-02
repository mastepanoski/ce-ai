---
date: 2026-10-02
topic: upstream-coupling-remediation
status: ready
source: docs/architecture/ce-ai-v2-architecture-prd.md
---

# Plan: Upstream Coupling Remediation & CE Contract Alignment

## Problem Frame & Scope

In response to upstream review from Compound Engineering, `ce-ai` must resolve 7 concrete coupling divergences to ensure seamless interoperability with the official CE plugin and CLI:
1. Dynamic `docs_root` resolution from `.compound-engineering/config.yaml`.
2. Conformance with upstream `schema.yaml` for solution frontmatter (making `applies_when` conditional on non-bugfix, `tags` optional, and validating `module`, `date`, `problem_type`, `component`, `severity`).
3. Parsing `component` (singular) and `related_components` in solution metadata.
4. Detecting modern requirements-only plans in `<docs_root>/plans/*-requirements.md` (and `.html`).
5. Expanding dead-path validation to all common code extensions beyond `.rs`.
6. Resolving Pi extension filename collision by using `ce-ai-companion.ts` with legacy cleanup.
7. Hardening `ce-debug` gate exemption to exact structured prefixes.

## Implementation Units

### Unit 1: Dynamic `docs_root` Discovery
- **Files:** `src/commands/workflow.rs`, `src/commands/doc.rs`, `src/commands/tests/workflow.rs`
- **Target:** ~120 LOC
- **Approach:**
  - Define `pub fn resolve_docs_root(repo_root: &Path) -> PathBuf`.
  - Read `.compound-engineering/config.yaml` or `config.local.yaml` for `docs_root: <path>`.
  - Replace hardcoded `repo_root.join("docs")` in solution collection and drift probing with `repo_root.join(resolve_docs_root(repo_root))`.
  - Unit test custom `docs_root` parsing and default fallback.

### Unit 2: Solution Frontmatter Upstream Conformance & Component Parsing
- **Files:** `src/commands/workflow.rs`, `src/commands/doc.rs`, `src/commands/tests/doc.rs`
- **Target:** ~140 LOC
- **Approach:**
  - Update `check_solution_frontmatter` to require: `module`/`category`, `date`, `problem_type`, `component`/`components`, and `severity`.
  - Only require `applies_when` if `problem_type` is not `bugfix` or `bug`.
  - Treat `tags` as optional.
  - In `parse_solution_file`, handle `component` (pushing singular value) and `related_components` (pushing list items).
  - Add unit tests validating upstream CE solution frontmatter.

### Unit 3: Brainstorm Detection, Dead Paths, Pi Extension & Gate Hardening
- **Files:** `src/commands/workflow.rs`, `src/commands/gate.rs`, `src/harness/pi.rs`, `src/harness/tests/pi.rs`, `src/commands/tests/gate.rs`
- **Target:** ~110 LOC
- **Approach:**
  - In `src/commands/workflow.rs`, check `<docs_root>/plans/` for `*-requirements.md` / `.html` in addition to legacy `docs/brainstorms/`.
  - In `clean_code_path`, match multi-language code extensions (`.ts`, `.py`, `.go`, etc.).
  - In `src/harness/pi.rs`, set `PI_EXTENSION_FILENAME = "ce-ai-companion.ts"`. In `remove_session_start_hook`, also remove legacy `compound-engineering.ts` if managed.
  - In `src/commands/gate.rs`, enforce `is_ce_debug = clean == "ce-debug" || clean.starts_with("ce-debug:") || clean.starts_with("ce-debug ") || clean.starts_with("ce-debug/")`.

## Verification & Acceptance
- Full unit test coverage for each of the 7 remediations.
- `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- `cargo test`.
- `cargo run -- doc lint --strict`.
