---
module: src/commands/audit.rs
tags: [audit, token-efficiency, context-quality, multi-harness, capability-detectors]
problem_type: architecture
---

# Multi-Harness Token-Efficiency & Context-Quality Audit Engine

## Problem
`ce-ai doctor` enforces mandatory configuration invariants and fails on errors, but `ce-ai` lacked an advisory, scored audit to evaluate whether token-saving compressors are active, whether MCP server sprawl (>5 global servers) is wasting context, whether duplicated prompt blocks are burning tokens, or whether code-intelligence and memory sidecars (CodeGraph, Engram, Context7) are present across installed harnesses.

## Solution
Built `ce-ai audit` in `src/commands/audit.rs` using a **capability-based detector design** (`Detector` trait):

```rust
pub trait Detector {
    fn detect(&self, ctx: &AuditCtx, harnesses: &[HarnessKind]) -> Vec<AuditCheck>;
}
```

### Key Design Highlights
1. **Capability Matrix**:
   - `cli-output-compression` (`rtk` binary + interceptor hooks).
   - `mcp-sprawl` (MCP server count >5 check per harness).
   - `prompt-duplication` (Scans agent prompt instructions for duplicated blocks $\ge$ 200 chars across $\ge$ 3 agents).
   - `persistent-memory` (`engram` DB existence).
   - `docs-grounding` (`context7` entry).
   - `code-intelligence` (`.codegraph/` index presence).
   - `learnings-library` (`docs/solutions/` doc count).
2. **Advisory Scoring**:
   - Scores capability checks as `(PASS*1.0 + WARN*0.5) / TotalApplicable * 100`.
   - Purely advisory by default (Exit 0).
3. **CI Threshold Gates**:
   - `--json` renders machine-readable output.
   - `--fail-under <pct>` exits with non-zero exit code if score falls below required percentage.
