---
name: sequential-thinking
description: "Dynamic, reflective step-by-step problem solving and hypothesis refinement"
argument-hint: "[thought or problem to analyze]"
scope: "global"
triggers:
  - "complex reasoning"
  - "debugging intricate bugs"
  - "architectural analysis"
  - "sequential thought"
  - "hypothesis testing"
  - "root cause diagnosis"
---

# Sequential Thinking Protocol

Dynamic, reflective step-by-step problem solving and hypothesis refinement. Use this protocol when tackling complex non-linear problems, diagnosing elusive software defects, evaluating intricate architectural trade-offs, or navigating multi-variable decision spaces.

## Core Operational Principles

1. **Explicit Step Progression & Dynamic Planning**:
   - Maintain active thought progression in each reasoning step: `Thought [N] / Estimated [M]`.
   - Update `Estimated [M]` dynamically as problem complexity expands or narrows during exploration.
   - Do not rush to premature conclusions; allow ideas to evolve and branch through systematic iterations.

2. **Hypothesis Formulation & Premise Tracking**:
   - Explicitly articulate hypotheses before inspecting implementation details or executing tests.
   - Clearly delineate observed empirical facts from unverified assumptions and prior beliefs.
   - For every candidate hypothesis, specify what empirical evidence is required to prove or falsify it.

3. **Dynamic Revision & Branch Management**:
   - When new evidence conflicts with an earlier premise, explicitly register a revision:
     `Revision: Revising Thought [K] because [Empirical Reason]`.
   - When multiple competing hypotheses remain viable, branch exploration explicitly and track parallel threads until sufficient evidence eliminates alternatives.

4. **Falsification & Negative Evidence Testing**:
   - Actively search for contradictory facts, boundary conditions, and counter-examples before declaring any hypothesis validated.
   - Rigorously test the contrapositive: "What would the system look like if this assumption were false?"

5. **Convergence & Actionable Synthesis**:
   - Transition to convergence only after competing hypotheses have been systematically tested and either validated or falsified.
   - Summarize the final validated reasoning chain and state clear, actionable next steps.

---

## Thought Execution Schema

When executing sequential thinking, format reasoning steps using this structured block:

```markdown
### Thought [N] / Estimated [M]
- **Intent**: [What this specific thought intends to explore, verify, or calculate]
- **Premise / Hypothesis**: [Underlying claim or assumption being tested]
- **Empirical Evidence**: [Observed data, code snippets, test logs, or system constraints]
- **Falsification Check**: [Counter-evidence evaluated, edge cases probed, or disproving criteria]
- **Deduction & Revision**: [Validated findings; note if revising an earlier Thought [K]]
- **Trajectory**: [Continue linear path | Branch alternative | Revise previous | Converge to synthesis]
```

---

## Exemplary Reasoning Flow

1. **Thought 1 / Estimated 4 (Problem Framing)**:
   Define the exact failure mode or architectural requirement; separate symptoms from candidate causes; set initial hypothesis space $H = \{H_1, H_2\}$.
2. **Thought 2 / Estimated 4 (Hypothesis Evaluation $H_1$)**:
   Inspect code and test logs against $H_1$; discover contradictory evidence in call flow; falsify $H_1$ with empirical citation.
3. **Thought 3 / Estimated 4 (Hypothesis Evaluation $H_2$ & Revision)**:
   `Revision: Revising Thought 1 scope`. Evaluate $H_2$ against observed state; find consistent supporting evidence across all test cases.
4. **Thought 4 / Estimated 4 (Convergence & Synthesis)**:
   Verify absence of remaining counter-evidence; synthesize root cause and formulate minimal, precise implementation plan.

---

## Directives for Autonomous AI Agents

- **In-Context Execution**: Execute the thought progression natively in your context window. Do not attempt to delegate to external RPC servers.
- **Evidence-First Grounding**: Ground every deduction in concrete source references (`file://` paths, line numbers, command outputs, or verified specifications).
- **Graceful Termination**: Once convergence is achieved and verified, terminate sequential thoughts and proceed immediately to implementation or user reporting.
