# Pull Request

## 📋 Summary

<!-- What does this PR change and why? Keep it short and direct. -->

## 🔗 Related Issues

<!-- Reference related issues so they close automatically on merge, e.g. `Closes #123`. -->

- Closes #

## 🏷️ Type of Change

<!-- Check exactly one primary type (Conventional Commits). -->

- [ ] `feat`: New feature (MINOR bump)
- [ ] `fix`: Bug fix (PATCH bump)
- [ ] `docs`: Documentation only
- [ ] `refactor`: Code change with no behavior change
- [ ] `test`: Test-only change
- [ ] `chore`: Tooling, CI, or maintenance
- [ ] `feat!` / `fix!`: Breaking API/CLI/schema change (MAJOR bump)

## 🧪 Verification Gates (Definition of Done)

<!-- Every gate must pass locally BEFORE requesting review. -->

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test`
- [ ] `make e2e` (containerized Docker E2E gate)
- [ ] New or changed behavior is covered by unit or CLI integration tests

## ✅ Compliance Checklist

- [ ] Branch name follows convention (`feature/*`, `fix/*`, `docs/*`)
- [ ] Commits follow [Conventional Commits](https://www.conventionalcommits.org/)
- [ ] OpenSpec change exists under `openspec/changes/<feature_name>/` (mandatory for any code change)
- [ ] SemVer bumped in `Cargo.toml`; `CHANGELOG.md` updated (shippable changes)
- [ ] No breaking CLI contract or JSON schema changes (or explicitly documented and reviewed)
- [ ] Mutations to `state.json` / `opencode.json` use `crate::state::write_atomic`
- [ ] Unmanaged user plugins/skills in `opencode.json` are preserved (no config clobbering)
- [ ] Errors map to standard `CeError` exit codes (0/1/2/3/4/5/6)
- [ ] No secrets, tokens, keys, or transient metadata committed

## 🤖 AI-Assisted Contribution Disclosure

<!-- Per opensource.guide: contributors remain responsible for reviewing AI-generated output. -->

- [ ] If AI tooling was used, all generated output was verified for accuracy, correctness, and project conventions

## 📝 Notes for Reviewers

<!-- Optional: extra context, before/after CLI output for UX-affecting changes, open questions, draft/WIP status. -->
