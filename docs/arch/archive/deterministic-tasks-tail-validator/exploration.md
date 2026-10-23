# Technical Exploration: Deterministic Tasks Tail Validator

## 1. Problem Context & Lessons from `validate-concepts.py`
Earlier in this repository, `CONCEPTS.md` suffered from catastrophic truncations by AI agents until `scripts/validate-concepts.py` was introduced. The key to its success was:
- Zero external dependencies (standard Python library).
- Runs sub-50ms.
- Clear error output stating exactly what failed and how to remediate.
- Direct integration into CI and local pre-commit hooks.

We apply the exact same architecture to `tasks.md` governance.

## 2. Detection Strategy (Code vs. Docs-Only)
Not all changes require code simplification or bugfix solution documents:
- **Code Changes**: Modifying files in `src/`, `tests/`, `benches/`, or script binaries requires code simplification, review receipts, and knowledge compounding.
- **Documentation/Chore Changes**: Modifying only `README.md`, `docs/`, `LICENSE`, or `.gitignore` does not produce executable code and is exempt from code simplification.

`validate-tasks-tail.py` resolves code change relevance via:
1. Explicit `--check-code` CLI flag.
2. Git diff of current branch against base/HEAD (`git diff --name-only origin/main...` or `git status --porcelain`).
3. Inspection of `proposal.md` or `design.md` in the same directory as `tasks.md` (scanning for `src/` or `tests/` paths).

## 3. Standardized Lifecycle Tail Template
When `--fix` is applied, the script appends:

```markdown
- [ ] **Unit Final: Lifecycle Completion & Governance Gate**
  - [ ] **Simplification (`ce-simplify-code`)**: Audit changed code for reuse, quality, and efficiency while preserving behavior.
  - [ ] **Revisión formal (`ce-code-review`)**: Run review and record `ce-ai workflow review-receipt`.
  - [ ] **Captura de conocimiento (`ce-compound`)**: Capture learnings in `docs/solutions/<category>/` and monotonically accrete `CONCEPTS.md`.
  - [ ] **Verificación de cierre**: Comply with DoD (`cargo fmt`, `cargo clippy`, `cargo test`, `make e2e`, `doc lint --strict`).
```

If the file already contains these elements, `--fix` is an idempotent no-op.
