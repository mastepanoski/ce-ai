# Tasks: Living System Specifications & Delta Promotion Engine

## Implementation Units & TDD Checklist

### Phase 1: Foundational Domain Specifications

- [ ] **Unit 1: Canonical Living Domain Specifications Seeding** (~250 LOC markdown)
  - [ ] 1.1 Author `openspec/specs/harnesses.md` with multi-harness architecture, registration contracts, and install manifests.
  - [ ] 1.2 Author `openspec/specs/workflow.md` with 7-stage compound engineering lifecycle, FSM, Turn-0 delivery, and checkpoints.
  - [ ] 1.3 Author `openspec/specs/doctor.md` with health check subsystem, diagnostic probes, and non-blocking warnings.
  - [ ] 1.4 Author `openspec/specs/state.md` with atomic writes, state schema, model profiles, and `.ce-ai.json` overrides.
  - [ ] 1.5 Author `openspec/specs/archive.md` with mechanical & STATUS-attested criteria, compaction, and milestone rollups.
  - [ ] 1.6 Author `openspec/specs/gate.md` with blocking gate check, validation receipts, and observe-only mode.
  - [ ] 1.7 Author `openspec/specs/project-adoption.md` with init-prj/deinit-prj, managed blocks, and tiers.

### Phase 2: Core Domain Spec Engine & CLI Handlers

- [ ] **Unit 2: Domain Spec In-Memory Models & Validator in `src/commands/spec.rs`** (~180 LOC)
  - [ ] 2.1 Implement `DomainSpecMetadata`, `DomainSpecSummary`, and `SpecValidationReport` data structures with Serde support.
  - [ ] 2.2 Implement `parse_domain_spec_frontmatter` and frontmatter extraction from markdown boundary markers (`---`).
  - [ ] 2.3 Implement `validate_domain_specs(repo_root)` verifying required frontmatter keys, domain-file slug match, and capability sections.
  - [ ] 2.4 Implement `list_domain_specs(repo_root)` computing requirement counts and last-updated metadata.
  - [ ] 2.5 Add unit tests in `src/commands/tests/spec.rs` for metadata parsing and validation failure cases.

- [ ] **Unit 3: CLI Subcommand Integration (`ce-ai spec`)** (~120 LOC)
  - [ ] 3.1 Register `SpecArgs` and `SpecCommand` enum in `src/commands/registry.rs` (`list`, `show`, `validate`, `promote`).
  - [ ] 3.2 Wire `Commands::Spec` dispatch in `src/commands/registry.rs` to `spec::run_spec(ctx, args)`.
  - [ ] 3.3 Implement `run_spec_list` supporting formatted tabular output and `--json`.
  - [ ] 3.4 Implement `run_spec_show` displaying complete specification content with exit code 2 on missing domain.
  - [ ] 3.5 Implement `run_spec_validate` outputting validation status and exiting with code 6 on structural errors.

### Phase 3: Requirement Delta Promotion Engine

- [ ] **Unit 4: Requirement Delta Promotion (`ce-ai spec promote` & `ce-ai archive --promote`)** (~190 LOC)
  - [ ] 4.1 Implement `extract_requirements_from_spec_md(spec_path)` parsing RFC 2119 requirement headers and bodies.
  - [ ] 4.2 Implement `promote_delta_to_domain_spec(repo_root, change_name, target_domain, dry_run)` merging new requirements with provenance annotations into `openspec/specs/<domain>.md`.
  - [ ] 4.3 Use `crate::state::write_atomic` to persist mutated domain specifications safely.
  - [ ] 4.4 Extend `ArchiveArgs` in `src/commands/workflow.rs` with `--promote` and `--domain <name>` flags.
  - [ ] 4.5 Wire automated promotion step into `workflow::run_archive` during change archival.

### Phase 4: Quality Verification, Doctor Integration & Docs

- [ ] **Unit 5: Doctor Health Probe Integration & Tests** (~200 LOC)
  - [ ] 5.1 Implement `probe_specs_health(repo_root)` auditing `openspec/specs/` presence, domain spec validity, and active changes domain tags.
  - [ ] 5.2 Wire non-blocking advisories into `src/commands/doctor.rs`.
  - [ ] 5.3 Implement comprehensive CLI integration tests in `tests/cli.rs` (`test_cli_spec_list_show_validate`, `test_cli_spec_promote_dry_run_and_apply`, `test_cli_archive_with_promote`).
  - [ ] 5.4 Update `CONCEPTS.md`, `README.md` (≤ 100 lines), and `CHANGELOG.md`.
