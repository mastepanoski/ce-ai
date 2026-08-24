# OpenSpec Exploration: Technical Tradeoffs for Multi-Harness Operations

## Options Evaluated
1. **Single-Harness Default with Manual Loops**: Require the user to pass `--harness <name>` repeatedly for every harness.
   - *Tradeoff*: High user friction; cumbersome for developers using 3+ harnesses simultaneously.
2. **Bulk Multi-Harness Dispatch with Local Source Guards (Chosen Option)**: Default to `--harness all` (or target selected harness), iterate over all active installed host harnesses, protect local trees (`source: local`), and render clear itemized results in CLI and TUI.
   - *Tradeoff*: Requires iterating over active state entries and probing host config paths, but provides an intuitive, friction-free UX.
