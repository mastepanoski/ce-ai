---
title: "Append-Only JSONL Operation Journal for Linear-Time Mutation Arming"
category: "architecture"
date: "2026-09-07"
tags:
  - journal
  - state
  - transactions
  - performance
  - append-only
  - jsonl
  - disaster-recovery
components:
  - state::journal
  - commands::install
  - commands::sync
  - commands::adopt
  - commands::doctor
applies_when: "Investigating performance bottlenecks during install/sync, understanding transactional rollback and recovery mechanics, or reviewing operation journal file formats"
problem_type: architectural_refactor
---

# Append-Only JSONL Operation Journal for Linear-Time Mutation Arming

## Context & Problem

In `ce-ai`, filesystem mutations executed during `install`, `sync`, and `adopt` are tracked by a crash-recovery operation journal (`src/state/journal.rs`, Issue #166). Every mutation records its target path and prior content *before* modifying the filesystem, ensuring deterministic recovery on crash or abort.

Prior to v1.44.2, `Journal::arm` persisted state on every mutation by re-serializing the entire accumulated list of `RecordedOp` entries (including full `prior: Option<Vec<u8>>` snapshots) via `serde_json::to_vec_pretty` and rewriting the journal file via `write_atomic`:

```rust
// Pre-1.44.2 O(N^2) arm implementation
pub fn arm(&mut self, path: &Path) -> Result<(), CeError> {
    self.writes_seen += 1;
    let prior = std::fs::read(path).ok();
    self.data.ops.push(RecordedOp { path: path.to_path_buf(), applied: true, prior });
    self.persist()?; // <-- re-serialized ALL ops and executed write_atomic on every single mutation!
    ...
}
```

Because `Journal` spans the entire multi-harness installation loop across ~780+ managed files, re-serializing the growing array of records on every mutation produced $O(N^2)$ cumulative CPU and disk write complexity. Profiling in Issue #312 confirmed that >95% of wall-clock execution time was spent inside `Journal::arm -> persist -> serde_json::to_vec_pretty`, preventing installations from completing within 10 minutes in debug builds.

## Solution Architecture

Release v1.44.2 refactors the operation journal to an append-only JSON Lines (`JSONL`) format with an open file handle, reducing the cost of each `arm()` invocation from $O(k)$ to $O(1)$, resulting in strictly linear $O(N)$ cumulative complexity.

### 1. On-Disk JSONL Schema

The on-disk journal layout at `install-journal.json` is restructured into two parts:
- **Line 1 (Header)**: Single-line JSON containing metadata:
  ```json
  {"command":"install","started_at":"2026-09-07T15:30:00Z"}
  ```
- **Lines 2..N (Operations)**: One self-contained `RecordedOp` JSON line per mutation:
  ```json
  {"path":"/path/to/file.txt","applied":true,"prior":[117,115,101,114]}
  ```

### 2. Persistent Open File Handle with Immediate `sync_all`

`Journal::begin` initializes the header atomically using `write_atomic`, and opens a persistent `std::fs::File` in append mode (`OpenOptions::new().append(true)`).

On each `arm()` call:
1. Prior content is read into memory.
2. A single `RecordedOp` JSON line is serialized and written directly to the open handle.
3. `self.file.sync_all()?` is called immediately, maintaining the exact same physical durability boundary as `write_atomic`.
4. No prior operations are read, re-serialized, or rewritten.

### 3. Crash Resilience & Partial Line Recovery

If a command crashes mid-write while appending an operation record:
- `Journal::begin` parses the file line-by-line.
- If a trailing line is incomplete or invalid JSON, it is safely ignored with a diagnostic warning.
- All preceding complete operations are recovered and rolled back in reverse order.
- If the header line itself is malformed, the journal is treated as corrupt (preserving the existing `corrupt_journal_is_treated_as_absent` contract).

### 4. Backward Compatibility with Legacy Single-Blob Format

To support seamless upgrades if a user has an uncompleted journal from v1.44.1 or earlier, `Journal::begin` and `recorded_command` fall back to deserializing legacy single-blob JSON structures if line 1 does not parse as `JournalHeader`.

### 5. Windows Safety on Removal

On Windows, unlinking a file while a file handle is open causes an OS sharing violation error. `Journal::complete` explicitly drops `self.file` before calling `std::fs::remove_file(&self.path)`.

## Quantitative Performance Verification

A dedicated complexity test in `src/state/tests/journal.rs` (`journal_arm_complexity_is_strictly_linear`) validates the performance improvement across 500 armed operations with 512-byte payloads:
- **Individual write size**: Bound to $O(1)$ per arm ($\Delta_k \le 2.5\text{ KB}$), invariant to the number of armed operations.
- **Total journal size**: Exactly linear $O(N)$ ($\approx 912\text{ KB}$ for 500 files), compared to $>200\text{ MB}$ of redundant write traffic under the $O(N^2)$ algorithm (a >200x reduction in disk write traffic).
- **Durability**: 100% of mutations preserve fsync durability before caller modifications.
