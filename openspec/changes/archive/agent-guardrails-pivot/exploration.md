# Exploration: Agent Guardrails & Risk Engine Pivot (`ce-ai guard`)

## Technical Investigation & Options Evaluated

### Option A: Retain Decision Engine As-Is and Document Manual Disabling
- **Description:** Keep `ce-ai decisions` unchanged. Instruct users in documentation to run `ce-ai decisions setup --preset off` if they don't want Jev or model routing.
- **Evaluation:**
  - *Pros:* Zero code modifications or migrations required.
  - *Cons:* Leaves ~1,000 LOC of dead routing and comparison code; continues to confuse new developers with hardcoded Anthropic model slots (`claude-3-5-*`); continues to trigger `doctor-warn: decision-provider-unauthenticated` in `ce-ai doctor` for standard setups. Leaves risk evaluation buried as a secondary subcommand (`ce-ai decisions check-risk`) rather than an actionable agent guardrail.
  - *Verdict:* **Ruled out.**

### Option B: Wholesale Deletion of `src/decisions/` (-3,500 LOC)
- **Description:** Completely rip out the entire `decisions` module, deleting all risk evaluation, provider interfaces, and CLI commands, reducing the binary to core Compound Engineering only.
- **Evaluation:**
  - *Pros:* Massive LOC reduction; eliminates `keyring` and `rpassword` dependencies; simplifies test suite.
  - *Cons:* Throws out valuable deterministic command security checks and semantic risk scoring. In modern autonomous workflows (headless CI, background subagents, `--dangerously-skip-permissions`), having an independent guardrail to block dangerous shell commands (`rm -rf /`, `git push --force origin main`, secret leaks) is a high-value safety differentiator.
  - *Verdict:* **Ruled out.**

### Option C: Pivot to Dedicated Agent Guardrails (`ce-ai guard`) (Selected)
- **Description:** Streamline the subsystem into a dedicated **Agent Guardrails & Risk Engine**. Decommission obsolete model routing (`route`, `compare`), dethrone cloud Jev as default in favor of local-first evaluation (Laya-MLX, Kev, or deterministic heuristics), and promote command risk evaluation to a first-class `ce-ai guard` command.
- **Evaluation:**
  - *Pros:*
    - Focuses the engine exclusively on high-value safety and policy enforcement for autonomous agents.
    - Eliminates ~800+ LOC of obsolete model routing and dead analytics.
    - Zero cloud network latency: deterministic safety rules execute in < 1 ms; local Laya-MLX executes in < 20 ms.
    - Eliminates false-positive `ce-ai doctor` warnings about missing `TYPESAFE_API_KEY`.
    - Preserves backward compatibility via command aliases.
  - *Verdict:* **Selected.**

## Architectural Trade-offs & Decisions

### 1. First-Class CLI Surface: `ce-ai guard` vs. `ce-ai decisions`
- **Decision:** Introduce `ce-ai guard` as the canonical command, while preserving `ce-ai decisions check-risk` as a backward-compatible alias.
- **Rationale:** "Decision Engine" implied a general-purpose brain choosing models, skills, and plans. "Guard" communicates exact intent: an operational security boundary and tool execution policy evaluator.

### 2. State Schema Migration & Backward Compatibility
- **Decision:** Keep `DecisionsConfig` in `state.json` or map `GuardConfig` transparently, deserializing existing configurations without requiring users to regenerate `state.json`.
- **Rationale:** Avoids breaking existing developer configurations across the adopted fleet of repositories.

### 3. Execution Latency & Deterministic Precedence
- **Decision:** Pre-execution checks MUST run deterministic string and regex checks before invoking any probabilistic provider (Laya, Kev, or Jev).
- **Rationale:** Catastrophic commands (`rm -rf /`, force push to main, `.env` / `.ssh` tampering) must be denied in sub-millisecond time without waiting for any model inference or network socket.
