# Artificial Intelligence Management & Governance Policy (AI Policy)

`ce-ai` provides plugin lifecycle management, skill delivery, and model assignment for autonomous AI coding agents across AI harnesses.

This policy establishes our AI Governance System, adhering strictly to:
- **ISO/IEC 42001:2023** — Artificial Intelligence Management System (AIMS)
- **NIST AI Risk Management Framework (AI RMF 1.0)** — Govern, Map, Measure, Manage
- **EU Artificial Intelligence Act** — Transparency, human oversight, and risk mitigation standards

---

## 🏛️ 1. GOVERN (ISO 42001 Clause 5 & NIST AI RMF Govern 1.1–6.2)

### 1.1 Organizational Principles
- **Human Agency & Oversight**: AI agents managed by `ce-ai` act strictly under explicit human direction. No autonomous modification of critical system parameters occurs without user approval.
- **Accountability**: The human engineer operating `ce-ai` retains ultimate authority and responsibility over all code generated or modified by AI tools.
- **Transparency**: All agent slot assignments, model providers, and prompt skills installed by `ce-ai` are inspectable in readable JSON (`state.json`, `opencode.json`) and markdown format.

### 1.2 Model & Provider Neutrality
- `ce-ai` provides model-agnostic assignment capabilities across multi-vendor LLMs (e.g. OpenAI, Anthropic, Kimi, DeepSeek, Google Gemini).
- Model assignments are explicitly scoped per agent role (e.g. `ce-brainstorm`, `ce-plan`, `ce-work`, `ce-code-review`, `ce-debug`) to enforce appropriate capability matching.

---

## 🗺️ 2. MAP (NIST AI RMF Map 1.1–5.2)

### 2.1 AI Agent Risk Classification
`ce-ai` maps potential risks associated with AI agent plugin management into three key risk vectors:

| Risk Vector | Description | Severity | Mitigation Strategy |
| ----------- | ----------- | -------- | ------------------- |
| **Model Drift** | Agent slot executing under unvetted model parameters | Medium | `ce-ai models` state tracking & profile snapshots |
| **Skill Tampering** | Unauthorized prompt injection or skill file corruption | High | SHA256 cryptographic manifest verification (`ce-ai sync`) |
| **Config Overwrite** | Destruction of developer harness settings during install | High | Pre-install timestamped backups & atomic restoration |

---

## 📐 3. MEASURE (NIST AI RMF Measure 1.1–4.3)

### 3.1 Quality & Integrity Metrics
- **Asset Integrity Index**: 100% of installed plugin assets must match their registered SHA256 checksums.
- **Diagnostic Verification**: `ce-ai doctor` executes automated structural validation of harness JSON files, checking schema validity and drift metrics.
- **Isolated Validation Gate**: All releases undergo containerized execution in `Dockerfile.e2e` to measure execution stability in clean environments.

---

## 🛠️ 4. MANAGE (NIST AI RMF Manage 1.1–4.3)

### 4.1 Safety & Fallback Controls
- **Dry-Run Mode (`--dry-run`)**: Enables engineers to preview all filesystem mutations, file copies, and JSON changes before writing to disk.
- **Deterministic Uninstallation**: Instant, 1-command rollback (`ce-ai uninstall`) restores pre-installation harness states cleanly.
- **Profile Snapshotting**: `ce-ai models profile save <name>` captures immutable snapshots of model assignments to prevent accidental drift across development sessions.

---

## 📜 Compliance Statement

By using `ce-ai`, organizations ensure that their AI coding assistant plugins, agent configurations, and model assignments follow standardized, auditable, and secure AI governance frameworks compliant with **ISO 42001** and **NIST AI RMF**.
