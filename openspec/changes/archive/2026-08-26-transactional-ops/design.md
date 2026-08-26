# Design

## Journal schema (`<config_dir>/install-journal.json`, atomic writes)

```json
{
  "command": "install",
  "started_at": "<rfc3339>",
  "ops": [
    {
      "path": "/abs/path",
      "applied": true,
      "prior": { "bytes_base64": "..." }   // or null = file did not exist
    }
  ]
}
```

- `prior` is captured **before** the mutation and persisted with the record
  (journal rewritten atomically on every step — O(n²) bytes worst case is fine
  for text-sized managed assets).
- Unknown/corrupt journal → stderr warning, treated as absent.

## API (`src/state/journal.rs`)

```rust
pub enum TrackedWrite { Applied, InjectedFailure } // internal
pub struct Journal { /* path, data */ }

impl Journal {
    /// Detects + rolls back a stale journal (reverse order), warning on
    /// stderr; then starts a fresh journal for `command`.
    pub fn begin(config_dir: &Path, command: &str) -> Result<Self, CeError>;
    /// Captures prior content, marks applied=true, persists journal, THEN
    /// performs write_atomic(bytes) unless fault-injected; returns the
    /// inner result so callers keep their error handling unchanged.
    pub fn tracked_write(&mut self, path: &Path, bytes: &[u8]) -> Result<(), CeError>;
    /// Removes the journal after the command's final state save succeeded.
    pub fn complete(self) -> Result<(), CeError>;
}
```

Fault injection reads `CE_AI_FAIL_AFTER_WRITES` once at `begin`.

## Integration points

- `install.rs`: loader write, managed-file loop, custom plugin/skill copies
  (file-level), rules-file block write, manifest write; **state.save stays
  last** and is followed by `journal.complete()`.
- `sync.rs` (`sync_with`): managed-tree action writes; same completion rule.
- `uninstall`/`init-prj` untouched (documented out-of-scope).
- `doctor.rs`: finding `install-journal: incomplete operation from '<command>'
  detected — run any install/sync to auto-recover` when the file exists.

## Recovery semantics

Reverse iteration over `applied == true` ops: prior bytes → rewrite atomically;
prior absent → remove file. Unapplied records are ignored. After rollback the
journal file is removed and recovery reported on stderr before the new
operation begins fresh.
