# OpenSpec Technical Design: Data Schemas, Structs & CLI Ergonomics

- **Change:** `companion-tool-readiness-and-freshness`
- **Issue:** #112
- **Author:** Antigravity AI
- **Date:** 2026-08-22

---

## 🏛️ 1. Module Structure & Architecture

```
src/
├── source/
│   └── tools_registry.rs    # Companion tool registry & version freshness engine
├── commands/
│   ├── doctor.rs            # Integrated doctor readiness probes & --strict flag
│   └── tools.rs             # Enhanced tools status with version numbers & suggestions
```

---

## 📐 2. Data Schemas & Rust Structs

### A. `FreshnessStatus` Enum (`src/source/tools_registry.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FreshnessStatus {
    Ok { version: String },
    Outdated { current: String, expected: String },
    Missing,
    Offline { current: String },
}
```

### B. `CompanionToolInfo` Struct
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompanionToolInfo {
    pub name: String,
    pub label: String,
    pub category: String,
    pub min_version: String,
    pub latest_version: String,
    pub install_cmd: String,
}
```

### C. `ToolsRegistry` Cache File (`~/.ce-ai/cache/companion-registry.json`)
```json
{
  "updated_at": "2026-08-22T21:40:00Z",
  "tools": {
    "engram": { "min_version": "1.2.0", "latest_version": "1.2.0", "install_cmd": "ce-ai tools install engram" },
    "codegraph": { "min_version": "0.5.0", "latest_version": "0.5.0", "install_cmd": "ce-ai tools install codegraph" },
    "context7": { "min_version": "1.0.0", "latest_version": "1.0.0", "install_cmd": "ce-ai tools install context7" },
    "rtk": { "min_version": "0.2.1", "latest_version": "0.2.1", "install_cmd": "ce-ai tools install rtk" }
  },
  "skills": {
    "sequential-thinking": { "install_cmd": "ce-ai skills resolve sequential-thinking" }
  }
}
```

---

## 💻 3. CLI Flags & Command Extensions

### A. `ce-ai doctor [--strict]`
- Adds `--strict` flag to `doctor.rs` args.
- Default behavior (`ce-ai doctor`):
  - `Missing` tool $\rightarrow$ pushes `finding` (fails doctor with Exit 1).
  - `Outdated` tool $\rightarrow$ prints `doctor-info: <tool> outdated (...)` (Exit 0).
- Strict behavior (`ce-ai doctor --strict`):
  - `Missing` OR `Outdated` tool $\rightarrow$ pushes `finding` (fails doctor with Exit 1).

### B. `ce-ai tools status [--json]`
- Renders human-readable summary or machine-readable JSON containing:
  - Companion tool statuses with versions.
  - Skill Registry suggestions.
  - `ce-ai` orchestrator self-update status.

---

## 🔒 4. Security & Governance Compliance

- **Atomic Writes**: `write_atomic` for updating `~/.ce-ai/cache/companion-registry.json`.
- **Preservation of Custom Configs**: MCP server insertion preserves all existing unmanaged user entries in `opencode.json` / `claude.json`.
- **Standard Exit Codes**: `CeError::Runtime(1)` on `--strict` failure or missing tool.
