# Exploration: canonical-skills-adoption

## Investigation (2026-08-24, session-verified)

Full causal chain, code claims, and institutional learnings were established during the session (ce-debug → brainstorm → doc-review → ce-plan research). Key verified facts driving the design:

1. **Harvest surface**: `MANAGED_PREFIXES` (install.rs:25, sync.rs:26) gates `read_local_tree`; `find_source_root` resolves the dir containing `.opencode/`, so top-level `skills/` is reachable by prefix extension alone. `skills_expected` (sync.rs:343-347) keys off the `skills/` manifest prefix, so the managed surface verifies unchanged.
2. **Destructive uninstall confirmed**: `uninstall.rs:229-257` runs `remove_dir_all` on harness skills dirs (claude/codex/copilot/grok/kimi/agy/pi/fx) — incompatible with adoption inside user-owned roots. Custom mode already does surgical manifest-scoped removal (L117-141) — the pattern to generalize.
3. **Registry gap confirmed**: Tier-3 scans `~/.ce-ai/harness-<kind>/skills` (no writer — dead); `collect_authorized_roots` already authorizes real harness dirs; `process_skill_file` with `target_harness: None` maps one path to all harnesses (canonical-store precedent at `~/.ce-ai/skills`).
4. **No interactive prompt infrastructure** in the CLI; `uninstall --yes` exists without a prompt; TUI spawns CLI vectors pinned by an anti-drift test. → adoption must be command-driven (`skills adopt`), never prompt-driven inside sync.
5. **Transactional toolkit exists**: `backup_file` (per-file, timestamped), `Journal::arm` (prior-bytes rollback, fault injection via `CE_AI_FAIL_AFTER_WRITES`), `write_atomic` everywhere.

## Evaluated options

- **Per-harness copies vs canonical + adoption**: copies rejected (token economy — description indexing per harness session is the cost; origin Key Decisions).
- **Prune vs adopt-in-place**: prune rejected (user preference: no deletions; adopt-in-place makes the pre-existing location canonical, avoiding double-index without removals — origin Key Decisions).
- **Adoption ledger in InstallManifest vs state.json**: manifest rejected — it is opencode-managed-dir-specific and dies with `uninstall --harness opencode`; adoption spans harnesses → `skill_surfaces` ledger in state.json (atomic via `State::save`).
- **Prompt-in-sync vs explicit adopt command**: prompt rejected (no-TTY/TUI hangs; R17) — sync reports `pending-adoption`, the command confirms.
- **Unconditional harvest vs conditional (origin R2)**: conditional retained — harvest writes the managed-dir copy only when no adoptable/adopted opencode surface exists; retirement (R13) covers both ledger-tracked and manifest-tracked surfaces so exactly one indexed surface per harness survives (plan doc-review P1 fix).

## Tradeoffs

- Ledger precision (manifest-driven indexing) over scanning whole harness roots: avoids indexing user skills as CE; cost — a manually dropped `ce-*` copy is invisible until adoption.
- External-origin detection scoped to known plugin-cache roots in v1: may under-report on layout drift; refinement deferred.
- Four chained PRs instead of one: honors the 400-line review boundary; sequencing rule — adoption execution and uninstall scoping land together (no destructive window).
