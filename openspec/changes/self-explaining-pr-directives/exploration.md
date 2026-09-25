# Exploration: Self-Explaining PR Directives & LOC Budget Interaction

## Conceptual Exploration: "A Change That Can Explain Itself"
In modern AI-assisted engineering workflows, the generation of code diffs has become frictionless, but human review remains the primary cognitive bottleneck. Reviewers are frequently presented with:
1. Long mechanical diffs without context on discarded approaches.
2. Unverified claims where the reviewer must repeatedly request CLI output or test logs.
3. PRs opened while CI or local automated checks are still failing.

The presentation slides formulate a dual paradigm:
- **"A change that can explain itself"**:
  > *"Today a pull request tells you what changed. It should be able to tell you what it ruled out, and which rule decided it."*
- **"And the back half is review"**:
  - **Evidence**: Attached upfront to the pull request, not asked for later.
  - **Browser / Automated Checks**: Run and passing before a human opens it.
  - **Routing**: Changes directed to whoever knows that area.

## Trade-off Analysis: PR LOC Limits vs. Self-Explaining Descriptions

A central architectural question is whether mandating self-explaining PRs and upfront evidence conflicts with repository PR size constraints (specifically `ce-ai`'s 400-line review boundary in `CONTRIBUTING.md` § "PR Size Boundaries & Bounded Corrections"):

| Option | Pros | Cons | Decision |
|---|---|---|---|
| **Option A: Relax the 400-Line Code Boundary to 600-800 lines** | Gives agents more room to bundle changes and test fixtures in a single PR. | Destroys reviewability. Large diffs inherently bundle multiple orthogonal decisions, making "explaining what was ruled out" noisy and impossible for a human to review. | **Rejected** |
| **Option B: Keep Strict 400-Line Code Boundary & Mandate Collapsible Markdown for Evidence** | Maintains small, atomic PRs where a single architectural choice can be clearly explained. Keeps PR description length within ~100–150 lines by folding raw logs into `<details>` widgets. | Requires agents to be disciplined in formatting logs rather than dumping stdout directly. | **Adopted** |
| **Option C: Only document in CONTRIBUTING.md without updating AGENTS.md managed blocks** | Zero code changes to `src/commands/init_prj.rs`. | Agents in adopted projects only read `AGENTS.md` and ignore `CONTRIBUTING.md`. Fails to govern agent behavior across ecosystem projects. | **Rejected** |

## Architectural Findings
1. **Synergy Between Small PRs and Self-Explanation**:
   - Small code changes (≤ 400 LOC, or ~200 LOC per task) possess a tight decision surface. In an atomic PR, distilling what was ruled out and which rule decided it takes 2–4 sentences, providing maximum signal to reviewers.
2. **Separation of Code Diff from Description Length**:
   - `git diff --numstat` measures code lines changed. Markdown in the PR body does not count against code LOC limits.
   - However, PR descriptions have their own cognitive budget (~100 lines, cap ~150 lines). Evidence must use `<details><summary>` blocks to provide complete proof without inflating visible line count.
3. **Block Version Coordination**:
   - Bumping `BLOCK_VERSION` 5 → 6 triggers `StaleVersion` detection in `ce-ai doctor` and `ce-ai status`, cleanly signaling operators to upgrade with `ce-ai init-prj`.
   - Test fixtures in `tests/cli.rs` (`CUR_BLOCK_VERSION`) must be updated in lockstep to avoid false drift assertions.
