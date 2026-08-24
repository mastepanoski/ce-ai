# Specification: Global `--dry-run` Purity & Zero-Mutation Contract

## Requirements

### R1: Zero Disk Mutation Under `--dry-run`
WHEN any CLI subcommand is executed with `--dry-run`,
THEN `ce-ai` SHALL NOT write, modify, delete, or create any files in the configuration directory (`ctx.config_dir`), home directory (`home_dir`), or project workspace (`workspace_dir`).

### R2: Transient Remote Tarball Download Under `--dry-run`
WHEN `ce-ai install` or `ce-ai upgrade` resolves a remote source under `--dry-run`,
THEN `ce-ai` SHALL extract the source tarball to a transient temporary directory and discard it after plan generation, leaving `cache/` and `state.json` untouched.

### R3: Triple-Directory Snapshot Verification
WHEN running CLI integration tests for `--dry-run` commands,
THEN `tests/cli.rs` SHALL assert byte-for-byte state equality before and after dry-run execution across `config_dir`, `home_dir`, and `workspace_dir`.
