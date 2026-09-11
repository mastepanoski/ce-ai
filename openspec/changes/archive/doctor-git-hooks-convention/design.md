# Design: Project-Aware Git-Hooks Probe

## Architectural Approach
The git-hooks probe in `src/commands/doctor.rs` inspects the repository root for adoption of the `.githooks` convention before making any assertion on `core.hooksPath`.

### Logic Flow
1. Check if `root_path.join(".githooks").exists()`. Let this boolean be `uses_githooks_convention`.
2. Execute `git config --get core.hooksPath`.
3. If successful:
   - Extract and normalize `hooks_val`.
   - Determine `points_to_githooks = hooks_val.ends_with(".githooks") || hooks_path.file_name() == Some(".githooks")`.
   - If `points_to_githooks`:
     - Verify `root_path.join(".githooks").join("pre-commit").exists()`. If missing, push finding: `"git-hooks: .githooks/pre-commit missing"`.
   - Else if `uses_githooks_convention`:
     - The repository has `.githooks/` checked in, but `core.hooksPath` points elsewhere (or is unset/redirected). Push drift finding:
       `"git-hooks: core.hooksPath set to '<hooks_val>', expected '.githooks'"`.
   - Else:
     - The repository does NOT use `.githooks/`. The non-`.githooks` hooks path indicates an alternate hook manager (e.g., Husky, lefthook).
     - Emit non-blocking informational notice:
       `"doctor-info: git-hooks core.hooksPath set to '<hooks_val>' (not the .githooks convention; skipping)"`.

## Interfaces & Exit Codes
No CLI arguments or interfaces change. Exit codes are preserved:
- Finding present: exit code 1 (or exit code 2 if `--strict` was triggered).
- No findings: exit code 0.
