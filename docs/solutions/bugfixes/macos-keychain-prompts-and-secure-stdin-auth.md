---
title: "macOS Keychain GUI Prompt Prevention and Secure Stdin Decisions Auth"
category: "bugfixes"
module: "src/decisions/auth.rs"
date: "2026-09-19"
problem_type: "security_issue"
component: "authentication"
severity: "high"
symptoms:
  - "macOS SecurityAgent displays modal popup prompting for Keychain password during CLI commands"
  - "CLI commands like `ce-ai doctor` or `ce-ai decisions auth --check` block on GUI password dialog"
  - "API keys passed via CLI flag `--key` exposed in shell history and process table"
root_cause: "config_error"
resolution_type: "code_fix"
applies_when: "When resolving or storing API keys for TypeSafe / Jev decision engine across macOS, Linux, and Windows"
tags:
  - "decisions"
  - "keychain"
  - "macos"
  - "credentials"
  - "securityagent"
  - "stdin"
---

# macOS Keychain GUI Prompt Prevention and Secure Stdin Decisions Auth

## Problem

When operators configured TypeSafe / Jev decision engine credentials via `ce-ai decisions auth`, two problems emerged:

1. **Shell History Exposure**: Passing an API key directly on the command line via `--key <VAL>` persists sensitive API tokens in shell history (`.zsh_history`, `.bash_history`) and exposes them to local process listings (`ps aux`).
2. **macOS Keychain GUI Password Dialogs**: When `ce-ai` integrated the OS keyring via the `keyring` crate, accessing or saving passwords in the macOS Keychain triggered macOS `SecurityAgent` modal dialogs asking: `"ce-ai wants to access key 'typesafe' in your keychain. Enter password for keychain 'login' to allow this."`
   This happened repeatedly because `ce-ai` binaries built locally (`cargo build` / `cargo test`) are ad-hoc signed. Each compilation changes the binary's code directory hash (CDHash). Without the `-A` access flag (or matching Apple Developer code signing), macOS considers each rebuilt binary an untrusted application, blocking execution until the user enters their macOS login keychain password. Furthermore, `resolve_api_key()` was querying OS Keyring *before* checking local `~/.config/ce-ai/credentials.toml` (mode `0600`), causing even routine read-only commands (`ce-ai doctor`, `ce-ai decisions status`) to block on macOS Keychain prompts.

## Solution

### 1. Inverted Credential Resolution Precedence
The credential resolution hierarchy in `src/decisions/auth.rs` was re-ordered to prioritize file-based storage over the OS Keyring:
1. `CE_AI_TYPESAFE_API_KEY` environment variable.
2. `TYPESAFE_API_KEY` / `JEV_API_KEY` environment variables.
3. `~/.config/ce-ai/credentials.toml` local file with strict `0600` permissions.
4. OS Keyring (`keyring_get()`) as a fallback (maintaining cross-compatibility with `jevkit`).

By checking `credentials.toml` first, normal CLI invocations resolve the API key in under 0.1 ms with zero IPC overhead and zero macOS Keychain interaction.

### 2. macOS `-A` Access Control Flag
When saving credentials on macOS via `keyring_set()`, `ce-ai` invokes `security add-generic-password -U -s ce-ai -a typesafe -w <KEY> -A`. The `-A` flag grants all applications access to the specific password entry, preventing `SecurityAgent` from prompting the operator when running updated or freshly-compiled development binaries.

### 3. Secure Stdin Ingestion and Masked Interactive Prompts
`ce-ai decisions auth` now supports:
- `--stdin`: Reads the API key directly from standard input (e.g. `cat secret.txt | ce-ai decisions auth --stdin` or `echo $KEY | ce-ai decisions auth --stdin`). It validates non-empty input and immediately saves it to `0600` storage and OS keyring.
- `--key` without value (or `ce-ai decisions auth` in an interactive TTY): Prompts the operator with masked input without echoing characters (`rpassword`), preventing shoulder-surfing and terminal buffer leaks.
- `--key <VAL>`: Emits an explicit security deprecation warning to `stderr` advising use of `--stdin` or interactive prompt.

## Verification

- `cargo fmt --check` passes cleanly.
- `cargo clippy --all-targets --all-features -- -D warnings` reports 0 warnings.
- `cargo test` passes all unit and CLI integration tests with hermetic credentials isolation (`CE_AI_CREDENTIALS_PATH`).
- `ce-ai doctor` runs in 600-800ms with zero GUI prompts and reports `decision-engine: jev active and healthy`.
