# OpenSpec Requirements & Acceptance Criteria

**Change Identifier:** `multi_harness_support`  

---

## Requirements

### Requirement 1: Multi-Harness Flag Parsing & Resolution
- **WHEN** the user invokes `ce-ai install --harness <name>`, **THEN** the CLI MUST validate `<name>` against `HarnessKind`. If invalid, return exit code `2` (Usage Error) with a list of supported harnesses.
- **WHEN** the user invokes `ce-ai install --all`, **THEN** the CLI MUST probe for installed harness configuration directories on the host system and execute installation for each detected harness.

### Requirement 2: JSON Configuration Merging
- **WHEN** installing or updating a JSON-based harness (`opencode`, `claude`, `pi`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`), **THEN** `ce-ai` MUST parse the target JSON file, update managed fields (`plugins`, `skills`), preserve unmanaged keys, and write back using `write_atomic`.

### Requirement 3: Markdown Instruction Block Ingestion
- **WHEN** installing or updating a Markdown-based harness (`cursor`, `copilot`), **THEN** `ce-ai` MUST append or update a managed block demarcated by `<!-- CE-AI MANAGED BLOCK BEGIN -->` and `<!-- CE-AI MANAGED BLOCK END -->`, leaving user instructions untouched.

### Requirement 4: Custom Harness Fallback Mode
- **WHEN** the user runs `ce-ai install --harness custom`, **THEN** the CLI MUST check for `--plugins-dir` and `--skills-dir` flags. If missing in interactive mode, prompt the user via `inquire`.

### Requirement 5: Model Role Syncing Across Harnesses
- **WHEN** `ce-ai models set <slot> = <model>` is run, **THEN** `ce-ai` MUST update central `state.json` and sync the model assignment into all installed harness configuration files.

### Requirement 6: Manifest Tracking & Backup Restoration
- **WHEN** any harness is installed or modified, **THEN** SHA256 file hashes MUST be saved in `manifest.json` under `~/.ce-ai/manifest.json`.
- **WHEN** `ce-ai uninstall --harness <name>` is executed, **THEN** `ce-ai` MUST restore the timestamped pre-install backup and remove managed files.
