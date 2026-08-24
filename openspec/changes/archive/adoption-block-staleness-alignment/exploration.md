# OpenSpec Exploration: Technical Investigation

- **Change:** `adoption-block-staleness-alignment`
- **Issue:** #149

---

## 🔍 Investigation & Refactoring Options

### Extracted Helper API
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

pub fn check_adoption_block_status(agents_file_path: &Path, tier: AdoptionTier) -> AdoptionBlockStatus
```

Both `doctor.rs` and `status.rs` will invoke `check_adoption_block_status`, eliminating duplicated string parsing and guaranteeing 100% alignment.
