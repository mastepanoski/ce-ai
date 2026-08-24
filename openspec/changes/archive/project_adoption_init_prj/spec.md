# OpenSpec Functional Specification: Project Adoption Engine

## Requirements & Acceptance Criteria

### Requirement 1: Project Initialization & Block Injection (`ce-ai init-prj`)
**WHEN** a user or agent executes `ce-ai init-prj [PATH] --tier <full|minimal|orchestrator>`  
**THEN** `ce-ai` MUST:
1. Resolve the absolute target path (defaulting to current working directory or Git repository root).
2. Read `AGENTS.md` in the target directory (creating the file if missing and recording `created_file: true`).
3. Inject the marker-delimited managed block:
   ```markdown
   <!-- ce-ai:block begin v=1 tier=full -->
   ...
   <!-- ce-ai:block end -->
   ```
4. Preserve all pre-existing user content outside marker boundaries.
5. Create derived reference stubs (e.g. `CLAUDE.md` containing `@AGENTS.md`) if missing.
6. Record the project adoption record (`path`, `tier`, `block_version`, `block_sha256`, `created_file`) in `~/.ce-ai/state.json` atomically.

### Requirement 2: Symmetric Clean De-initialization (`ce-ai deinit-prj`)
**WHEN** a user or agent executes `ce-ai deinit-prj [PATH]`  
**THEN** `ce-ai` MUST:
1. Extract and remove the `<!-- ce-ai:block begin -->` ... `<!-- ce-ai:block end -->` segment from `AGENTS.md`.
2. If `AGENTS.md` was created by `ce-ai init-prj` and contains no user content outside markers, delete `AGENTS.md` and derived stubs.
3. If `AGENTS.md` contains pre-existing user content, save the cleaned file atomically.
4. Remove the project entry from `state.json` atomically.
5. Guarantee that `bytes(after deinit-prj) == bytes(before init-prj)`.

### Requirement 3: Idempotency & Conflict Resolution
**WHEN** `ce-ai init-prj` is run on an already adopted repository:
- **IF** the existing managed block's SHA matches the template, `ce-ai` MUST perform no file writes and return exit code 0.
- **IF** the existing managed block was manually edited (SHA mismatch), `ce-ai` MUST refuse to overwrite and demand `--force`.

### Requirement 4: Health Audit & Diagnostics (`status` & `doctor`)
**WHEN** `ce-ai status` or `ce-ai doctor` is executed:
**THEN** `ce-ai` MUST audit all adopted projects listed in `state.json`, checking for:
- Missing `AGENTS.md` files.
- Manual edit drift (SHA mismatch inside managed markers).
- Stale block versions eligible for upgrade via `ce-ai sync`.

### Requirement 5: Error Exit Code Compliance
- Invalid flags or path arguments map to `CeError::Usage` (Exit Code `2`).
- File write or permission errors map to `CeError::Io` (Exit Code `4`).
- Registry state corruption maps to `CeError::State` (Exit Code `3`).
