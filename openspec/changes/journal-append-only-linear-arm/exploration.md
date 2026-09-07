# Exploration: Technical Alternatives for Operation Journal Mutation Arming

## Investigation of Existing Bottleneck

In `src/state/journal.rs`:
```rust
pub fn arm(&mut self, path: &Path) -> Result<(), CeError> {
    self.writes_seen += 1;
    let prior = std::fs::read(path).ok();
    self.data.ops.push(RecordedOp {
        path: path.to_path_buf(),
        applied: true,
        prior,
    });
    self.persist()?;
    ...
}

fn persist(&self) -> Result<(), CeError> {
    write_atomic(
        &self.path,
        &serde_json::to_vec_pretty(&self.data).map_err(CeError::Json)?,
    )
}
```

When executing `ce-ai install --harness all` with ~780+ managed skill and plugin files:
- Mutation 1: serializes 1 record, writes tempfile, syncs, renames.
- Mutation 2: serializes 2 records, writes tempfile, syncs, renames.
...
- Mutation $k$: serializes $k$ records, writes tempfile, syncs, renames.

Total serialized and written bytes across $N$ mutations:
$$\sum_{k=1}^N k \cdot S = \frac{N(N+1)}{2} \cdot S = O(N^2 \cdot S)$$
where $S$ is the average size of a `RecordedOp` (including captured prior bytes). For 800 files with an average op size of 1 KB, this amounts to over 320 MB of redundant JSON serialization and disk writes, accompanied by 800 tempfile creations, 800 `sync_all` calls, and 800 atomic file replacements.

## Evaluated Architectural Options

### Option 1: In-Memory Buffering with Batch Flush
- **Approach**: Accumulate `RecordedOp` records in memory and only flush to disk every $M$ mutations or upon command completion.
- **Tradeoff**: Violates the fundamental transactional invariant of the journal. If a crash or power loss occurs before the batch flush, filesystem mutations made on disk would have no corresponding recovery record, resulting in permanent silent corruption.
- **Verdict**: Rejected. Durability must precede mutation.

### Option 2: Embedded Database (SQLite / Sled / RocksDB)
- **Approach**: Use an embedded transactional key-value store or SQLite database.
- **Tradeoff**: Introduces heavy C dependencies or large Rust crates, inflating compile times, binary size, and cross-platform complexity for an operation that is inherently a simple sequential write-ahead log (WAL).
- **Verdict**: Rejected. Over-engineered.

### Option 3: Append-Only JSON Lines (JSONL) with Persistent Open File Handle
- **Approach**: 
  - Line 1: Header object (`JournalHeader { command, started_at }`) written and synced upon `Journal::begin`.
  - Lines 2..N: Appended incrementally as single-line JSON records (`serde_json::to_vec(&op)` + `b'\n'`) upon each `arm()` invocation.
  - An open `std::fs::File` with `append(true)` is held within `Journal` across the command lifecycle.
  - Each `arm()` writes only its own line and calls `sync_all()` directly on the open file handle.
- **Complexity**:
  - Each `arm()` takes $O(1)$ serialization and write cost.
  - Cumulative disk writes for $N$ mutations scale strictly as $O(N \cdot S)$.
  - Zero tempfile creations or renames during `arm()`.
- **Durability**: Each record is fully flushed and synced to disk before the caller mutates the target path.
- **Verdict**: Selected as the optimal, zero-dependency, minimally-invasive solution.

## Edge Case Analysis

1. **Process Crash During Trailing Line Append**:
   - If the process terminates while writing line $k$, line $k$ may be incomplete/malformed JSON.
   - When `Journal::begin` parses the file, it splits on `\n`. If a line fails `serde_json::from_slice::<RecordedOp>`, it is safely ignored while lines $1..k-1$ are recovered and rolled back.
   - If the header line (line 1) is malformed, the entire journal is treated as corrupt (preserving the existing `corrupt_journal_is_treated_as_absent` behavior).

2. **Windows File Handle Locking on Removal**:
   - On Windows, calling `std::fs::remove_file` while the file handle remains open results in `ERROR_SHARING_VIOLATION`.
   - In `Journal::complete`, dropping `self.file` before invoking `std::fs::remove_file` guarantees clean file deletion.

3. **Legacy Single-Blob Journal Compatibility**:
   - Stale journals created by earlier versions of `ce-ai` will not match the JSONL header format.
   - `Journal::begin` and `recorded_command` include a fallback branch that deserializes the legacy single JSON blob, ensuring seamless upgrades.
