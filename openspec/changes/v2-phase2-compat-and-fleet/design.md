# Technical Design: Phase 2 — CE Compatibility Layer & Fleet Subsystem

## System Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                       CE-AI CLI                             │
│       fleet status  │  fleet pin  │  fleet sync             │
└──────────────┬───────────────────────────────┬──────────────┘
               │                               │
               ▼                               ▼
┌─────────────────────────────┐ ┌─────────────────────────────┐
│        src/fleet/           │ │        src/compat/          │
│  Fleet Governance Engine    │ │   CE Compatibility Layer    │
│                             │ │                             │
│ ┌─────────────────────────┐ │ │ ┌─────────────────────────┐ │
│ │ FleetHarnessDriver      │ │ │ │ CeDocsConfig            │ │
│ │ • detect_version()      │ │ │ │ • discover docs_root    │ │
│ │ • plan_sync()           │ │ │ │ • plans_dir()           │ │
│ └─────────────────────────┘ │ │ │ • solutions_dir()       │ │
│                             │ │ └─────────────────────────┘ │
│ ┌─────────────────────────┐ │ │ ┌─────────────────────────┐ │
│ │ Native Harness Drivers  │ │ │ │ CeSolutionFrontmatter   │ │
│ │ • ClaudeDriver          │ │ │ │ • upstream schema.yaml  │ │
│ │ • OpenCodeDriver        │ │ │ └─────────────────────────┘ │
│ │ • PiDriver              │ │ │ ┌─────────────────────────┐ │
│ │ • CodexDriver           │ │ │ │ CeSkillContract         │ │
│ └─────────────────────────┘ │ │ │ • mode:return-to-caller │ │
│                             │ │ └─────────────────────────┘ │
│ ┌─────────────────────────┐ │ │ ┌─────────────────────────┐ │
│ │ FleetStatus & Reports   │ │ │ │ CeRelease               │ │
│ │ • aligned / divergent   │ │ │ │ • tag / semver parsing  │ │
│ └─────────────────────────┘ │ │ └─────────────────────────┘ │
└──────────────┬──────────────┘ └─────────────────────────────┘
               │
               ▼
┌─────────────────────────────┐
│    state.json (atomic)      │
│  "fleet": { "pinned": ... } │
└─────────────────────────────┘
```

---

## 1. Compatibility Layer (`src/compat/`)

### `src/compat/docs.rs`
Provides dynamic docs path resolution, respecting `.compound-engineering/config.yaml` and `.local.yaml`:
```rust
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CeDocsConfig {
    pub docs_root: PathBuf,
}

impl CeDocsConfig {
    pub fn discover(repo_root: &Path) -> Self;
    pub fn plans_dir(&self, repo_root: &Path) -> PathBuf;
    pub fn solutions_dir(&self, repo_root: &Path) -> PathBuf;
    pub fn parse_docs_root(config_file: &Path) -> Option<PathBuf>;
}
```

### `src/compat/schema.rs`
Full Serde mapping for upstream `schema.yaml`:
```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CeSolutionFrontmatter {
    #[serde(default)]
    pub schema_version: Option<String>,
    pub module: String,
    pub date: NaiveDate,
    pub problem_type: String, // bugfix, architecture, pattern, discovery, config, preference
    pub component: String,
    #[serde(default)]
    pub related_components: Vec<String>,
    pub severity: String, // critical, major, standard, minor
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub applies_when: Option<String>,
}

impl CeSolutionFrontmatter {
    pub fn parse(yaml_str: &str) -> Result<Self, String>;
    pub fn is_bug_track(&self) -> bool;
}
```

### `src/compat/contracts.rs`
Documented mode tokens and integration contracts:
```rust
pub const MODE_RETURN_TO_CALLER: &str = "mode:return-to-caller";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CeSkillContract {
    pub name: &'static str,
    pub return_to_caller_supported: bool,
    pub primary_output_field: Option<&'static str>,
}

pub fn get_canonical_skill_contract(skill_name: &str) -> Option<CeSkillContract>;
```

### `src/compat/release.rs`
Released tag parsing and comparison:
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CeRelease {
    pub tag: String,
    pub version: String,
}

impl CeRelease {
    pub fn parse_tag(tag: &str) -> Option<Self>;
    pub fn compare(&self, other: &Self) -> std::cmp::Ordering;
}
```

---

## 2. Fleet Subsystem (`src/fleet/`)

### `src/fleet/driver.rs`
```rust
use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FleetAction {
    UpToDate,
    RunCommand {
        program: String,
        args: Vec<String>,
        description: String,
    },
    UpdateConfig {
        file_path: std::path::PathBuf,
        key: String,
        value: serde_json::Value,
        description: String,
    },
}

pub trait FleetHarnessDriver: Send + Sync {
    fn harness_kind(&self) -> HarnessKind;
    fn detect_version(&self, ctx: &Context) -> Result<Option<String>, CeError>;
    fn plan_sync(&self, ctx: &Context, target_version: &str) -> Result<FleetAction, CeError>;
}
```

### Supported Harness Drivers:
- **`OpenCodeDriver`**: Inspects `opencode.json` plugins and materializes the plugin entry with version pinning.
- **`ClaudeDriver`**: Inspects `~/.claude/` or installed plugin manifest for `compound-engineering` version; generates `claude plugin update/install` action.
- **`PiDriver`**: Inspects `.pi/extensions/` npm package versions; generates npm sync action.
- **`CodexDriver` / `CursorDriver`**: Inspects native instructions and rules configuration.

### `src/fleet/status.rs`
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HarnessAlignment {
    Aligned,
    Divergent { installed: String, pinned: String },
    Missing { pinned: String },
    Unmanaged { installed: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FleetStatusReport {
    pub pinned_version: Option<String>,
    pub harnesses: Vec<FleetHarnessStatus>,
    pub is_aligned: bool,
}
```

---

## 3. State Schema Addition

In `src/state/state.rs`:
```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetState {
    pub pinned_version: Option<String>,
    pub last_sync: Option<String>,
}

// Added to State struct:
#[serde(default)]
pub fleet: FleetState,
```

---

## 4. CLI Commands & User Interface

- `ce-ai fleet status`:
  - Renders a clean table showing each harness, installed version, pinned target version, and alignment status.
  - Supports `--json` for machine consumption.
- `ce-ai fleet pin <version>`:
  - Validates version string (or fetches release tag).
  - Persists `state.fleet.pinned_version` atomically via `write_atomic`.
- `ce-ai fleet sync [--dry-run]`:
  - Iterates over all active harnesses, runs `plan_sync()`, displays execution plan (or executes actions in parallel).
