# Tasks: Implementation Checklist for Binary Self-Update

Work-Unit Budget: ~780 LOC total forecast across 5 atomic units (~150 LOC avg per unit).
Follows TDD methodology: Test-First (RED) ➔ Implementation (GREEN) ➔ Refactor.

---

### Work Unit 1: Target Resolution, Checksum Parsing & Integrity Verification (`src/source/binary_release.rs`)
**Forecast**: ~130 changed lines.
**Focus**: Platform detection, SHA256 checksum parsing and fail-closed integrity validation.

- [x] **1.1 Unit Tests (RED)**:
  - Create `src/source/tests/binary_release.rs`.
  - Add test `test_current_target_resolves_known_triples`.
  - Add test `test_asset_name_for_target_formats_tar_gz_and_zip`.
  - Add test `test_parse_sha256sums_extracts_correct_digest`.
  - Add test `test_parse_sha256sums_handles_missing_asset`.
  - Add test `test_verify_checksum_accepts_matching_sha256`.
  - Add test `test_verify_checksum_rejects_mismatch_with_verification_error`.
- [x] **1.2 Implementation (GREEN)**:
  - Create `src/source/binary_release.rs`.
  - Implement `current_target() -> Option<&'static str>`.
  - Implement `asset_name_for_target(target: &str) -> String`.
  - Implement `parse_sha256sums(content: &str, asset_name: &str) -> Option<String>`.
  - Implement `verify_checksum(bytes: &[u8], expected_sha256: &str) -> Result<(), CeError>`.
- [x] **1.3 Refactor & Verify**:
  - Run `cargo test --lib source::tests::binary_release`. Verify all pass green.

---

### Work Unit 2: Safe Archive Extraction for `.tar.gz` and `.zip` (`src/source/binary_release.rs`, `Cargo.toml`)
**Forecast**: ~140 changed lines.
**Focus**: Path traversal defense (Zip-Slip) and in-memory executable extraction.

- [x] **2.1 Unit Tests (RED)**:
  - Add test `test_extract_binary_from_tar_gz_success`.
  - Add test `test_extract_binary_from_zip_success`.
  - Add test `test_extract_binary_rejects_zip_slip_traversal_tar`.
  - Add test `test_extract_binary_rejects_zip_slip_traversal_zip`.
  - Add test `test_extract_binary_rejects_empty_archive`.
- [x] **2.2 Implementation (GREEN)**:
  - Add `zip = { version = "2.2", default-features = false, features = ["deflate"] }` to `Cargo.toml`.
  - Implement `extract_binary_from_tar(bytes: &[u8]) -> Result<Vec<u8>, CeError>`.
  - Implement `extract_binary_from_zip(bytes: &[u8]) -> Result<Vec<u8>, CeError>`.
  - Implement `extract_binary(bytes: &[u8], is_zip: bool) -> Result<Vec<u8>, CeError>`.
- [x] **2.3 Refactor & Verify**:
  - Run `cargo test --lib source::tests::binary_release`. Verify all pass green.

---

### Work Unit 3: Cross-Platform Executable Replacement & Cleanup Engine (`src/source/binary_release.rs`, `src/main.rs`)
**Forecast**: ~160 changed lines.
**Focus**: POSIX atomic rename and Windows `.old` swap and startup cleanup.

- [x] **3.1 Unit Tests (RED)**:
  - Add test `test_replace_executable_posix_atomic_swap_and_permissions`.
  - Add test `test_replace_executable_fails_on_unwritable_parent_directory`.
  - Add test `test_cleanup_stale_update_files_removes_old_and_temp_files`.
- [x] **3.2 Implementation (GREEN)**:
  - Implement `replace_executable(target_path: &Path, binary_bytes: &[u8]) -> Result<(), CeError>`.
  - Implement POSIX branch: temp file in same directory, `chmod 0o755`, atomic `rename`.
  - Implement Windows branch: rename running `.exe` to `.old`, move new to `.exe`, best-effort delete `.old`.
  - Implement `cleanup_stale_update_files(dir: &Path)` in `src/source/binary_release.rs`.
  - Call `cleanup_stale_update_files` in `src/main.rs` before command execution.
- [x] **3.3 Refactor & Verify**:
  - Run `cargo test --lib source::tests::binary_release`. Verify all pass green.

---

### Work Unit 4: Release Resolution & Network Fetching (`src/source/binary_release.rs`)
**Forecast**: ~170 changed lines.
**Focus**: Sourcing latest release, `SHA256SUMS.txt`, and asset URLs from `mastepanoski/ce-ai`.

- [x] **4.1 Unit Tests (RED)**:
  - Add test `test_parse_cli_release_payload`.
  - Add test `test_extract_tag_from_cli_redirect_url`.
  - Add test `test_extract_latest_tag_from_cli_atom_feed`.
  - Add test `test_compare_cli_versions`.
- [x] **4.2 Implementation (GREEN)**:
  - Implement `compare_cli_versions(a: &str, b: &str) -> std::cmp::Ordering`.
  - Implement `resolve_latest_cli_release(client: &reqwest::blocking::Client, token: Option<&str>) -> Result<CliRelease, CeError>`.
  - Implement zero-friction fallback via `/releases/latest` redirect and `/releases.atom` feed.
  - Implement `download_release_asset_and_sums(client: &reqwest::blocking::Client, release: &CliRelease, target: &str) -> Result<(Vec<u8>, String), CeError>`.
- [x] **4.3 Refactor & Verify**:
  - Run `cargo test --lib source::tests::binary_release`. Verify all pass green.

---

### Work Unit 5: CLI Subcommand `ce-ai self-update` & `ce-ai upgrade --bin` (`src/commands/self_update.rs`, `src/commands/upgrade.rs`, `src/commands/registry.rs`)
**Forecast**: ~180 changed lines.
**Focus**: Command orchestration, CLI arguments (`--check`, `--to`, `--force`), dry-run handling, and upgrade flag.

- [x] **5.1 Unit & Integration Tests (RED)**:
  - Add test `test_self_update_check_flag_does_not_mutate_disk`.
  - Add test `test_self_update_already_up_to_date_exits_zero`.
  - Add test `test_self_update_dry_run_previews_replacement`.
  - Add test `test_upgrade_command_bin_flag_wiring`.
- [x] **5.2 Implementation (GREEN)**:
  - Create `src/commands/self_update.rs` with `Args` and `run(ctx, args)`.
  - Register `Commands::SelfUpdate` in `src/commands/registry.rs`.
  - Add `--bin` flag to `src/commands/upgrade.rs` and invoke self-update on demand.
  - Add doctor suggestion on successful update: `Updated ce-ai from vX to vY! Run 'ce-ai doctor' to verify health.`
- [x] **5.3 Verification & Quality Gates**:
  - Run `cargo fmt --check`.
  - Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - Run `cargo test`.
