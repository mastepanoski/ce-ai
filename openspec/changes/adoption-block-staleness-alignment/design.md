# OpenSpec Technical Design: Data Schemas & API Helpers

- **Change:** `adoption-block-staleness-alignment`
- **Issue:** #149

---

## 📐 Data Schemas & Helper Functions

In `src/commands/init_prj.rs`:
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

pub fn check_adoption_block_status(agents_file: &std::path::Path, tier: AdoptionTier) -> AdoptionBlockStatus
```

Usage in `status.rs`:
```rust
match check_adoption_block_status(&agents_file, p.tier) {
    AdoptionBlockStatus::Ok => "OK".to_string(),
    AdoptionBlockStatus::StaleVersion { version } => format!(
        "STALE BLOCK v={} — re-run ce-ai init-prj --tier {} to upgrade",
        version, p.tier.as_str()
    ),
    AdoptionBlockStatus::DriftDetected => "DRIFT DETECTED".to_string(),
    AdoptionBlockStatus::MalformedBlock => "MALFORMED BLOCK".to_string(),
    AdoptionBlockStatus::BlockMissing => "BLOCK MISSING".to_string(),
    AdoptionBlockStatus::FileMissing => "MISSING".to_string(),
    AdoptionBlockStatus::ReadError => "READ ERROR".to_string(),
}
```
