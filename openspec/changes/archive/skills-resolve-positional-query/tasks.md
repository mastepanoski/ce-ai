# Tasks: Skills Resolve Positional Query Support

Total Estimated Changed Lines: ~65 LOC (Forecast: well within 400 LOC budget).

- [x] **Task 1: Update `Action::Resolve` in `src/commands/skills.rs`** (~20 LOC)
  - Add positional `query_pos: Option<String>` with `#[arg(value_name = "QUERY")]`.
  - Change `query: String` to `query: Option<String>` with `#[arg(long)]`.
  - Resolve effective query with `query_pos.as_deref().or(query.as_deref()).unwrap_or("").trim()`.
  - Return `CeError::Usage` if the effective query is empty.

- [x] **Task 2: Add integration tests in `tests/cli.rs`** (~35 LOC)
  - `skills_resolve_accepts_positional_query`: verifies `ce-ai skills resolve sequential-thinking` exits 0.
  - `skills_resolve_accepts_flag_query`: verifies `ce-ai skills resolve --query sequential-thinking` exits 0.
  - `skills_resolve_without_query_fails_usage`: verifies `ce-ai skills resolve` exits 2.

- [x] **Task 3: Version Bump & Release Documentation** (~10 LOC)
  - Bump SemVer to `1.39.2` in `Cargo.toml` and `Cargo.lock`.
  - Add `1.39.2` entry in `CHANGELOG.md` referencing Issue #298.
