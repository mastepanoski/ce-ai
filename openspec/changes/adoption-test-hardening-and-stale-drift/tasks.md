# Tasks: Adoption v2 Test Hardening & Stale-Block Doctor Signal

- [ ] U1: LF-only v1→v2 replacement integration test (Covers S1).
- [ ] U2: Malformed-block fail-closed test (Covers S2).
- [ ] U3: sha256 triangle consistency test (Covers S3).
- [ ] U4: Doctor stale-version targeted finding (Covers S4) — parse `v=` in doctor.rs; `AdoptionTier::as_str` helper.
- [ ] U5: Tampered-v2 generic drift test (Covers S5).
- [ ] U6: Full gates — fmt, clippy `-D warnings`, `cargo test`, `make e2e`.
