# Spec Delta: `custom-harness-r4`

## MODIFIED Requirement 4 (multi_harness_support): Custom Harness Fallback Mode

The custom harness mode SHALL install Compound Engineering assets into
user-configured directories with an explicit, fail-fast configuration
contract.

### R4.1 Configuration Resolution

- **WHEN** the user runs any command targeting `--harness custom`,
  **THEN** the CLI MUST resolve the target configuration in this precedence
  order: explicit CLI flags (`--plugins-dir`, `--skills-dir`, `--rules-file`)
  first, the persisted state-entry snapshot second (uninstall/sync), the
  config file `~/.ce-ai/custom_harness.json` third.
- **WHEN** no source provides `plugins_dir` and `skills_dir`,
  **THEN** the CLI MUST fail with `CeError::Usage` (exit code 2) before any
  filesystem mutation, printing guidance for both configuration mechanisms.
- **WHEN** a configured path starts with `~/` or is relative,
  **THEN** the CLI MUST expand it against `$HOME` / the process CWD
  respectively before use.

### R4.2 Install

- **WHEN** the user runs `ce-ai install --harness custom --plugins-dir P
  --skills-dir S [--rules-file R]`, **THEN** the CLI MUST create `P` and `S`
  if absent, copy every managed plugin file `plugins/<rest>` to
  `P/<rest>` and every managed skill `skills/<rest>` to `S/<rest>`, and
  record every copied file with its SHA256 in
  `P/compound-engineering/install-manifest.json`.
- **WHEN** `--rules-file R` is provided, **THEN** the CLI MUST ensure the
  file contains exactly one current-version managed CE block
  (`<!-- ce-ai:block begin v=2 ... -->` ... `<!-- ce-ai:block end -->`),
  preserving all non-managed content verbatim.
- **WHEN** the install targets `--harness custom`, **THEN** the CLI MUST NOT
  write any OpenCode-format config (`plugin` array / `skills.paths`) to any
  location, and MUST NOT create `~/.config/custom/` or `~/.custom/`.
- **WHEN** the install completes, **THEN** the state entry for `custom`
  MUST embed the resolved configuration under a `custom` key.
- **WHEN** `--dry-run` is set, **THEN** the CLI MUST print the plan
  (create/copy actions with resolved absolute paths) and perform zero writes,
  including failing fast with exit code 2 when configuration is unresolvable.

### R4.3 Uninstall

- **WHEN** the user runs `ce-ai uninstall --harness custom`, **THEN** the
  CLI MUST remove exactly the files recorded in the install manifest, the
  manifest itself, and prune CE-owned directories it emptied.
- **WHEN** the rules file contains a managed CE block, **THEN** the CLI MUST
  remove that block and preserve every other byte of the file.
- **WHEN** the directories contain files not recorded in the manifest,
  **THEN** the CLI MUST leave them untouched.
- **WHEN** neither the state snapshot nor flags nor the config file resolve,
  **THEN** the CLI MUST fail with `CeError::Usage` (exit code 2).

### R4.4 Sync

- **WHEN** `ce-ai sync` processes a `custom` state entry, **THEN** the CLI
  MUST re-copy the desired managed trees into the recorded directories and
  refresh the manifest.
- **WHEN** post-sync hashing finds drift on a custom surface,
  **THEN** the CLI MUST report `FAILED` for that surface in the verification
  matrix and exit with code 6.
- **WHEN** sync rebuilds `state.installed_harnesses`, **THEN** it MUST
  preserve each entry's `custom` snapshot.

### R4.5 Single Path Contract

- **WHEN** any code resolves a default custom-mode path,
  **THEN** it MUST yield `~/.ce-ai/custom_harness.json`; no other hardcoded
  custom path MAY exist in the codebase.
- **WHEN** the repository builds, **THEN** the dead `generic_json` module and
  all `#[allow(dead_code)]` markers on `CustomAdapter` MUST be gone.
