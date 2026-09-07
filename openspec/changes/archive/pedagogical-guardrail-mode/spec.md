# Specification: Pedagogical Guardrail Mode (`ce-ai guard`)

## Requirements Matrix

### 1. Enable Guardrail Mode (`ce-ai guard enable`)

- **REQ-GUARD-1.1 (Default Activation):**
  - **WHEN** the user executes `ce-ai guard enable` without arguments,
  - **THEN** `state.json` MUST be updated atomically with `guardrail: { enabled: true, level: "junior", harness: null, updated_at: "<ISO8601>" }` and return exit code 0.

- **REQ-GUARD-1.2 (Strict Level):**
  - **WHEN** the user executes `ce-ai guard enable --level strict`,
  - **THEN** `state.json` MUST be updated atomically with `guardrail.level = "strict"` and return exit code 0.

- **REQ-GUARD-1.3 (Harness Scoping):**
  - **WHEN** the user executes `ce-ai guard enable --harness claude`,
  - **THEN** `state.json` MUST persist `guardrail.harness = Some("claude")`.

- **REQ-GUARD-1.4 (Invalid Level Error):**
  - **WHEN** the user executes `ce-ai guard enable --level invalid_level`,
  - **THEN** the CLI MUST reject the command with `CeError::Usage` (exit code 2) and descriptive error text listing valid levels (`junior`, `strict`).

- **REQ-GUARD-1.5 (Dry-Run Protection):**
  - **WHEN** the user executes `ce-ai guard enable --dry-run`,
  - **THEN** the command MUST print the planned activation without modifying `state.json` or any disk file.

---

### 2. Disable Guardrail Mode (`ce-ai guard disable`)

- **REQ-GUARD-2.1 (Clean Deactivation):**
  - **WHEN** the user executes `ce-ai guard disable`,
  - **THEN** `state.json` MUST record `guardrail.enabled = false` atomically, printing a clean deactivation confirmation and returning exit code 0.

- **REQ-GUARD-2.2 (Idempotency):**
  - **WHEN** `ce-ai guard disable` is executed on an already-disabled or unset guardrail state,
  - **THEN** the command MUST succeed idempotently with exit code 0.

- **REQ-GUARD-2.3 (Dry-Run Protection):**
  - **WHEN** `ce-ai guard disable --dry-run` is executed,
  - **THEN** no disk changes MUST be made.

---

### 3. Status Reporting (`ce-ai guard status`)

- **REQ-GUARD-3.1 (Human-Readable Output):**
  - **WHEN** the user executes `ce-ai guard status`,
  - **THEN** the CLI MUST print:
    - Status (`enabled` or `disabled`),
    - Active level (`junior` or `strict`),
    - Scope (`global` or harness name),
    - Last updated timestamp.

- **REQ-GUARD-3.2 (JSON Output):**
  - **WHEN** the user executes `ce-ai guard status --json`,
  - **THEN** the CLI MUST emit a JSON payload adhering to:
    ```json
    {
      "enabled": true,
      "level": "junior",
      "harness": null,
      "updated_at": "2026-08-28T00:00:00Z"
    }
    ```

---

### 4. System & Health Integration

- **REQ-GUARD-4.1 (Doctor Integration):**
  - **WHEN** `ce-ai doctor` runs,
  - **THEN** it MUST inspect the guardrail state and surface:
    - `[OK] Guardrail: enabled (junior)` when active, or
    - `[INFO] Guardrail: disabled` when inactive.

- **REQ-GUARD-4.2 (State Backward Compatibility):**
  - **WHEN** `State::load` reads a legacy `state.json` lacking the `guardrail` key,
  - **THEN** it MUST deserialize successfully with `state.guardrail == None`.
