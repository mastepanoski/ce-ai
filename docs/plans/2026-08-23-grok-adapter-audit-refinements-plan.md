# Implementation Plan: Grok Adapter Audit Refinements

- **Goal**: Resolve test environment race condition and clean up legacy dead code for Grok native harness adapter.

## Proposed Work Steps

### Step 1: OpenSpec Contract & Doc Review
- Author OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`).
- Run `ce-doc-review` panel and address findings.

### Step 2: Implementation
- `src/harness/grok.rs`: Protect unit tests accessing `GROK_HOME` with a static `Mutex` lock to prevent parallel test races.
- `src/harness/generic_json.rs`: Remove dead legacy `HarnessKind::Grok` mapping and update module docstring.

### Step 3: Verification & Quality Gates
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

### Step 4: Code Review & Shipping
- Run `ce-code-review` panel.
- Update existing solution document `docs/solutions/architecture/grok-native-harness-adapter.md` with audit refinement details (thread safety mutex guard & generic JSON cleanup).
- Save Engram observation.
- Bump SemVer to `v1.13.2` in `Cargo.toml` and `Formula/ce-ai.rb`, update `CHANGELOG.md`.
- Commit, push, open PR, wait for green CI matrix, merge, tag `v1.13.2`, release.
