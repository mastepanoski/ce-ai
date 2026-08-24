# Exploration: `sync-registration-strategy`

## Why a table instead of trait-object Strategy

The user-facing review of PR #206 challenged the if/else chain. The textbook
Strategy pattern (trait + per-harness impls) was evaluated and rejected for
this surface:

1. **The eight native arms already share one signature** —
   `fn(&Path, &str, &str, &[&str], &BTreeMap<String,String>) -> Result<(), CeError>`
   — verified across cursor/claude/codex/copilot/grok/kimi/agy/fx. They differ
   in exactly two data points: the function itself and the skills subpath.
   Data beats polymorphism when variation is data.
2. **Rust matches give compile-time exhaustiveness**, which is the property
   that actually kills the bug class (forgotten arm → silent fictional write).
   Trait objects would move that guarantee to runtime or lose it entirely.
3. **Three kinds genuinely don't fit the shared shape**: opencode writes
   plugin/skills JSON, custom is state-snapshot-driven, deepseek is
   de-scoped. Forcing them through one trait yields option-littered impls.

## Follow-up debt

`install.rs` carries the analogous chain (~10 arms with richer per-vendor
steps). Consolidating it needs a slightly richer spec (backup handling,
per-arm manifests) and should land as its own change once this table proves
out in production.
