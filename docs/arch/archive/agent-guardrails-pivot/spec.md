---
title: "Agent Guardrails & Risk Engine Pivot (ce-ai guard)"
domain: gate
version: 1.0.0
last_updated: "2026-10-18"
---

# Specification: Agent Guardrails & Risk Engine Pivot (`ce-ai guard`)

## Requirement 1: Dedicated Guard Command & Deterministic Precedence

### WHEN `ce-ai guard check <TOOL> <COMMAND>` is invoked with a catastrophic command
- **THEN** the deterministic security evaluator MUST immediately return `Deny` (or `RequireConfirmation` for sensitive operations) in sub-millisecond execution time.
- **THEN** it MUST NOT perform any external network requests or wait for background daemon sockets when a deterministic match is found.
- **THEN** it MUST identify hardcoded violations including, but not limited to:
  - Destructive file deletion (`rm -rf /`, `rm -rf ~`, `rm -rf .*`)
  - Force-pushing to protected branches (`git push --force origin main`, `git push -f origin master`)
  - Secret and credential exfiltration (`cat ~/.ssh/id_*`, reading `.env` files)
  - Unsanitized root script execution (`curl ... | sudo bash`)

### WHEN `ce-ai guard check <TOOL> <COMMAND> --json` is executed
- **THEN** it MUST serialize a valid JSON object containing:
  - `tool`: String identifier of the tool (e.g. `run_command`).
  - `command`: Target command string.
  - `policy`: One of `"Allow"`, `"RequireConfirmation"`, or `"Deny"`.
  - `composite_score_pct`: Numerical score (0 to 100).
  - `reason`: Explanation of the policy determination.
  - `latency_ms`: Execution latency in milliseconds.
- **THEN** the process MUST exit with code `0` (Success), communicating the policy inside the payload rather than crashing the tool caller.

## Requirement 2: Backward Compatibility for Existing CLI Consumers

### WHEN `ce-ai decisions check-risk` is executed by existing scripts or agent hooks
- **THEN** the CLI dispatcher MUST delegate execution directly to the `guard check` handler.
- **THEN** it MUST support all existing flags (`--task`, `--verbose`, `--json`) and produce identical stdout/stderr formatting.

## Requirement 3: Deprecation of Model Routing and Dead Analytics

### WHEN `ce-ai decisions route <task>` is executed
- **THEN** the CLI MUST display a deprecation advisory notice explaining that modern AI harnesses manage model assignments natively.
- **THEN** the process MUST exit with code `0`.

### WHEN `ce-ai decisions compare` is executed
- **THEN** the CLI MUST display a deprecation notice and exit cleanly with code `0`.

## Requirement 4: Local-First Setup and Clean Doctor Diagnostics

### WHEN `ce-ai guard setup --preset recommended` (or `ce-ai decisions setup --preset recommended`) is executed
- **THEN** on Apple Silicon macOS (`aarch64`), it MUST configure local provider `laya` with fallback to deterministic rules.
- **THEN** it MUST NOT prompt for or mandate a third-party `TYPESAFE_API_KEY`.

### WHEN `ce-ai doctor` is executed on a default or newly initialized project
- **THEN** it MUST report guardrail operational status (`doctor-info: guard: active (...)`).
- **THEN** it MUST NOT emit `doctor-warn: decision-provider-unauthenticated` when TypeSafe AI / Jev credentials are absent.
