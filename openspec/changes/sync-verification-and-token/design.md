# Design: Fix `ce-ai upgrade` Verification Drift Error and GitHub Token Discovery

## 1. System Architecture & Component Interactions

The changes touch two core modules:
1. `src/source/release.rs`: GitHub token acquisition and release resolution.
2. `src/commands/sync.rs`: Post-sync reconciliation and verification matrix classification.

```
[ce-ai upgrade / install / sync]
           │
           ├─► github_token_from_env()
           │     ├── 1. CE_AI_GITHUB_TOKEN
           │     ├── 2. GITHUB_TOKEN
           │     ├── 3. GH_TOKEN
           │     └── 4. `gh auth token` fallback
           │
           └─► sync_with()
                 │
                 ├── 1. Reconcile OpenCode managed tree (Copy / Restore / Remove)
                 ├── 2. Re-register companion MCP servers per active harness
                 ├── 3. Update state.json & manifest
                 └── 4. Build Verification Matrix:
                       ├── OpenCode: Hash-verified against desired managed tree
                       ├── Custom: Hash-verified against snapshot
                       ├── Adopted Surface (in state.skill_surfaces): Hash-verified
                       ├── Pending Adoption (local ce-* present): CheckStatus::PendingAdoption
                       └── Native Harness (MCP only, not adopted):
                             CheckStatus::NotVerified { reason: REASON_NO_MANAGED_SKILLS }
```

## 2. Interface and Contract Changes

### `src/source/release.rs`
Update `github_token_from_env()` implementation:
```rust
pub fn github_token_from_env() -> Option<String> {
    for var in ["CE_AI_GITHUB_TOKEN", "GITHUB_TOKEN", "GH_TOKEN"] {
        if let Ok(tok) = std::env::var(var) {
            let trimmed = tok.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    // Fall back to GitHub CLI token if `gh` is installed and authenticated
    if let Ok(output) = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
    {
        if output.status.success() {
            let tok = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !tok.is_empty() {
                return Some(tok);
            }
        }
    }
    None
}
```

### `src/commands/sync.rs`
Update the verification check branch for native harnesses (`Claude`, `Codex`, `Copilot`, `Grok`, `Kimi`, `Agy`, `Pi`, `Fx`):
```rust
            } else if matches!(
                kind,
                HarnessKind::Claude
                    | HarnessKind::Codex
                    | HarnessKind::Copilot
                    | HarnessKind::Grok
                    | HarnessKind::Kimi
                    | HarnessKind::Agy
                    | HarnessKind::Pi
                    | HarnessKind::Fx
            ) {
                surfaces.push(SurfaceCheck {
                    harness: name.clone(),
                    status: CheckStatus::NotVerified {
                        reason: REASON_NO_MANAGED_SKILLS,
                    },
                });
            }
```

## 3. Data Integrity and State Preservation
- No schema changes to `state.json` or `opencode.json`.
- The atomic write guarantee (`crate::state::write_atomic`) is strictly preserved.
- The adoption ledger remains the single source of truth for whether a native harness has managed skills.
