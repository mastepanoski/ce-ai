# OpenSpec Exploration: Release v0.7.0 Technical Investigation

## Technical Alternatives Evaluated

### Option A: Direct State Mutation in Workspace File
- *Approach*: Write all state changes directly to `.ce-ai.json` if inside a Git repo.
- *Tradeoff*: Disrupts user expectation where global defaults exist. If `.ce-ai.json` is partially populated, global settings would be ignored.
- *Decision*: Rejected in favor of Option B.

### Option B (Selected): Layered Merging Engine (`State::load_with_overrides`)
- *Approach*: Load global `~/.config/ce-ai/state.json`, check if `.ce-ai.json` exists in `git_root()` or `std::env::current_dir()`, and merge field by field.
- *Rationale*: Preserves global defaults (installed harnesses, global model assignments) while allowing repositories to override specific slots (e.g. `ce-work` provider) or active profiles locally.

### Option C: Complete Harness Uninstall Dispatches per Adapter
- *Approach*: Extend `HarnessAdapter` trait with `uninstall_plugin(&self, ctx: &Context, all: bool) -> Result<(), CeError>`.
- *Rationale*: Clean modular separation where each harness adapter (OpenCode, Claude, Cursor, Copilot, Pi, Antigravity, etc.) knows its own plugin directory and manifest structure.

---

## Architectural Tradeoffs & Conclusions

- **Precedence Order**:
  1. CLI Flags / Arguments (highest priority)
  2. Local Workspace Config (`.ce-ai.json`)
  3. Global User Config (`~/.config/ce-ai/state.json`)
  4. System Defaults (lowest priority)
- **Uninstall Safety**:
  - `uninstall` without `--all` restores newest backup.
  - `uninstall --all` removes managed loaders, skills, and manifests across target harnesses.
