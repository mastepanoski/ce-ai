# Exploration

## Findings

- The README leads with “Rust CLI,” “orchestrator,” and “FSM,” while onboarding says CE-AI only installs skills. Neither explains its workflow value or boundary.
- EveryInc’s official plugin currently describes 36 skills on 14 hosts. It owns the methodology and reusable skills; CE-AI should not present itself as a replacement.
- Upstream requires an issue before a non-maintainer PR and explicit approval for new skills, so outreach belongs in a feedback-first discussion issue.
- Gentle AI documents ODD as its everyday path. Alan Buscaglia created ODD; CE-AI adapts it rather than claiming it.

## Decision

Lead with “CE-AI is not another coding agent. It is an engineering workflow for coding agents.” Use a concise layer model, attribute Compound Engineering and ODD accurately, and draft an issue that requests feedback—not integration.

## Sources

- [Compound Engineering README](https://github.com/EveryInc/compound-engineering-plugin)
- [Compound Engineering contributing guide](https://github.com/EveryInc/compound-engineering-plugin/blob/main/CONTRIBUTING.md)
- [Gentle AI intended usage](https://github.com/Gentleman-Programming/gentle-ai/blob/main/docs/intended-usage.md)
