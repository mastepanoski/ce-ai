# OpenSpec Design: Multi-Harness Skill Registry Engine

- **Feature Name**: `skill_registry_engine`
- **Issue Reference**: #96
- **Status**: Draft / Proposed

---

## 1. Data Schemas & Rust Structs

### `SkillEntry` Struct (`src/source/registry.rs`)
```rust
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub scope: String,
    pub triggers: Vec<String>,
    pub sha256: String,
    /// Absolute paths per harness kind (e.g. "opencode" => "/path/to/SKILL.md")
    pub harness_paths: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SkillRegistry {
    pub version: String,
    pub updated_at: String,
    pub skills: Vec<SkillEntry>,
}
```

---

## 2. Component Integration

```
[src/commands/install.rs] ──┐
[src/commands/sync.rs]    ──┼──> [src/source/registry.rs] ──> [~/.ce-ai/skills-registry.json]
[src/commands/upgrade.rs] ──┘         (write_atomic)
                                          │
                                          ▼
                                [ce-ai skills list]
                                [ce-ai skills resolve]
                                [ce-ai doctor]
```

---

## 3. Subcommand Contracts

### `ce-ai skills list [--harness <name>] [--json]`
Lists all indexed skills with scope, target harnesses, and health status.

### `ce-ai skills resolve --query "<keyword>" [--harness <name>]`
Searches skills by trigger or description keywords and outputs absolute `SKILL.md` paths ready for prompt injection.

### `ce-ai skills doctor`
Audits `skills-registry.json` integrity:
- Validates missing `SKILL.md` files on disk.
- Verifies SHA256 hashes against actual file contents.
- Flags malformed YAML frontmatter headers.
