# Technical Design: Living System Specifications & Delta Promotion Engine

## 1. System Architecture & Component Interactions

The living system specifications subsystem bridges active change deltas in `openspec/changes/` with permanent capability contracts in `openspec/specs/`:

```
┌──────────────────────────────────────┐       ┌──────────────────────────────────────┐
│       openspec/changes/<feat>/       │       │           openspec/specs/            │
│  - proposal.md                       │       │  - harnesses.md                      │
│  - spec.md  (domain: harnesses)      │       │  - workflow.md                       │
│  - tasks.md                          │       │  - doctor.md                         │
└──────────────────┬───────────────────┘       │  - state.md                          │
                   │                           │  - archive.md                        │
                   │ ce-ai archive --promote   │  - gate.md                           │
                   ▼                           │  - project-adoption.md               │
┌──────────────────────────────────────┐       └──────────────────▲───────────────────┘
│       ce-ai spec promotion engine    │                          │
│  - Parse delta requirements          │──────────────────────────┘
│  - Atomic merge into domain spec     │ (atomic write)
│  - Validate schema compliance        │
└──────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│           ce-ai doctor               │
│  - probe_specs_health                │
│  - Validates formatting & coverage   │
└──────────────────────────────────────┘
```

## 2. CLI Interface & Command Structure

### 2.1 Subcommand Registration (`src/commands/registry.rs`)

Add `spec` subcommand to Clap CLI parser:

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands ...
    /// Manage living system specifications in openspec/specs/.
    Spec(SpecArgs),
}

#[derive(Args, Debug, Clone)]
pub struct SpecArgs {
    #[command(subcommand)]
    pub command: SpecCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SpecCommand {
    /// List all living domain specifications.
    List {
        /// Emit JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Show the specification for a specific domain.
    Show {
        /// Domain name (e.g. harnesses, doctor, workflow).
        domain: String,
    },
    /// Validate all domain specifications against schema rules.
    Validate {
        /// Emit JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Promote delta requirements from an OpenSpec change into its target domain spec.
    Promote {
        /// OpenSpec change name to promote from.
        change: String,
        /// Target domain spec override (defaults to domain in spec.md frontmatter).
        #[arg(long)]
        domain: Option<String>,
        /// Dry run preview without mutating files.
        #[arg(long)]
        dry_run: bool,
    },
}
```

### 2.2 Archive Command Extension

Extend `ArchiveArgs` in `src/commands/workflow.rs`:

```rust
#[derive(Args, Debug, Clone)]
pub struct ArchiveArgs {
    // ... existing fields ...
    /// Automatically promote delta requirements into target domain spec during archival.
    #[arg(long)]
    pub promote: bool,

    /// Target domain override for specification promotion.
    #[arg(long)]
    pub domain: Option<String>,
}
```

## 3. Data Structures & Schema Definitions

### 3.1 Domain Specification YAML Frontmatter

```yaml
---
title: "Harness Architecture & Multi-Agent Integration"
domain: harnesses
version: 1.0.0
last_updated: "2026-09-15"
dependencies: [state, project-adoption]
---
```

### 3.2 Domain Specification In-Memory Models (`src/commands/spec.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSpecMetadata {
    pub title: String,
    pub domain: String,
    pub version: String,
    pub last_updated: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSpecSummary {
    pub domain: String,
    pub title: String,
    pub version: String,
    pub last_updated: String,
    pub requirement_count: usize,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecValidationFinding {
    pub file: PathBuf,
    pub domain: Option<String>,
    pub severity: SpecFindingSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecFindingSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecValidationReport {
    pub valid: bool,
    pub specs_count: usize,
    pub findings: Vec<SpecValidationFinding>,
}
```

## 4. Promotion & Merging Algorithm

When promoting from `openspec/changes/<change>/spec.md` into `openspec/specs/<domain>.md`:
1. **Target Domain Resolution:**
   - Command flag `--domain <name>` has highest precedence.
   - Fallback: parse YAML frontmatter `domain: <name>` from `spec.md`.
   - If neither is provided, fail with `CeError::Usage("target domain not specified and not found in spec.md frontmatter")`.
2. **Requirement Extraction:**
   - Parse `spec.md` for requirement blocks matching `### R<N>. <Title>` or `### <Title>`.
   - Extract the requirement prose (Given/When/Then, MUST/SHALL/SHOULD).
3. **Domain Spec Mutation:**
   - Locate `openspec/specs/<domain>.md`. If it does not exist, create a new domain spec template.
   - Locate the `## Capabilities & Requirements` section.
   - Append or update the requirement items, tagging each with provenance: `<!-- source: change:<change_name> date:<YYYY-MM-DD> -->`.
   - Update frontmatter `last_updated` date to current UTC date.
4. **Atomic Write Guarantee:**
   - Write updated domain spec using `crate::state::write_atomic`.

## 5. Doctor Health Check Probe (`probe_specs_health`)

Add a dedicated probe in `src/commands/doctor.rs` / `src/commands/workflow.rs`:
- Validates that `openspec/specs/` exists and contains valid markdown files with compliant frontmatter.
- Audits active changes in `openspec/changes/` (excluding `archive/`): checks whether each `spec.md` specifies a valid known `domain: <domain>`.
- Emits non-blocking `doctor-warn:` advisories:
  - `doctor-warn: active change '<feature>' lacks target domain mapping in spec.md`
  - `doctor-warn: domain spec '<domain>.md' is missing required section '## Capabilities & Requirements'`
