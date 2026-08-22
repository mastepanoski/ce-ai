# Tasks: Adoption Block SSOT Guidance (v2)

TDD order: Red ➔ Green per unit. Verify each unit with `cargo test --lib` /
targeted `cargo test <name>` before moving on.

- [ ] Unit 1: Introduce `pub const BLOCK_VERSION: u32 = 2;` and wire it into
      both the header `format!` and `ProjectAdoptionEntry.block_version`.
      - [ ] Red: update existing assertion in
            `init_prj_preserves_preexisting_content_and_crlf` to expect
            `v=2 tier=minimal` and watch it fail against current literal.
      - [ ] Green: apply constant; test passes.
- [ ] Unit 2: Append SSOT section to `full` tier string in
      `render_block_content`.
      - [ ] Red: new test asserts rendered full-tier block contains
            `Single Source of Truth Rule` and `already clear`.
      - [ ] Green: implement.
- [ ] Unit 3: Append single distillation line to `orchestrator` tier.
      - [ ] Red: new test asserts `distill, never duplicate` present exactly
            once.
      - [ ] Green: implement.
- [ ] Unit 4: Pin `minimal` tier regression guard (byte-equality against v1
      string). Red first: test references expected constant; fails if drift.
- [ ] Unit 5: Integration test — v1 ➔ v2 in-place replacement.
      - [ ] Red: hand-write v1 block into temp-project `AGENTS.md` (with CRLF
            + surrounding text), run `init-prj`, assert v2 content between
            markers, preserved surroundings, `state.json`
            `block_version == 2`.
      - [ ] Green: confirm existing replacement logic satisfies it (expected:
            no production change needed).
- [ ] Unit 6: Integration test — idempotent second run reports up-to-date and
      leaves file bytes untouched.
- [ ] Unit 7: Docs — update `docs/user-guide/project-adoption-guide.md` tier
      table/description with v2 contents + re-run upgrade path.
- [ ] Unit 8: Ship prep — SemVer bump (`Cargo.toml`, `Formula/ce-ai.rb`),
      `CHANGELOG.md` entry under Unreleased.
- [ ] Unit 9: Full gate — `cargo fmt --check`, `cargo clippy --all-targets
      --all-features -- -D warnings`, `cargo test`, `make e2e`.
