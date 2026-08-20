# Delta Spec: ce-ai — CE Plugin Manager CLI

## ADDED Requirements

### ce-source-fetching

| ID | Requirement |
|---|---|
| SF-1 | MUST resolve latest CE GitHub release. |
| SF-2 | SHOULD fall back to `main` tarball if release metadata is missing. |
| SF-3 | MUST cache tarballs under `~/.ce-ai/cache` and record SHA256 digest in `state.json`. |
| SF-4 | MUST support `--source <local-path>` to use a local CE tree. |

#### Scenario: Fetch latest release
- GIVEN network access
- WHEN `ce-ai install` runs
- THEN the latest release is cached and its digest recorded.

#### Scenario: Use local source
- GIVEN a local CE clone
- WHEN `ce-ai install --source /path/to/ce` runs
- THEN no network fetch occurs.

### opencode-install

| ID | Requirement |
|---|---|
| OI-1 | MUST back up `opencode.json` before mutation. |
| OI-2 | MUST merge the CE plugin entry without clobbering user config. |
| OI-3 | MUST copy the CE plugin loader into the OpenCode plugins dir. |
| OI-4 | MUST register the CE skills path in `opencode.json`. |
| OI-5 | MUST write `install-manifest.json` listing managed files. |

#### Scenario: Fresh install
- GIVEN no CE install
- WHEN `ce-ai install --harness opencode` runs
- THEN backup, entry, loader, skills path, and manifest are created.

#### Scenario: Re-install is idempotent
- GIVEN CE installed
- WHEN `ce-ai install --harness opencode` runs again
- THEN no duplicate entries appear.

### sync-upgrade

| ID | Requirement |
|---|---|
| SU-1 | MUST reconcile desired manifest against installed files. |
| SU-2 | MUST repair missing or modified managed files. |
| SU-3 | MUST detect drift and report it. |
| SU-4 | MUST support `--dry-run`. |
| SU-5 | `upgrade` MUST fetch latest release then sync. |

#### Scenario: Sync repairs drift
- GIVEN deleted managed file
- WHEN `ce-ai sync` runs
- THEN file is restored and manifest updated.

#### Scenario: Dry-run writes nothing
- GIVEN `--dry-run`
- WHEN any mutating command completes
- THEN no writes occur and changes are listed.

### models-management

| ID | Requirement |
|---|---|
| MM-1 | MUST persist model assignments in `state.json`. |
| MM-2 | MUST apply assignments to `opencode.json`. |
| MM-3 | MUST support named profiles. |
| MM-4 | Profile saves MUST create append-only snapshots. |

#### Scenario: Set agent model
- GIVEN CE installed
- WHEN `ce-ai models set sdd-explore opencode-go/kimi-k2.6` runs
- THEN both files reflect the assignment.

#### Scenario: Unknown slot warns
- GIVEN unknown slot
- WHEN `ce-ai models set <slot> provider/model` runs
- THEN assignment is persisted; warning MAY be shown.

#### Scenario: Profile round-trip
- GIVEN profile snapshot exists
- WHEN `ce-ai models profile load <name>` runs
- THEN `opencode.json` matches snapshot.

### cli-commands

| ID | Requirement |
|---|---|
| CC-1 | MUST implement all required subcommands. |
| CC-2 | MUST support global flags. |
| CC-3 | `uninstall` MUST restore backup and remove managed files. |

#### Scenario: Status output
- GIVEN CE installed
- WHEN `ce-ai status` runs
- THEN status details are printed.

#### Scenario: Uninstall restores state
- GIVEN CE installed with backup
- WHEN `ce-ai uninstall --harness opencode` runs
- THEN backup restored and managed files removed.

### e2e-docker-gate

| ID | Requirement |
|---|---|
| DG-1 | MUST include a Docker integration test. |
| DG-2 | The test MUST use a fresh HOME. |
| DG-3 | MUST skip if Docker is unavailable. |

#### Scenario: Docker E2E proves flow
- GIVEN Docker available
- WHEN the E2E test runs
- THEN all install artifacts are verified.

#### Scenario: Docker unavailable skips
- GIVEN Docker unreachable
- WHEN the E2E test runs
- THEN it skips and exits zero.
