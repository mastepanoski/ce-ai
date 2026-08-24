> STATUS (v1.20.1): Tools probes live in src/commands/doctor.rs (#112). Residual open boxes below were not re-audited item-by-item.

# OpenSpec Tasks: Companion-Tool Readiness & Version Freshness

- **Change:** `companion-tool-readiness-and-freshness`
- **Issue:** #112
- **Author:** Antigravity AI
- **Date:** 2026-08-22

---

## 📋 Task Checklist & TDD Execution Steps

### Unit 1: Embedded Tools Registry & 24h Local Cache (`src/source/tools_registry.rs`)
- [ ] **Task 1.1**: Create `src/source/tools_registry.rs` defining `FreshnessStatus` enum (`Ok`, `Outdated`, `Missing`, `Offline`) and `CompanionToolInfo` struct.
- [ ] **Task 1.2**: Implement embedded default constants for Engram (`v1.2.0`), CodeGraph (`v0.5.0`), Context7 (`v1.0.0`), RTK (`v0.2.1`), and `ce-ai` (`env!("CARGO_PKG_VERSION")`).
- [ ] **Task 1.3**: Implement local cache reader/writer targeting `~/.ce-ai/cache/companion-registry.json` with `write_atomic` and 24-hour TTL validation.
- [ ] **Task 1.4**: Implement non-blocking HTTP manifest refresh with 500ms timeout and graceful `(offline)` fallback.
- [ ] **Task 1.5**: Add unit tests in `src/source/tools_registry.rs` for version parsing, SemVer comparisons, TTL expiration, and offline degradation.

### Unit 2: Enhanced Status Command (`src/commands/tools.rs`)
- [ ] **Task 2.1**: Update `ce-ai tools status` to extract version strings from installed binaries (`engram --version`, `codegraph --version`, `rtk --version`, `ce-ai --version`).
- [ ] **Task 2.2**: Integrate `tools_registry.rs` to compute `FreshnessStatus` for each sidecar.
- [ ] **Task 2.3**: Render actionable remediation hints for missing or outdated tools (e.g. `ce-ai tools install codegraph`, `ce-ai upgrade`).
- [ ] **Task 2.4**: Add Skill Registry suggestions section for missing skills (e.g. `sequential-thinking`).
- [ ] **Task 2.5**: Update `--json` machine-readable output contract in `src/commands/tools.rs`.

### Unit 3: Doctor Readiness Probes & `--strict` Flag (`src/commands/doctor.rs`)
- [ ] **Task 3.1**: Add `--strict` boolean flag argument to `doctor.rs`.
- [ ] **Task 3.2**: Integrate `tools_registry.rs` readiness probes into `ce-ai doctor`.
- [ ] **Task 3.3**: Enforce exit code policy: `Missing` tools push findings (Exit 1); `Outdated` tools emit `doctor-info:` (Exit 0), unless `--strict` is set.
- [ ] **Task 3.4**: Add self-update notification hint when `ce-ai` is behind the latest release.
- [ ] **Task 3.5**: Add CLI integration tests in `tests/cli.rs` testing `ce-ai doctor`, `ce-ai doctor --strict`, and `ce-ai tools status`.

### Unit 4: Verification & E2E Gate
- [ ] **Task 4.1**: Verify zero Clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
- [ ] **Task 4.2**: Verify formatting compliance (`cargo fmt --check`).
- [ ] **Task 4.3**: Run unit and CLI integration tests (`cargo test`).
- [ ] **Task 4.4**: Run containerized Docker E2E gate (`make e2e`).
