---
title: "State Management, Model Profiles & Configuration Overrides"
domain: state
version: 1.0.0
last_updated: "2026-09-19"
---

# Specification: State Management, Model Profiles & Configuration Overrides

## 1. Overview & Architectural Boundaries

The `state` module manages global persistence (`~/.config/ce-ai/state.json`), model slot assignments across agent personas (`ce-brainstorm`, `ce-plan`, `ce-work`), model snapshot profiles, and repository-local configuration overrides (`.ce-ai.json`).

## 2. Capabilities & Requirements

### R1. Atomic File Persistence (`write_atomic`)
WHEN saving state or writing configuration files  
THEN the system MUST write through a temporary sibling file and perform an atomic rename (`rename(2)` / POSIX atomic swap) to prevent corruption during unexpected termination.

### R2. Workspace Local Overrides (`.ce-ai.json`)
WHEN a repository defines `.ce-ai.json` at the root  
THEN `ce-ai` MUST deserialize local overrides (model assignments, `doc_hygiene`, `archive_compaction_threshold`) and merge them over global state with higher precedence.

### R3. Model Assignment Drift Reconciliation
WHEN model configurations change in host harness settings or `state.json`  
THEN `ce-ai sync` MUST bidirectionally reconcile assignments and update `state.json` cleanly.

### R4. Model Profile Snapshots
WHEN operator saves a profile snapshot (`ce-ai models profile save <name>`)  
THEN the system MUST snapshot all active model slots and allow restoring the snapshot via `ce-ai models profile load <name>`.

## 3. Data Models & CLI Contracts

- `State` struct: schema of `~/.config/ce-ai/state.json`.
- `ModelAssignment`: mapping agent slots (`brainstorm`, `plan`, `work`, etc.) to specific LLM models.
- `DocHygieneConfig`: `{ stale_spec_days, check_solution_paths, require_solution_frontmatter, archive_compaction_threshold }`.
- CLI commands: `ce-ai models set`, `ce-ai models list`, `ce-ai models profile save/load`.

## 4. Invariants & Operational Boundaries

- Direct unbuffered file overwrites of `state.json` or `opencode.json` are strictly forbidden (Hard Invariant #3).
- Model assignments MUST validate slot names against recognized canonical roles.

### REQ-AUTH-01: Standard Input Flag (`--stdin`)
<!-- promoted-from: change:secure-decisions-auth-stdin date:2026-09-19 -->
- **WHEN** an operator runs `ce-ai decisions auth --stdin` with piped or redirected input containing a non-empty key:
  - **THEN** `ce-ai` reads the key until EOF or newline, trims whitespace, saves it securely to `~/.config/ce-ai/credentials.toml` with `0600` permissions, and prints confirmation with exit code 0.
- **WHEN** an operator runs `ce-ai decisions auth --stdin` with empty or whitespace-only input:
  - **THEN** `ce-ai` aborts without mutating credentials, emits an error message to stderr, and exits with exit code 2 (`CeError::Usage`).

### REQ-AUTH-02: Interactive Masked Prompting
<!-- promoted-from: change:secure-decisions-auth-stdin date:2026-09-19 -->
- **WHEN** an operator runs `ce-ai decisions auth --key` (flag without parameter) in an interactive terminal:
  - **THEN** `ce-ai` prompts `Enter TypeSafe/Jev API key (input hidden, press Enter to cancel): ` and reads keystrokes in raw mode without echoing characters to screen.
- **WHEN** an operator inputs a valid key and presses `Enter`:
  - **THEN** `ce-ai` saves the key to `credentials.toml` and prints `Saved API key to <PATH>`.
- **WHEN** an operator runs `ce-ai decisions auth` (no flags, no `--check`) in an interactive terminal:
  - **THEN** `ce-ai` prompts `Enter TypeSafe/Jev API key (input hidden, press Enter to cancel): `.
  - **THEN** if `Enter` is pressed with empty input, `ce-ai` prints `No key entered; preserving current configuration.` and displays current status.
- **WHEN** an operator presses `Ctrl+C` or `Escape` during interactive prompting:
  - **THEN** `ce-ai` immediately restores terminal raw mode, cancels credential update, and exits with code 2 (`CeError::Usage`).

### REQ-AUTH-03: Backward Compatibility & Security Warning
<!-- promoted-from: change:secure-decisions-auth-stdin date:2026-09-19 -->
- **WHEN** an operator runs `ce-ai decisions auth --key <VALUE>` with an explicit CLI argument:
  - **THEN** `ce-ai` saves the key as before, but emits a security warning to stderr:
    `warning: passing API key via command-line arguments exposes it in shell history and process lists; prefer interactive prompt or '--stdin'`.
  - **THEN** `ce-ai` exits with exit code 0.

### REQ-AUTH-04: Non-Interactive Environment Safety
<!-- promoted-from: change:secure-decisions-auth-stdin date:2026-09-19 -->
- **WHEN** `ce-ai decisions auth` is run in a non-interactive environment (where `stdin` is not a TTY) without `--stdin` and without `--key`:
  - **THEN** `ce-ai` does NOT block or attempt to read from stdin, and displays the current configuration status.

### REQ-AUTH-05: Verification with Stdin Input
<!-- promoted-from: change:secure-decisions-auth-stdin date:2026-09-19 -->
- **WHEN** an operator runs `echo "<KEY>" | ce-ai decisions auth --stdin --check`:
  - **THEN** `ce-ai` saves the key from stdin and immediately performs a health check against the configured provider using the newly saved key.
