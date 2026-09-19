---
title: "Secure Stdin & Masked Input for Decisions Auth"
domain: state
version: 1.0.0
last_updated: "2026-09-19"
---

# Specification: Secure Stdin & Masked Input for Decisions Auth

## Requirements & Acceptance Criteria

### REQ-AUTH-01: Standard Input Flag (`--stdin`)
- **WHEN** an operator runs `ce-ai decisions auth --stdin` with piped or redirected input containing a non-empty key:
  - **THEN** `ce-ai` reads the key until EOF or newline, trims whitespace, saves it securely to `~/.config/ce-ai/credentials.toml` with `0600` permissions, and prints confirmation with exit code 0.
- **WHEN** an operator runs `ce-ai decisions auth --stdin` with empty or whitespace-only input:
  - **THEN** `ce-ai` aborts without mutating credentials, emits an error message to stderr, and exits with exit code 2 (`CeError::Usage`).

### REQ-AUTH-02: Interactive Masked Prompting
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
- **WHEN** an operator runs `ce-ai decisions auth --key <VALUE>` with an explicit CLI argument:
  - **THEN** `ce-ai` saves the key as before, but emits a security warning to stderr:
    `warning: passing API key via command-line arguments exposes it in shell history and process lists; prefer interactive prompt or '--stdin'`.
  - **THEN** `ce-ai` exits with exit code 0.

### REQ-AUTH-04: Non-Interactive Environment Safety
- **WHEN** `ce-ai decisions auth` is run in a non-interactive environment (where `stdin` is not a TTY) without `--stdin` and without `--key`:
  - **THEN** `ce-ai` does NOT block or attempt to read from stdin, and displays the current configuration status.

### REQ-AUTH-05: Verification with Stdin Input
- **WHEN** an operator runs `echo "<KEY>" | ce-ai decisions auth --stdin --check`:
  - **THEN** `ce-ai` saves the key from stdin and immediately performs a health check against the configured provider using the newly saved key.
