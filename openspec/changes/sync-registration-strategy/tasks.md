# Tasks: `sync-registration-strategy`

- [ ] **T1.1** Add `McpRegistrar`, `RegistrationSpec`, and the exhaustive
      `registration_spec()` table; collapse the nine copy-paste arms into the
      dispatch block. ✅ Existing 94 CLI tests green unchanged.
- [ ] **T1.2** Unit test pinning table contents (pi = no registrar, agy =
      `config/skills`, None set = {opencode, custom, deepseek}).
- [ ] **T2.1** Gates: fmt / clippy `-D warnings` / cargo test / make e2e.
- [ ] **T2.2** Bump 1.19.2 + CHANGELOG entry.
