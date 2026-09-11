# Proposal: Append-Only JSONL Operation Journal for Linear-Time Mutation Arming

## Problem Statement

In `src/state/journal.rs`, `Journal::arm` re-serializes the entire operation journal history on every filesystem mutation:
```rust
pub fn arm(&mut self, path: &Path) -> Result<(), CeError> {
    self.writes_seen += 1;
    let prior = std::fs::read(path).ok();
    self.data.ops.push(RecordedOp { path: path.to_path_buf(), applied: true, prior });
    self.persist()?; // re-serializes entire self.data via serde_json::to_vec_pretty & write_atomic
    ...
}
```

Because `Journal` spans the entire multi-harness installation loop (`for harness_kind in &target_harnesses` in `src/commands/install.rs`) across ~780+ managed files, re-serializing the growing array of `RecordedOp` records (each containing full `prior: Option<Vec<u8>>` snapshots) and executing `write_atomic` on every single mutation produces $O(N^2)$ cumulative CPU and disk write complexity.

Issue #312 confirmed that profiling debug installations showed >95% of wall-clock execution time concentrated in `Journal::arm -> persist -> serde_json::to_vec_pretty`, preventing installations from completing within 10 minutes.

## Scope Boundaries

### In Scope
- Transition `install-journal.json` on-disk layout from a single JSON blob (`{command, started_at, ops: [...]}`) to an append-only JSON Lines (`JSONL`) format:
  - Line 1: Header object (`{"command": "...", "started_at": "..."}`).
  - Lines 2..N: Individual `RecordedOp` JSON records (`{"path": "...", "applied": true, "prior": [...]}`).
- Maintain an open `std::fs::File` handle in `Journal` opened with `append(true)`.
- `Journal::arm` appends only the newly armed `RecordedOp` JSON record followed by `b'\n'` and invokes `sync_all()` to preserve fsync durability before filesystem mutations occur.
- `Journal::begin` parses the JSONL journal on startup, rolling back recorded operations in reverse order.
- Tolerant parsing: If a process terminates mid-write on the final line, the partial trailing line is safely ignored while all preceding complete records are recovered and rolled back.
- Legacy format compatibility: If the file was written by an earlier version of `ce-ai` (single JSON blob), `begin` and `recorded_command` fall back to parsing the legacy structure.
- Windows safety: Explicitly drop the open file handle before `std::fs::remove_file` in `Journal::complete` to prevent file sharing violations on Windows.
- Quantitative complexity test: Assert that $N$ mutations write $O(N)$ linear bytes without quadratic amplification.

### Out of Scope
- Changing the `prior: Option<Vec<u8>>` representation (e.g. hashing, compression, or external blob storage).
- Modifying calling contracts or call sites outside `src/state/journal.rs` (public API remains strictly identical).
- Modifying `custom.rs` or multi-harness loop structure.

## Risk Evaluation
- **Durability Guarantee**: Calling `self.file.sync_all()?` after writing the single record preserves the exact same POSIX/Windows durability guarantee previously provided by `write_atomic`.
- **Fault-Injection Integrity**: `CE_AI_FAIL_AFTER_WRITES` triggers after the op has been written and synced to disk, preserving deterministic crash-recovery testing.
- **Windows File Locking**: Closing the file handle before unlinking prevents `os error 32` (Sharing Violation).

## Success Criteria
1. Public API remains 100% backward-compatible (`Journal::begin`, `journal.arm`, `journal.complete`, `recorded_command`, `journal_path`).
2. `Journal::arm` complexity is reduced from $O(k)$ to $O(1)$, resulting in $O(N)$ total cost for $N$ mutations.
3. Partial/incomplete trailing lines in aborted journals are safely skipped without discarding earlier valid operations.
4. `cargo test --all-features`, `cargo clippy`, and `make e2e` pass 100% cleanly across all platforms.
