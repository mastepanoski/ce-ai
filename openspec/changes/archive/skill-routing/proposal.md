# Proposal: Dynamic Skill Selection & Injection (Intelligent Skill Routing)

## Problem Statement

As the `ce-ai` Skill Registry index expands across official Compound Engineering skills, project-level skills, and custom domain skills, discovering and injecting the right skills for a given task becomes increasingly challenging:
1. **Keyword-matching brittleness**: Pure string/substring matching fails when task descriptions use natural phrasing or synonyms that don't literally contain the skill name or hardcoded trigger keywords (e.g., "audit token expiry in cookie headers" might not match `security-review` if only "security" is in the trigger list).
2. **Context Window Token Bloat**: Passing the full catalog of available skills into the primary LLM prompt consumes thousands of tokens and causes distraction or hallucination on simple tasks.
3. **Security & Boundary Preservation**: While fast probabilistic classification (System 1) can identify high-level user intent, it must never be allowed to bypass local file boundaries, inject unverified code, or load arbitrary paths outside the existing SHA256 cryptographic registry.

Issue [#384](https://github.com/mastepanoski/ce-ai/issues/384) introduces **Intelligent Skill Routing** as an advisory layer: the Pluggable Decision Engine identifies relevant semantic categories (`security`, `testing`, `architecture`, `debugging`, `review`, etc.), and the authoritative, deterministic Skill Registry then resolves, bounds, verifies, and formats the actual matching skill artifacts.

---

## In-Scope

1. **Semantic Category Intent Classification**:
   - Query the Decision Engine with category-specific evaluation questions:
     - `needs_architecture` (system design, boundaries, domain modeling)
     - `needs_security` (vulnerabilities, authentication, authorization, cryptography)
     - `needs_testing` (unit tests, integration tests, E2E, mock strategies)
     - `needs_debugging` (stack traces, regressions, memory leaks, error tracing)
     - `needs_documentation` (architecture notes, guides, ADRs, user documentation)
     - `needs_code_review` (code quality, standard compliance, pull request reviews)
     - `needs_research` (literature, benchmark exploration, codebase discovery)
   - Evaluate returned confidence scores against a configurable confidence threshold (`minimum_confidence_pct: u32 = 70`).

2. **Skill Classification Metadata & Indexing**:
   - Extend `SkillEntry` and `SkillFrontmatter` in `src/source/registry.rs` to optionally parse `categories` from `SKILL.md` YAML frontmatter:
     ```yaml
     ---
     name: security-review
     description: Review authentication and sensitive endpoints
     categories: [security, review, authentication]
     ---
     ```
   - Maintain backward compatibility: skills without explicit categories are categorized by sensible fallback heuristics (e.g. matching standard category keywords against existing `name`, `triggers`, and `description`).

3. **Deterministic Precedence & Integrity Invariant**:
   - The Decision Provider returns **candidate categories only**, never filesystem paths or file contents.
   - The authoritative `SkillRegistry` remains 100% responsible for:
     - Skill existence and scope resolution (workspace `.ce-ai` > workspace `.opencode` > global).
     - Cryptographic SHA256 integrity verification.
     - Security boundary traversal validation (`canonicalize_and_validate_path`).
     - Prompt markdown injection.

4. **Graceful Degradation & Fallback Behavior**:
   - Feature is optional and disabled by default (`enabled = false`).
   - If routing is disabled, encounters an error, times out, or trips the circuit breaker, `ce-ai` immediately degrades to standard deterministic keyword resolution with exit code 0.

5. **CLI & Diagnostic Ergonomics**:
   - `ce-ai skills resolve --harness <harness> "<task>" [--json] [--verbose]`: Transparent diagnostic output displaying category confidence scores, selected candidate categories, and resolved skills.
   - `ce-ai doctor`: Checks skill routing configuration and reports status.
   - Presets: Update `ce-ai decisions setup --preset recommended` to enable skill routing.

---

## Out-of-Scope

1. **Direct Arbitrary Code Execution**: Skills remain static markdown prompt files (`SKILL.md`); no dynamic code execution or remote skill downloading is introduced.
2. **Automated Live Turn-0 Tool Interception**: Harness-specific hook generation is handled by existing harness integration surfaces.
3. **Live Token Usage Metering**: Tracked separately under Issue [#387](https://github.com/mastepanoski/ce-ai/issues/387).

---

## Risk Evaluation & Mitigation

- **Risk 1: Misclassification injects irrelevant skills into agent context.**
  - *Mitigation:* Conservative confidence thresholding (default: 70% confidence) and bounded top-$K$ category limits prevent prompt clutter.
- **Risk 2: External provider failure delays skill resolution.**
  - *Mitigation:* Decision engine timeout (1,000ms) with immediate fallback to standard deterministic keyword search ensures zero CLI stalls.
- **Risk 3: Malicious or hallucinated provider output attempts path traversal.**
  - *Mitigation:* Provider outputs only typed category strings; the local `SkillRegistry` exclusively resolves local, authorized, SHA256-verified filesystem paths.

---

## Success Criteria

1. 100% offline testability: `MockDecisionProvider` can simulate any combination of category classifications deterministically.
2. Zero degradation in existing `ce-ai skills list`, `resolve`, or `adopt` commands.
3. Security boundary checks pass 100% in `tests/security.rs`.
4. Full compliance with `AGENTS.md` DoD: zero Clippy warnings, 100% test pass across all matrix platforms.
