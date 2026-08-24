# Specification: Multi-Harness Reconciliation, DeepSeek De-scope & Release Fallback Hardening

## Requirements

### R1: Qualified README & Spec Documentation
WHEN viewing `README.md` or OpenSpec specifications,
THEN `ce-ai` SHALL document the 10 real native adapters (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `fx`) and their native formats, and `openspec/changes/multi_harness_support/spec.md` SHALL be reconciled with the native adapter architecture.

### R2: DeepSeek De-scope Usage Error & Detection Exclusion
WHEN a user invokes a CLI subcommand targeting `deepseek` (`install`, `uninstall`, `sync`, `init-prj`, `deinit-prj`, `tools`),
THEN `ce-ai` SHALL return exit code 2 (usage error) explaining the developer-preview status of `dsh` and guiding users to supported native harnesses, AND automatic harness detection SHALL filter out `deepseek`.

### R3: GitHub API 403 & Network Error Fallback
WHEN `resolve_latest_release` encounters network send failures or non-success HTTP status codes (403, 429),
THEN it SHALL log an informative notice to stderr and return `Ok(None)`, triggering SF-2 main branch tarball fallback without hard-failing.

### R4: Audit Configuration Coverage Output
WHEN `ce-ai audit` renders its final score summary or evaluates `--fail-under`,
THEN it SHALL display `configuration coverage` instead of `audit score`.
