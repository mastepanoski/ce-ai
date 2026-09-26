# Design: Monotonic Accretion Guard & Anti-Clobber Protection for CONCEPTS.md

## System Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                   Layer 1: Agent Contract                        │
│   • ce-compound Phase 2.4: Mandate Read before edit              │
│   • Prefer Edit (surgical replace) over Write (blind overwrite)  │
│   • Empirical reporting: "N entries (was M, +X added, -Y removed)│
└─────────────────────────────────┬────────────────────────────────┘
                                  │
┌─────────────────────────────────▼────────────────────────────────┐
│             Layer 2: Portable Python Validator                   │
│   scripts/validate-concepts.py                                   │
│   • Pure stdlib (Python 3, sys, os, re, subprocess)              │
│   • Parses markdown headings: `## ` / `### ` & `- **Term**:`     │
│   • Compares working tree against `git show HEAD:CONCEPTS.md`    │
│   • Exit codes: 0 = Monotonic/Valid, 1 = Clobber, 2 = Usage      │
└─────────────────────────────────┬────────────────────────────────┘
                                  │
┌─────────────────────────────────▼────────────────────────────────┐
│           Layer 3: Native Rust Diagnostic & CLI Gates            │
│   src/commands/workflow.rs                                       │
│   • probe_concepts_drift(repo_root, git_available)               │
│   • ConceptsDriftFinding { deleted_entries, ... }                │
│   • Added to DocDebtReport struct                                │
│                                                                  │
│   src/commands/doc.rs                                            │
│   • run_doc_lint: checks concepts drift alongside solutions      │
│   • Strict mode: exits with CeError::Verification on shrinkage   │
│                                                                  │
│   src/commands/doctor.rs                                         │
│   • doctor-warn on destructive CONCEPTS.md shrinkage             │
└──────────────────────────────────────────────────────────────────┘
```

## Data Structures & Schemas

### Rust Structures (`src/commands/workflow.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConceptsDriftFinding {
    pub path: String,
    pub head_entries_count: usize,
    pub current_entries_count: usize,
    pub deleted_entries: Vec<String>,
    pub added_entries: Vec<String>,
}

pub struct DocDebtReport {
    pub git_available: bool,
    pub openspec_desync: ProbeStatus<Vec<OpenSpecDesyncFinding>>,
    pub stale_pending: ProbeStatus<Vec<StalePendingOpenSpecFinding>>,
    pub solution_drift: ProbeStatus<Vec<SolutionDriftFinding>>,
    pub archive_compaction: ProbeStatus<ArchiveCompactionFinding>,
    pub concepts_drift: ProbeStatus<ConceptsDriftFinding>,
}
```

### Parsing Logic
Concept headings are identified by either:
1. ATX Headings: `^#{2,4}\s+(.+)$`
2. Bold Definition Lists: `^[-*]\s+\*\*([^*]+)\*\*:`

Preamble headers (such as `# Concepts`, `## Introduction`, `## Table of Contents`) are excluded from vocabulary term count.
Scrub comments matching `<!--\s*scrub:\s*([^*]+)\s*-->` or `<!--\s*retired:\s*([^*]+)\s*-->` indicate intentional removal and do not trigger clobber errors.

## CLI Contracts

### 1. `scripts/validate-concepts.py`
```
Usage: python3 scripts/validate-concepts.py [CONCEPTS.md path] [--allow-shrink] [--json]

Exit codes:
  0: Monotonically accreted (no unscrubbed entries lost)
  1: Destructive clobber detected (entries missing from HEAD)
  2: Usage error / file not found
```

### 2. `ce-ai doc lint`
```
$ ce-ai doc lint
doc lint: all solutions have valid frontmatter and resolvable source paths
doc lint: CONCEPTS.md accreted monotonically (24 entries, +1 added)

$ ce-ai doc lint --strict
doc lint: 1 finding(s) detected in CONCEPTS.md:
  [concepts-clobber] CONCEPTS.md: destructive shrinkage detected: 22 heading(s) deleted: (BriefSystem, EmailQueue, ...)
Error: solution or concepts drift findings detected in strict mode
Exit code: 6 (Verification)
```

### 3. `ce-ai doctor`
```
doctor-warn: CONCEPTS.md suffered destructive shrinkage (-22 entries from HEAD: BriefSystem, EmailQueue, ...) — run 'git checkout CONCEPTS.md' or re-compound with surgical Edit
```
