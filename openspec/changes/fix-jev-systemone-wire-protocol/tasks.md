# Tasks: Remediate Hallucinated Jev Endpoint to TypeSafe System One Wire Protocol

- [x] **Work Unit 1: Centralize System One Wire Protocol Types in `src/decisions/types.rs`** (~120 LOC)
  - [x] Move `SystemOneWireQuestion`, `SystemOneWireRequest`, `SystemOneWireAnswer`, and `SystemOneWireResponse` into `src/decisions/types.rs`.
  - [x] Add `build_systemone_questions` helper in `types.rs`.
  - [x] Add `parse_systemone_answers` helper in `types.rs`.
  - [x] Re-export wire types in `src/decisions/kev.rs` and update `src/decisions/laya.rs` imports for backward compatibility.
  - [x] Verification: `cargo check`.

- [x] **Work Unit 2: Refactor `JevProvider` to System One Protocol in `src/decisions/jev.rs`** (~90 LOC)
  - [x] Replace `JevWireRequest` and `JevWireResponse` with `SystemOneWireRequest` and `SystemOneWireResponse`.
  - [x] Update `build_wire_payload` to assemble `state` and map questions using `build_systemone_questions`.
  - [x] Update `parse_wire_response` to decode `answers` using `parse_systemone_answers` and extract `estimated_cost_usd` from `usage`.
  - [x] Update `evaluate` URL target from `/decide` to `/systemone` (or `/v1/systemone`).
  - [x] Verification: `cargo check`.

- [x] **Work Unit 3: Update and Expand Unit Tests in `src/decisions/tests/jev_tests.rs`** (~80 LOC)
  - [x] Update `test_jev_wire_payload_serialization_round_trip` to assert `SystemOneWireRequest` structure (`state`, `model`, `questions` with `noul`, `choice`).
  - [x] Update `test_jev_wire_response_parsing` to assert `SystemOneWireResponse` structure with `noul`, `choice`, `score`, and `usage`.
  - [x] Add wire round-trip test covering `score` question and answer.
  - [x] Verification: `cargo test --lib decisions::tests::jev_tests`.

- [x] **Work Unit 4: Fix Companion Tool Detection & End-to-End Verification** (~40 LOC)
  - [x] Update `init_codegraph` in `src/commands/tools.rs` to verify `.codegraph/codegraph.db` exists before assuming indexed.
  - [x] Test `cargo run -- tools init codegraph .`.
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
  - [x] Run `make e2e`.
  - [x] Verification: 100% green test suite.

- [ ] **Work Unit 5: Version Bump, Changelog, Knowledge Capture & Release** (~30 LOC)
  - [x] Bump version in `Cargo.toml` to `1.68.2` and update `Cargo.lock`.
  - [x] Add release entry in `CHANGELOG.md` following Keep a Changelog.
  - [x] Create solution document in `docs/solutions/`.
  - [ ] Commit, push branch `fix/jev-systemone-wire-protocol`, create PR (`gh pr create`).
  - [ ] Watch CI (`gh pr checks --watch`) and merge.
  - [ ] Tag `v1.68.2`, create GitHub Release (`gh release create`).
  - [ ] Post-merge lifecycle: pull main, archive OpenSpec change package (`ce-ai archive fix-jev-systemone-wire-protocol`), submit archive PR.
