# Technical Exploration: Compound Capture & Doc Styling Alignment

## 1. Documentation Styling Audit

### Finding 1: README.md Section Order and Size
`docs/references/docs-styling.md` Section 2 specifies:
1. Title + what & why (max 3 lines)
2. Quick Path: the shortest verified route to value (`## Try it in two minutes`)
3. Documentation map: table linking into `docs/` with audience labels
4. Minimal pointers: security, contributing, license

Currently, `README.md` inserts `## Why CE-AI?` and `## How the pieces fit` (28 lines total) ahead of the Quick Path, pushing the Quick Path down to line 34. In addition, the file is 101 lines, exceeding the 100-line budget.
*Resolution*:
- Relocate deep positioning content (`Why CE-AI?`, `How the pieces fit`) into `docs/user-guide/ce-ai-positioning.md`.
- Lead with Title + What/Why (3 lines).
- Directly follow with Quick Path (`## Try it in two minutes`).
- Follow with Documentation Map and Project Links.
- Compress line count to ≤ 85 lines.

### Finding 2: Diátaxis Quadrant Violations
`docs/references/docs-styling.md` Section 1: "Never blend quadrants inside one document section."
- `docs/user-guide/project-adoption-guide.md` declares `> **Intent**: How-to & Explanation`.
  *Resolution*: Normalize to `> **Intent**: How-to`.
- `docs/references/docs-styling.md` Section 1 examples for Reference and Explanation are unlinked plain text.
  *Resolution*: Turn into relative markdown links to `docs/user-guide/harness-matrix.md` and `docs/user-guide/architectural-and-conceptual-guide.md`.

## 2. Knowledge Capture (Stage 6)

### OpenCode V2 Dual Loader Solution
The technical solution solved two distinct bugs:
1. OpenCode V2 config key changed from `"plugin"` to `"plugins"`.
2. OpenCode V2 enforces ESM `default` export schema: `export default { id, setup(ctx) }`, rejecting V1 `export const CompoundEngineeringPlugin = async...`.
3. Commands in V2 are no longer dynamically injected via `config` hook, requiring native markdown materialization in `commands/`.

We will structure this into `docs/solutions/bugfixes/opencode-v2-dual-loader.md` using the canonical schema.

### Monotonic Concept Accretion (CONCEPTS.md)
We will add 3 distinct domain terms to `CONCEPTS.md`:
- `OpenCode Dual Plugin Loader`: Hybrid ESM export pattern satisfying both OpenCode 1.x `server()` and OpenCode 2.x `default.setup(ctx)` contracts.
- `Native Command Materialization`: Generation of Markdown command files (`commands/<name>.md`) with frontmatter metadata for CLI hosts that lack runtime dynamic command registration hooks.
- `OpenCode Config Key Agnosticism`: Serialization and deserialization logic accepting either `"plugins"` or legacy `"plugin"` array keys in `opencode.json` without data loss or clobbering.

Each term will be appended under appropriate categories in `CONCEPTS.md` without modifying or removing any existing entry.
