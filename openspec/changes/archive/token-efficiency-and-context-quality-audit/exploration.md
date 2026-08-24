# OpenSpec Exploration: Technical Investigation & Architecture Options

- **Change:** `token-efficiency-and-context-quality-audit`
- **Issue:** #117

---

## 🔍 1. Technical Investigation

### A. Detector Architecture & Extensibility
To prevent hard-coding single vendors, the audit module utilizes a plugin/detector architecture (`Detector` trait):

```rust
pub enum AuditStatus { Pass, Warn, Fail, Info }

pub struct AuditCheck {
    pub id: String,
    pub category: String,
    pub status: AuditStatus,
    pub satisfied_by: Option<String>,
    pub detail: String,
}

pub trait Detector {
    fn name(&self) -> &str;
    fn detect(&self, ctx: &AuditCtx) -> AuditCheck;
}
```

### B. Detector Implementations
1. `CliCompressionDetector`: Verifies `rtk` binary on PATH and hook configuration in Claude settings / OpenCode plugins.
2. `McpSprawlDetector`: Parses MCP servers map in `opencode.json` and `claude.json`; triggers `WARN` if >5 global servers.
3. `PromptDuplicationDetector`: Scans agent instructions across harness configs, finding identical blocks ($\ge$ 200 chars) shared by 3 or more agents.
4. `PersistentMemoryDetector`: Checks `engram` DB existence or equivalent memory provider.
5. `DocsGroundingDetector`: Checks `context7` provider entry.
6. `CodeIntelDetector`: Checks `.codegraph/` directory at repository root.
7. `LearningsLibraryDetector`: Checks `<repo>/docs/solutions/` directory and counts solution markdown files.

---

## 💡 2. Architectural Decision

**Decision**: Implement static detector registry `BUILTIN_DETECTORS` in `src/commands/audit.rs`. Injectable `AuditCtx` allows hermetic testing with `tempfile::TempDir` fixtures without touching local `HOME` or global state.
