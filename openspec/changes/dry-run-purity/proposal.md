# Proposal: Global `--dry-run` Purity & Zero-Mutation Contract

## Problem Statement
The global `--dry-run` flag is intended as a 100% side-effect-free preview. However, two commands currently violate this contract:
1. `ce-ai workflow checkpoint` calls `State::save` without inspecting `ctx.dry_run`, writing `state.json` to disk during preview mode.
2. `ce-ai install` and `ce-ai upgrade` with remote sources cache downloaded tarballs in `~/.ce-ai/cache` and update `release_provenance` in `state.json` prior to evaluating the dry-run branch.

## Proposed Changes
1. **Side-Effect-Free Policy**: Thread `ctx.dry_run` through all state mutation, cache storage, and backup operations.
2. **Remote Dry-Run Resolution**: Under `--dry-run`, remote source tarballs are downloaded to isolated temporary directories (`tempfile::TempDir`) and discarded; cache and `state.json` remain untouched.
3. **Snapshot Verification**: Introduce `assert_dry_run_zero_mutation` in CLI integration tests to snapshot `$HOME` and `$CE_AI_CONFIG_DIR` before and after dry-run execution, asserting zero disk mutation.

## Acceptance Criteria
- `--dry-run workflow checkpoint` modifies zero files on disk.
- `--dry-run install --source <remote>` downloads nothing into the persistent cache directory and writes no state.
- `--dry-run upgrade` downloads nothing into the persistent cache directory and writes no state.
- All dry-run integration tests verify zero mutation via snapshot assertion.
