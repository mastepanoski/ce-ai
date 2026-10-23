# Technical Design: Deterministic Tasks Tail Validator

## CLI Interface (`scripts/validate-tasks-tail.py`)

```bash
usage: validate-tasks-tail.py [-h] [--fix] [--check-code] [--allow-exempt] [--json] [path]

Validate presence of lifecycle tail in OpenSpec tasks.md

positional arguments:
  path            Path to tasks.md or directory containing tasks.md (defaults to active change)

options:
  -h, --help      show this help message and exit
  --fix           Append standardized lifecycle tail if missing
  --check-code    Enforce lifecycle tail regardless of git diff or proposal heuristics
  --allow-exempt  Allow documentation-only changes to omit lifecycle tail (default: True)
  --json          Emit output as JSON
```

## Parsing Engine

1. **Path Resolution**:
   - If argument provided: resolve path; if directory, search for `tasks.md` inside it.
   - If omitted: scan `openspec/changes/*/tasks.md` (excluding `archive/`), matching against current branch or active feature in `state.json`.

2. **Tail Content Scanning**:
   Check for presence of:
   - `simplify` or `ce-simplify-code`
   - `review` or `review-receipt` or `ce-code-review`
   - `compound` or `ce-compound` or `docs/solutions` or `CONCEPTS.md`

3. **Status Codes**:
   - `0`: Valid (tail present, or successfully `--fix`ed, or exempt).
   - `1`: Invalid (tail missing for code change).
   - `2`: Usage / IO error (file not found or unreadable).

## Test Suite (`tests/test_validate_tasks_tail.py`)
Python `unittest` suite covering:
1. Incomplete `tasks.md` without tail -> fails with code 1.
2. Complete `tasks.md` with tail -> passes with code 0.
3. `--fix` adds tail and subsequent validation passes.
4. `--fix` is idempotent.
5. Documentation-only changes are recognized as exempt.
