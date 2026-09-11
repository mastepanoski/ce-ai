# Specification: `init-prj` dry-run output correctness

## Requirements

### R1: Dry-run never claims adoption
- WHEN `ce-ai --dry-run init-prj <path>` runs against a project without an up-to-date managed block:
  - THEN stdout MUST contain a line matching `dry-run: would adopt project at`.
  - AND stdout MUST NOT contain `✓ Adopted project`.
  - AND the command MUST exit with code `0`.
- WHEN the global `state.json` is missing or corrupt during a dry-run:
  - THEN the read MUST be best-effort (preview proceeds without registry context) and the command MUST still exit `0` — a write-free preview MUST NOT be aborted by unrelated global-state corruption.

### R2: Dry-run previews RTK hook actions
- WHEN `ce-ai --dry-run init-prj <path>` runs and RTK is not opted out (`--skip-rtk`, `--skip-companions`, `CE_AI_SKIP_RTK`, `CE_AI_SKIP_COMPANIONS`):
  - THEN for each harness `reconcile_rtk_hooks_if_supported` would configure, stdout MUST contain `[dry-run] would configure rtk hook for <harness>`.
  - AND a missing `rtk` binary MUST NOT cause a non-zero exit or a write.
  - AND detection parity MUST hold: because a real run always creates the derived `CLAUDE.md` stub, a fresh project MUST preview the Claude hook even when `.claude` is absent and no harness is registered.
- WHEN RTK is opted out:
  - THEN stdout MAY contain the existing `rtk: hook injection skipped (opted out)` notice, and MUST NOT attempt configuration.

### R3: Dry-run is side-effect free
- WHEN `ce-ai --dry-run init-prj <path>` runs:
  - THEN `AGENTS.md`, `CLAUDE.md`, `.gitignore`, all harness config files under the target, and `state.json` MUST be byte-identical to their pre-run state.
  - AND no new files MUST be created under the target directory.

### R4: Up-to-date dry-run is honest
- WHEN the project already has a managed block identical to the current tier block:
  - THEN stdout MUST report the already-adopted up-to-date status.
  - AND no file MUST be written.

### R5: Real-run behavior is preserved
- WHEN `ce-ai init-prj <path>` runs without `--dry-run` against a fresh project:
  - THEN stdout MUST contain `✓ Adopted project`.
  - AND the managed block, derived stub, `.gitignore` block, registry entry, and harness/RTK reconciliation MUST be applied in the same order and shape as before this change.
  - AND `State::load` MUST remain after the project writes, so a corrupt `state.json` still fails fast on a real run (no writes) without changing the previous failure ordering.
- A real run MUST still create `state.json` and reconcile hooks even though the dry-run path now performs a best-effort read.

### R6: Quiet mode
- WHEN `--quiet` is supplied with `--dry-run`:
  - THEN no ce-ai informational line (including the final preview line and hook previews) MUST be printed.
- Note: in real mode, `--quiet` suppresses ce-ai's own lines but does not intercept stdout of optional external tools (e.g. the `codegraph` binary), which is out of scope for this change.
