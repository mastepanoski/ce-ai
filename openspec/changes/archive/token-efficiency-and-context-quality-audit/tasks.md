> STATUS (v1.20.1): ce-ai audit engine live in src/commands/audit.rs. Residual open boxes below were not re-audited item-by-item.

# OpenSpec Tasks: Token Efficiency & Context Quality Audit

- **Change:** `token-efficiency-and-context-quality-audit`
- **Issue:** #117

---

## 📋 Task Checklist

- [ ] **Task 1**: Create `src/commands/audit.rs` defining `Args`, `AuditReport`, `AuditCheck`, `AuditStatus`, `Detector` trait, and builtin detectors.
- [ ] **Task 2**: Register `Audit(audit::Args)` subcommand in `src/main.rs`.
- [ ] **Task 3**: Implement detectors:
  - `CliCompressionDetector` (`rtk` binary + hook matcher)
  - `McpSprawlDetector` (MCP server count >5 check)
  - `PromptDuplicationDetector` ($\ge$ 200 char block scan across agents)
  - `PersistentMemoryDetector` (`engram` DB scan)
  - `DocsGroundingDetector` (`context7` entry)
  - `CodeIntelDetector` (`.codegraph/` index scan)
  - `LearningsLibraryDetector` (`docs/solutions/` doc scan)
- [ ] **Task 4**: Implement `--json` rendering and `--fail-under <pct>` threshold enforcement.
- [ ] **Task 5**: Add unit tests in `src/commands/audit.rs` and CLI integration tests in `tests/cli.rs`.
- [ ] **Task 6**: Verify formatting (`cargo fmt --check`), strict clippy (`cargo clippy --all-targets --all-features -- -D warnings`), and test suite (`cargo test`).
