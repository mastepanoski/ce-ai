# Spec: Sync Verification Matrix UX Clarity

## ADDED Requirements

### Requirement SVX-1: Unambiguous Registered State

The sync verification matrix SHALL label every surface where ce-ai manages no
files as `registered` with a reason stating why nothing was hash-verified,
and SHALL NOT use the wording `synced — verification not performed` or the
word `unverified` anywhere in the matrix output.

#### Scenario: Plugin-only release leaves native harnesses registered

- **WHEN** `ce-ai sync` runs against an install whose manifest contains no
  `skills/` entries
- **THEN** each native harness line reads
  `○ <harness>: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)`
- **AND** the cursor line reads
  `○ cursor: registered — config registration only — no managed assets to hash-verify`
- **AND** the reconciliation line reads
  `reconciliation status: <V> verified, <U> registered (nothing to verify), <F> failed`

### Requirement SVX-2: Verified and Failed Wordings Are Explicit

The matrix SHALL render verified surfaces as
`✓ <harness>: verified — <matched>/<total> managed files match SHA256` and
drifted surfaces as `✗ <harness>: FAILED — <count> file(s) drifted` followed
by one indented line per drifted path.

#### Scenario: Verified and drifted surfaces keep their contract

- **WHEN** a surface's managed files all match their manifest SHA256
- **THEN** its line matches the verified wording above
- **WHEN** a surface has one mismatched file `plugins/x.js`
- **THEN** its header matches the FAILED wording with count 1
- **AND** the next line is `      plugins/x.js`

### Requirement SVX-3: Adoption Guidance Note

WHEN at least one surface is `registered`, the matrix output SHALL append a
guidance note that states all of the following:

- `registered` means ce-ai manages no files on that surface;
- CE installed via other channels (plugin marketplaces, manual copies) is
  outside ce-ai's verification scope;
- the command to put a harness under ce-ai management
  (`ce-ai install --harness <name>`, or `--harness all`);
- skill files are managed per harness only when the installed source ships a
  managed skills tree.

#### Scenario: Note appears only when needed

- **WHEN** the matrix contains at least one `registered` surface
- **THEN** the note block follows the reconciliation line
- **WHEN** every surface is verified or failed
- **THEN** no note block is printed

### Requirement SVX-4: Documented Management Model

The user guide (`docs/user-guide/sync-and-upgrade-mechanisms.md`) SHALL
document, for each supported harness, what ce-ai writes during install/sync,
the command that puts the harness under ce-ai management, and the scope
boundary for CE installed via other channels; its Step 6 sample output SHALL
match the matrix contract of SVX-1..SVX-3.

#### Scenario: A newbie can self-serve from the docs

- **WHEN** a reader follows the Step 6 section of the user guide
- **THEN** they find the three verification states explained and a per-harness
  management table including the adoption command
