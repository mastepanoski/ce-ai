# Proposal: Fix `ce-ai upgrade` Verification Drift Error and GitHub Token Discovery

## 1. Problem Statement
When running `ce-ai upgrade` to update to the latest version of the Compound Engineering plugin (e.g. `compound-engineering-v3.24.0`), users encounter two consecutive blocking failure modes:

1. **Unauthenticated GitHub API Rate Limiting (`HTTP 403 Forbidden` / Exit Code 5):**
   - GitHub's unauthenticated API rate limit is capped at 60 requests/hour per IP.
   - `ce-ai`'s `github_token_from_env()` exclusively checks the `CE_AI_GITHUB_TOKEN` environment variable.
   - Even when users have `GITHUB_TOKEN`, `GH_TOKEN`, or are authenticated via GitHub CLI (`gh auth login`), `ce-ai` ignores these credentials and queries `api.github.com/repos/everyinc/compound-engineering-plugin/releases` without authorization. This immediately fails with `HTTP 403 Forbidden`, in direct contradiction to documentation in `README.md` claiming fallback support.

2. **False Verification Drift on Native Harnesses (Exit Code 6):**
   - In commit `26a129d` ("feat(skills): harvest top-level skills tree and add adoption surface detection"), `ce-ai` transitioned to an adoption-only delivery model (R4 token-neutrality). In this model, `ce-ai install` and `ce-ai sync` intentionally do **not** copy harvested skills into native harness directories (`~/.claude/skills`, `~/.codex/skills`, etc.); only MCP companion servers (`codegraph`, `engram`) are registered.
   - However, during the post-sync verification step in `src/commands/sync.rs`, if a native harness (`claude`, `codex`, `copilot`, `grok`, `kimi`, `agy`, `pi`, `fx`) is host-detected but not adopted in `state.skill_surfaces` and not pending adoption, the verification logic still attempted to hash-verify all 393 release skills against `sync_skills_root(kind, &home_dir)`.
   - Because `ce-ai` deliberately never copied these files to the native harness directory, all 393 files were flagged as missing/drifted:
     ```
     error: verification error: sync verification failed for claude (393 drifted), copilot (393 drifted), codex (393 drifted), grok (393 drifted), kimi (393 drifted), agy (393 drifted), fx (393 drifted)
     ```
   - This aborts `ce-ai upgrade` and `ce-ai sync` with exit code 6 (`CeError::Verification`), leaving users unable to complete upgrades.

## 2. In-Scope vs Out-of-Scope Boundaries

### In-Scope:
- **Comprehensive GitHub Token Discovery:** Enhance `github_token_from_env()` to check `CE_AI_GITHUB_TOKEN`, then `GITHUB_TOKEN`, `GH_TOKEN`, and finally fall back to `gh auth token` via the GitHub CLI if present and authenticated.
- **Accurate Post-Sync Verification Matrix for Native Harnesses:** In `src/commands/sync.rs`, for table-driven native harnesses that are registered (MCP companions only) and not adopted in the adoption ledger, report `CheckStatus::NotVerified { reason: REASON_NO_MANAGED_SKILLS }` rather than verifying against a non-existent skills directory.
- **TDD Regression Tests:**
  - Unit tests for expanded GitHub token discovery.
  - CLI integration tests ensuring that host-detected native harnesses without adopted skills report `registered` and allow `sync` and `upgrade` to complete with exit code 0.

### Out-of-Scope:
- Modifying how `skills adopt` works or changing the adoption ledger schema.
- Changing `opencode` or `custom` harness file layouts or verification mechanics.

## 3. ISO/IEC 42001 & NIST AI RMF Risk Register

| Risk ID | Description | Severity | Mitigation |
| :--- | :--- | :--- | :--- |
| **R1** | Masking real drift on adopted skill surfaces | High | Ensure surfaces with `status == "adopted"` in `state.skill_surfaces` continue to be strictly hash-verified against their ledger files. |
| **R2** | Masking adoptable user skills | Medium | Ensure `pending_adoptions` check remains intact before the registered status check. |
| **R3** | CLI hanging on `gh auth token` | Low | Run `gh auth token` with proper process handling and fallback on failure without blocking CLI execution. |

## 4. Success Criteria
1. `ce-ai upgrade` runs successfully against GitHub releases without hitting rate limits when `GITHUB_TOKEN`, `GH_TOKEN`, or `gh` is authenticated.
2. `ce-ai upgrade` and `ce-ai sync` succeed with exit code 0 on machines where native harnesses (`claude`, etc.) exist on the host but are not adopted.
3. Verification matrix accurately reflects:
   ```
   reconciliation status: 1 verified, N registered (nothing to verify), 0 failed
   ```
4. 100% of existing unit and CLI integration tests pass (`cargo test`).
5. Zero clippy warnings with `-D warnings`.
