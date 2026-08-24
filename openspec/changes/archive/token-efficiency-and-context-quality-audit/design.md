# OpenSpec Technical Design: Data Schemas & CLI Contracts

- **Change:** `token-efficiency-and-context-quality-audit`
- **Issue:** #117

---

## 📐 1. Data Schemas & Rust Structs

```rust
#[derive(clap::Args, Debug, Default)]
pub struct Args {
    /// Render machine-readable JSON output.
    #[arg(long)]
    pub json: bool,
    /// Exit with non-zero code if score falls below specified percentage (0-100).
    #[arg(long)]
    pub fail_under: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditReport {
    pub harnesses_detected: Vec<String>,
    pub checks: Vec<AuditCheck>,
    pub score_percentage: u32,
    pub pass_count: usize,
    pub warn_count: usize,
    pub fail_count: usize,
}
```

---

## 💻 2. CLI Command Ergonomics

### A. Default Output
```text
== [ce-ai Agent Environment Audit] ==
harnesses detected: opencode, claude

[repo]      PASS code-intelligence        .codegraph/ index present
[repo]      PASS learnings-library        docs/solutions/ (12 docs)
[tokens]    WARN mcp-sprawl/opencode      10 servers configured globally (>5)
[tokens]    PASS cli-compression/claude   satisfied-by: rtk 0.45.0 (hook wired)
[grounding] PASS persistent-memory        satisfied-by: engram
[grounding] PASS docs-grounding           satisfied-by: context7

score: 83% (5 pass / 1 warn / 0 fail)
```

### B. Machine-Readable JSON Output (`--json`)
```json
{
  "harnesses_detected": ["opencode", "claude"],
  "score_percentage": 83,
  "pass_count": 5,
  "warn_count": 1,
  "fail_count": 0,
  "checks": [...]
}
```
