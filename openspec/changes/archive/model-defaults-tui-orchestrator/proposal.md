# Proposal: model-defaults-tui-orchestrator

Refs: GitHub issue #111.

## Problem

1. ce-ai writes `agent.<slot>.model` assignments into `opencode.json` but nothing seeds sane defaults at install time; every assignment depends on manual `ce-ai models set`.
2. There is no orchestrator agent slot named `ce-ai`; slots are per-stage only (`ce-brainstorm`, `ce-plan`, `ce-work`, ...).
3. The TUI "Models & Profiles" tab is read-only; users must fall back to the CLI to customize models.
4. State/config desync is invisible: a state reset leaves stale `opencode.json` assignments while `models list` reports none (observed on host: `agent.ce-brainstorm` present, `state.json.model_assignments` empty).

## In scope

- Structural `ce-ai` orchestrator agent definition seeded by `install` into harness configs that support agent entries (description, mode `primary`, permissions) — **never** `model` or `variant`; users customize those.
- No default model seeding anywhere: assignments are always user-driven via CLI/TUI.
- Editable Models tab in the TUI (slot navigation + model picker backed by live harness discovery via `opencode models`).
- All TUI action output captured through a subprocess so `println!`-based commands can no longer corrupt the alternate-screen layout.
- `doctor` detects drift between `state.json` and `opencode.json` model assignments (CE-known slots); `sync` repairs desync by importing config→state.

## Out of scope

- Creating agent definitions (prompt/mode/permission) for the `ce-ai` slot — only the model assignment slot.
- Multi-harness model config (assignments target opencode.json only, as today).
- New CLI subcommands.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Defaults clobber user customization | Medium | Apply defaults only when the slot has no `model` in opencode.json AND no state assignment |
| Sync import overwrites newer state with older config | Low | Import direction is config→state (config is the live effective truth); snapshot written before change |
| Doctor false positives on non-ce agent slots | Medium | Only evaluate slots tracked in state or matching known CE slots |
| Harness CLI missing/unavailable in picker | Medium | Discovery errors surface explicitly in a modal; never silently fall back to a static list |

## Success criteria

- Fresh install yields `ce-ai`, `ce-brainstorm`, `ce-plan`, `ce-work` defaults without touching pre-existing slots.
- TUI can set a slot's model from the harness's live catalog without leaving the dashboard.
- The observed desync scenario (config-only assignment) is reported by `doctor` and repaired by `sync`.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test` green.
