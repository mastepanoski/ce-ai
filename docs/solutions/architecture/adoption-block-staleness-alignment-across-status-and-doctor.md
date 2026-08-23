---
module: src/commands/init_prj.rs
tags: [adoption-block, status, doctor, refactoring, single-source-of-truth]
problem_type: architecture
---

# Adoption Block Staleness Alignment Across Status and Doctor Diagnostics

## Problem
`ce-ai doctor` distinguished between stale adoption blocks (`v < BLOCK_VERSION`) with an actionable upgrade hint (`re-run ce-ai init-prj --tier <tier> to upgrade`) and generic SHA drift. However, `ce-ai status` reported only `DRIFT DETECTED` for both cases because block version parsing was duplicated and inline in `doctor.rs`.

## Solution
Extracted `check_adoption_block_status` and `AdoptionBlockStatus` enum into `src/commands/init_prj.rs` as the single source of truth for adoption block status classification:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdoptionBlockStatus {
    Ok,
    StaleVersion { version: u32 },
    DriftDetected,
    MalformedBlock,
    BlockMissing,
    FileMissing,
    ReadError,
}

pub fn check_adoption_block_status(agents_file: &Path, tier: AdoptionTier) -> AdoptionBlockStatus
```

Both `src/commands/doctor.rs` and `src/commands/status.rs` now consume this helper, guaranteeing 100% diagnostic alignment and actionable upgrade hints across commands.
