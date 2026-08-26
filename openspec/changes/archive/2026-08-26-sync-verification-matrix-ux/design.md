# Design: Sync Verification Matrix UX Clarity

## Output Contract (after this change)

```text
== [Sync Verification Matrix] ==
version: compound-engineering-v3.23.3
source: github-release
  ✓ opencode: verified — 1/1 managed files match SHA256
  ○ claude: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
  ○ pi: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
  ○ copilot: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
  ○ codex: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
  ○ grok: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
  ○ kimi: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
  ○ agy: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
  ○ fx: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
  ○ cursor: registered — config registration only — no managed assets to hash-verify

reconciliation status: 1 verified, 9 registered (nothing to verify), 0 failed
note: 'registered' = ce-ai wrote harness config only; it manages no files on that
      surface, so there is nothing to hash-verify. CE installed via other channels
      (plugin marketplaces, manual copies) is outside ce-ai's verification scope.
      To put a harness under ce-ai management: ce-ai install --harness <name>
      (or --harness all). Skill files are managed per harness only when the
      installed source ships a managed skills tree.
```

The `note:` block prints only when `unverified > 0`.

## New Static Reason Strings

| Surface | Old | New |
|---|---|---|
| Native skills harnesses, empty desired tree | `no managed skills tree present` | `ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)` |
| Cursor (registration-only arm) | `config registration only — asset hashes not checked` | `config registration only — no managed assets to hash-verify` |
| Custom, empty desired tree | `no managed tree present` | `no managed files — nothing to hash-verify` |
| Custom, missing directory snapshot | `no directory snapshot` | `no directory snapshot — re-run 'ce-ai install --harness custom'` |

## Module Changes (`src/commands/sync.rs`)

```rust
/// One rendered matrix line for a harness surface (pure; unit-tested).
fn matrix_line(harness: &str, status: &CheckStatus) -> String

/// Failed-surface detail lines (one per drifted path), indented under the header.
fn failed_detail_lines(mismatched: &[String], missing: &[String]) -> Vec<String>

/// Reconciliation summary line (pure; unit-tested).
fn reconciliation_line(verified: usize, unverified: usize, failed: usize) -> String

/// Newbie guidance printed when unverified > 0 (pure; unit-tested).
fn guidance_note_lines() -> Vec<String>
```

- `matrix_line` maps:
  - `Verified` → `  ✓ {harness}: verified — {matched}/{total} managed files match SHA256`
  - `Failed` → `  ✗ {harness}: FAILED — {count} file(s) drifted`
  - `NotVerified { reason }` → `  ○ {harness}: registered — {reason}`
- `run()` replaces the inline `match` + counters with calls to these functions
  and prints `guidance_note_lines()` after the reconciliation line when
  `unverified > 0`. The failure aggregation into `CeError::Verification`
  (exit 6) is unchanged.

## Docs Changes (`docs/user-guide/sync-and-upgrade-mechanisms.md`)

- Replace the Step 6 sample output with the new contract above.
- Add a "Verification states" subsection (verified / registered / FAILED).
- Add a "How ce-ai manages each harness" subsection: per-harness table of what
  ce-ai writes (loader+manifest for opencode; MCP companion registration for
  native harnesses; config-only for cursor/custom variants) and the
  `ce-ai install --harness <name>` adoption command; explicit scope boundary
  for CE installed via other channels.

## Version & Release

- `Cargo.toml`: `1.22.2` → `1.22.4` (tag `v1.22.3` already exists on `main`
  from a docs-only release that skipped the in-repo bump; the next free patch
  is 1.22.4).
- `CHANGELOG.md`: `[1.22.4]` entry under `### Changed`.

## Test Plan (TDD)

New unit tests in `src/commands/sync.rs` `mod tests` (written first, RED by
missing symbols, then GREEN):

1. `matrix_line_pins_registered_wording` — `NotVerified` renders
   `○ claude: registered — <new reason>`; pins the exact string.
2. `matrix_line_pins_verified_and_failed_wording` — pins
   `✓ opencode: verified — 1/1 managed files match SHA256` and the FAILED
   header; `failed_detail_lines` indents each drifted path.
3. `reconciliation_line_uses_registered_not_unverified` — pins
   `reconciliation status: 1 verified, 9 registered (nothing to verify), 0 failed`
   and asserts the word `unverified` is absent.
4. `guidance_note_explains_adoption_and_scope` — note mentions
   `install --harness`, "outside ce-ai's verification scope", and the managed
   skills tree condition; empty for zero unverified (call-site guard).

Existing tests untouched (none pin the old strings — verified by grep).
