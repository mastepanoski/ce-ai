# Exploration: Fix `ce-ai upgrade` Verification Drift Error and GitHub Token Discovery

## 1. Technical Investigation

### Area 1: GitHub API Authentication & Token Discovery
In `src/source/release.rs`:
```rust
pub fn github_token_from_env() -> Option<String> {
    std::env::var("CE_AI_GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
}
```
GitHub's REST API enforces strict unauthenticated rate limits (60 req/hr per public IP address). In developer environments where CI, browsers, or other tools query GitHub, this unauthenticated pool quickly hits `HTTP 403 Forbidden`.

Developers normally have tokens available in one of several standard locations:
1. `CE_AI_GITHUB_TOKEN` (ce-ai specific)
2. `GITHUB_TOKEN` (standard GitHub Actions / CLI env var)
3. `GH_TOKEN` (GitHub CLI standard env var)
4. GitHub CLI keyring (`gh auth token`)

Expanding `github_token_from_env()` to check these standard sources hierarchically ensures seamless authentication without requiring manual export of `CE_AI_GITHUB_TOKEN`.

### Area 2: Native Harness Sync Verification Matrix
In commit `26a129d`, `ce-ai` stopped copying harvested skills into native harness directories (`~/.claude/skills`, `~/.copilot/skills`, etc.) to enforce token neutrality (R4) and avoid polluting host agent prompts. Adoption (`ce-ai skills adopt`) became the sole mechanism for placing skills under ce-ai management on native harnesses.

In `src/commands/sync.rs`, the verification matrix classifies each active harness:
1. **Adopted Surface:** Handled in lines 414–435. If tracked in `state.skill_surfaces` with `status == "adopted"`, files are verified against the recorded SHA256 hashes.
2. **Pending Adoption:** Handled in lines 436–442. If adoptable `ce-*` directories exist on disk, reported as `CheckStatus::PendingAdoption`.
3. **Native Harnesses (Unadopted, No Local Skills):** Lines 482–508 retained pre-R4 code:
   ```rust
   let skills_dir = sync_skills_root(kind, &home_dir);
   if skills_expected.is_empty() { ... }
   let drift = verify_tree_against(&skills_dir, &skills_expected);
   surfaces.push(SurfaceCheck {
       harness: name.clone(),
       status: CheckStatus::from_drift(skills_expected.len(), drift),
   });
   ```
Because `skills_expected` contains the 393 release skills and `skills_dir` contains no skill files (none were copied by install/sync), `verify_tree_against` flags all 393 files as missing, yielding `CheckStatus::Failed`.
When any surface fails, lines 581–586 trigger an error:
```rust
return Err(CeError::Verification(format!(
    "sync verification failed for {}",
    failed_surfaces.join(", ")
)));
```

Since `ce-ai` manages **no** skill files on unadopted native harnesses (only MCP companions), this surface is correctly classified as `CheckStatus::NotVerified { reason: REASON_NO_MANAGED_SKILLS }`, exactly matching the design documented in `guidance_note_lines()`:
> `'registered' = ce-ai wrote harness config only; it manages no files on that surface, so there is nothing to hash-verify.`

## 2. Tradeoffs Evaluated

- **Option A: Copy all skills into native harness directories during sync**
  - *Tradeoff:* Violates the explicit architectural invariant (R4 token neutrality) established in v1.20 and commit `26a129d`. Pollutes harness prompt contexts with hundreds of unselected skills.
  - *Verdict:* Rejected.
- **Option B: Classify unadopted native harnesses as `registered (nothing to verify)`**
  - *Tradeoff:* Aligns verification with actual file management reality. Preserves strict hash verification for adopted surfaces. Accurately reports MCP companion registration.
  - *Verdict:* Chosen.
