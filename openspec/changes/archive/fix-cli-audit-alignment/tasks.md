# Tasks: CLI audit alignment

- [x] **Unit 1 (~90 LOC):** Add failing usage-report filtering/validation
  tests; implement RFC 3339 interval filtering and explicit `--by` handling.
  Verify targeted usage tests.
- [x] **Unit 2 (~45 LOC):** Add a guard scope-mismatch regression test and
  validate `guard disable --harness` before persisting state. Verify targeted
  guard tests.
- [x] **Unit 3 (~180 LOC):** Correct help/comments and publish a concise CLI
  reference; repair known invalid user-guide examples and README routing.
  Verify `ce-ai doc lint --strict` and README line count.
- [x] **Unit 4 (~0 LOC):** Run `cargo fmt --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and
  `make e2e`.
