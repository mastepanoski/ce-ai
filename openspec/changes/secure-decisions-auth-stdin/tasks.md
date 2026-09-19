# Tasks: Secure Stdin & Masked Input for Decisions Auth

- [x] **Work Unit 1: Secret Ingestion Utilities in `src/decisions/auth.rs`** (~70 LOC)
  - [x] Implement `read_api_key_from_stdin() -> Result<String, CeError>` to read trimmed key from `std::io::stdin()`.
  - [x] Implement `prompt_api_key_interactive(prompt: &str) -> Result<Option<String>, CeError>` using `rpassword` without character echo.
  - [x] Implement OS Keyring storage (`keyring_get`, `keyring_set`, `keyring_delete`) with macOS Keychain, Windows Credential Manager, and Linux Secret Service support, with cross-compatibility for `jevkit`.
  - [x] Unit test `read_api_key_from_reader` with empty vs populated streams in `src/decisions/tests/auth_tests.rs`.
  - [x] Verification: `cargo test --lib decisions::auth`.

- [x] **Work Unit 2: CLI Interface & Flow Integration in `src/commands/decisions.rs`** (~60 LOC)
  - [x] Update `Action::Auth` Clap schema:
    - [x] `key: Option<Option<String>>` with `num_args = 0..=1`.
    - [x] `stdin: bool` flag.
    - [x] `check: bool` flag.
  - [x] In `handle_auth`:
    - [x] If `stdin` is true: read from stdin via `read_api_key_from_stdin()`.
    - [x] If `key` is `Some(Some(k))`: emit stderr security warning and save `k`.
    - [x] If `key` is `Some(None)`: read via `prompt_api_key_interactive` on TTY.
    - [x] If `key` is `None` and `!stdin`: display current status (configured/not set) without blocking.
    - [x] If `check` is true: execute provider health verification.
  - [x] Verification: `cargo check`.

- [x] **Work Unit 3: CLI Integration Tests** (~80 LOC)
  - [x] Add integration tests in `tests/cli.rs` (or `src/commands/tests/decisions_tests.rs`):
    - [x] Piped stdin saving: `echo "test-key-stdin" | ce-ai decisions auth --stdin`.
    - [x] Empty stdin error: `echo "" | ce-ai decisions auth --stdin` exits with code 2.
    - [x] Deprecation warning emitted to stderr when `--key <VAL>` is used.
    - [x] Non-interactive status display without flags does not block on stdin.
  - [x] Verification: `cargo test`.

- [x] **Work Unit 4: Quality Gates, Versioning & CHANGELOG** (~20 LOC)
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
  - [x] Run `make e2e`.
  - [x] Bump minor version in `Cargo.toml` (`1.64.0`).
  - [x] Update `CHANGELOG.md` following Keep a Changelog.
  - [x] Verification: 100% green local test suite.

