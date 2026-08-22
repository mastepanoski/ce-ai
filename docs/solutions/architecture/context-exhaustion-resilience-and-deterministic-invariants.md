---
module: workflow/invariants
date: '2026-08-22'
problem_type: architecture_pattern
category: architecture
component: development_workflow
severity: high
applies_when:
- Preventing AI agent token dilution and context compaction from causing forgotten workflow rules in long sessions
- Enforcing deterministic GitHub API branch protection rules via scripts/protect-branch.sh and ce-ai doctor health probes
- Maintaining a compact Hard-Gate Invariant Index at top of AGENTS.md for high-priority prompt attention
tags:
- context-exhaustion
- workflow-invariants
- branch-protection
- doctor-probes
- agents-md
- hard-gate-invariants
- github-api
- token-dilution
---

# Architectural Solution: Deterministic Platform Boundaries vs. Probabilistic Prompt Decay

### Author: Master Computer Science Professor
### Topic: Enforcement Architecture for Autonomous AI Engineering (Issue #97 / ISO 42001 & NIST AI RMF)

---

## 1. Context & Core Dilemma (The Professor's Opening)

### The Great AI Illusion: "Why Did My AI Agent Push Directly to Main?!"
Welcome, junior engineer. Imagine you are working with a brilliant, hyper-fast pair programmer who has memorized every software engineering textbook, but suffers from severe short-term anterograde amnesia. In every extended coding session, as conversation turns to complex refactoring, test failures, and long stack traces, this developer gradually loses track of your initial working rules established hours ago.

In Large Language Model (LLM) agent engineering, this phenomenon is not a bug in the model's intelligence—it is a fundamental property of transformer architectures known as **Attention Decay and Token Dilution** within finite context windows.

Let's define three core computer science concepts that explain why relying on prose prompt rules for hard system invariants inevitably fails over time:

1. **Context Window Limits**: Every transformer model operates over a fixed sequence length limit (e.g., 128k, 200k, or 2M tokens). The context window is the model's entire working memory during an inference step, containing system instructions, conversation transcripts, tool outputs, and source code files.
2. **Compaction Loss (Summarization Drift)**: As pair-programming sessions extend across dozens of tool calls, the cumulative transcript exceeds working memory limits. Multi-agent systems trigger **compaction**—summarizing past turns into abbreviated notes to free up token capacity. Crucially, fine-grained prose instructions (such as *"Always run gh pr create instead of git push origin main"*) are often omitted, compressed, or lose their explicit imperative semantics during compaction.
3. **Token Dilution & Needles in Haystacks**: Even when instructions remain inside the context window, as total context grows from 2,000 to 100,000 tokens, the attention mechanism distributes softmax weights across millions of key-value vector pairs. A rule buried on line 450 of a 2,000-line prompt document loses mathematical weight relative to the massive incoming stream of code files and error tracebacks.

### The Core Dilemma
If your repository governance standard or compliance framework (e.g., ISO/IEC 27001, ISO 42001, NIST AI RMF 1.0) dictates that **"Pushes to `main` must never occur directly without 100% green CI verification"**, trusting an LLM's probability distribution to honor a text rule is a fundamental architectural flaw. 

An LLM is **probabilistic**, whereas security boundaries must be **deterministic**.

---

## 2. The Architectural Shift (Deterministic Boundaries)

### Shift Left: From "Prose Memory Prompting" to "Deterministic Platform Boundaries"
Instead of asking, *"How can we write a prompt so persuasive that the AI never forgets?"*, master software architects ask: **"How can we engineer the environment such that violating the rule is physically impossible at the infrastructure layer?"**

This shift represents a move from **probabilistic prompt governance** to **fail-closed deterministic platform boundaries**:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Probabilistic Prompt Layer                      │
│   "AGENTS.md High-Density Hard-Gate Index" (Top of Context Window)     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                       Deterministic Client Probes                      │
│   `ce-ai doctor` Health Probes (.githooks/pre-commit & branch check)   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                      Deterministic Platform Enforcement                │
│   GitHub REST API Branch Protection (`main` PUT payload + CI Matrix)   │
└────────────────────────────────────────────────────────────────────────┘
```

### Defense in Depth: The 3-Tier Enforcement Model
1. **Tier 1: Infrastructure Boundary (GitHub REST API Branch Protection)**: Enforces hard rules at the server level. Even if an AI agent runs `git push origin main --force`, GitHub's API physically rejects the HTTP payload with an execution error.
2. **Tier 2: Client Diagnostic Boundary (`ce-ai doctor` Health Probes)**: Proactively inspects local git configuration (`core.hooksPath`), local hook scripts (`.githooks/pre-commit`), and GitHub API branch protection rules. If any check fails, `ce-ai doctor` flags the drift before code shipping occurs.
3. **Tier 3: Context Top-Loading Boundary (`AGENTS.md` Hard-Gate Index)**: Places a high-density, concise ~22-line invariant index at the very beginning of the system prompt / project instruction file (`AGENTS.md`). Because transformer attention prioritizes tokens at the absolute start and end of context ("primacy & recency bias"), top-loading ensures maximum attention weight when prompts are processed.

---

## 3. How It Was Solved (Step-by-Step Breakdown)

Let's examine how this architectural solution was implemented in `ce-ai` (Issue #97).

### Step 1: Automated Infrastructure Protection (`scripts/protect-branch.sh`)
We created a dedicated Bash automation script (`scripts/protect-branch.sh`) using the GitHub CLI (`gh api`) to programmatically apply branch protection rules to `main`:
- **Pre-flight Checks**: Validates that `gh` CLI is installed and authenticated (`gh auth status`).
- **Auto-Detection**: Dynamically resolves the target repository (`REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)`).
- **API PUT Payload**: Executes an HTTP `PUT` request to `repos/{owner}/{repo}/branches/main/protection` with a JSON payload that:
  - Requires strict status checks (`"strict": true`).
  - Mandates 100% green status checks across all matrix runners (`ubuntu-latest`, `macos-latest`, `windows-latest`, containerized E2E gate, security audit, Windows PowerShell gate).
  - Explicitly disallows force pushes (`"allow_force_pushes": false`) and branch deletions (`"allow_deletions": false`).

### Step 2: Diagnostic & Drift Prevention (`ce-ai doctor` Health Probes)
In `src/commands/doctor.rs`, we extended the `ce-ai doctor` diagnostic suite with automated health probes to continuously monitor compliance:
- **Git Hooks Probe**: Checks `git config --get core.hooksPath`. Normalizes Windows/Unix path separators (stripping trailing `/` and `\`) to ensure hooks route to `.githooks`. Verifies that `.githooks/pre-commit` exists.
- **Branch Protection Probe**: Inspects git remote URL to determine if the repository is hosted on GitHub (`github.com`).
  - **Offline/Unauthenticated Resilience**: Checks `gh auth status`. If offline or unauthenticated, outputs a non-blocking diagnostic message (`doctor-info: gh CLI unauthenticated or offline, skipping branch protection probe`) instead of panicking.
  - **Branch Protection Verification**: Queries `gh api repos/{repo}/branches/main/protection`. If missing or returning non-200 status (e.g. 404 or 403 without admin access), records a concrete finding (`branch-protection: main branch protection missing or unconfigured on {repo}`).

### Step 3: Prompt Top-Loading (`AGENTS.md` Hard-Gate Invariant Index)
We restructured `AGENTS.md` to feature a high-density, ~22-line header at the very top of the file (`## ⚡ Hard-Gate Invariant Index`).
- Summarizes the non-negotiable delivery rules (No direct commit to main, 100% green CI matrix, atomic writes, OpenSpec required, strict exit codes).
- Ensures that when LLMs ingest `AGENTS.md`, the critical invariants receive highest primacy attention weight.

---

## 4. Why This Works (The Deep Computer Science Lesson)

### Fail-Closed Determinism & Compliance Frameworks
In software safety and cybersecurity, a system is **fail-closed** if, upon encountering an error, unexpected state, or memory compaction, it defaults to a secure state that prevents unauthorized action.

- **ISO 42001 (AI Management System)** & **NIST AI RMF 1.0**: Require demonstrable safety controls, risk management, and governance over autonomous AI output.
- **ISO/IEC 27001 & 27002 (Security Controls)**: Require cryptographic integrity, access control, and strict change control workflows.

Relying on an AI model's "memory" or prompt adherence is **fail-open**: if context compaction strips the prompt rule, the default behavior of `git push` is to succeed!
By enforcing rules at the GitHub API level, we make the system **fail-closed**:
- Even if an AI agent experiences complete prompt loss, attempts `git push origin main`, and has zero awareness of repository rules, **the server physically drops the connection and rejects the commit**.

```
[ AI Agent Context Reset / Amnesia ]
                 │
                 ▼
     Runs `git push origin main`
                 │
                 ▼
┌────────────────────────────────────────────────┐
│   GitHub REST API Branch Protection Gate       │
│   HTTP 403 Forbidden: Direct pushes restricted │
└────────────────┬───────────────────────────────┘
                 │
                 ▼
         [ Push Rejected! ]
    (Zero Human Policy Breach)
```

---

## 5. When to Apply: Engineering Decision Guide

As a junior developer, how do you decide whether to use **Platform Enforcement** or **Model Prompts**? Use this decision matrix:

| Rule Type | Use Platform-Level Enforcement | Use Model System Prompts |
| :--- | :--- | :--- |
| **Security & Permissions** | Hard Branch Protection, OAuth Scopes, API Token Restrictions | Code style preference for auth variables |
| **Build & Test Quality** | CI Matrix Gates, Pre-commit Hooks, `cargo clippy -D warnings` | Instructions to write thorough unit tests |
| **State File Integrity** | Atomic Write helper functions (`write_atomic` with tempfile+rename) | Prompt instructions on schema organization |
| **Creative / Task Logic** | *N/A (Too rigid for compiler checks)* | Refactoring suggestions, docstring style, naming conventions |

**Rule of Thumb**: If violating a rule causes a security incident, data corruption, or broken deployment on `main`, **enforce it in code or platform infrastructure**. If violating a rule affects formatting or style, **guide it via system prompts**.

---

## 6. Concrete Code Examples

### A. Infrastructure Script (`scripts/protect-branch.sh`)
```bash
#!/usr/bin/env bash
# scripts/protect-branch.sh — Configures GitHub API Branch Protection for `main`
set -e

# Pre-flight auth check
if ! gh auth status >/dev/null 2>&1; then
    echo "[ERROR] GitHub CLI ('gh') is not authenticated or offline."
    exit 1
fi

REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)

PAYLOAD=$(cat <<EOF
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Build & Test (ubuntu-latest)",
      "Build & Test (macos-latest)",
      "Build & Test (windows-latest)",
      "Containerized E2E Gate (NIST AI RMF & ISO 42001)",
      "Supply Chain Security Audit (ISO 27001 / ISO 27002)",
      "Windows PowerShell Installer Gate (NIST SP 800-53)"
    ]
  },
  "enforce_admins": false,
  "allow_force_pushes": false,
  "allow_deletions": false
}
EOF
)

gh api -X PUT "repos/${REPO}/branches/main/protection" --input - <<< "$PAYLOAD"
```

### B. Diagnostic Health Probe (`src/commands/doctor.rs`)
```rust
// Git Hooks Health Probe
if let Ok(hooks_output) = std::process::Command::new("git")
    .args(["config", "--get", "core.hooksPath"])
    .current_dir(root_path)
    .output()
{
    if hooks_output.status.success() {
        let raw_val = String::from_utf8_lossy(&hooks_output.stdout);
        let hooks_val = raw_val.trim().trim_end_matches('/').trim_end_matches('\\');
        if !hooks_val.ends_with(".githooks") && hooks_path.file_name() != Some(std::ffi::OsStr::new(".githooks")) {
            findings.push(format!("git-hooks: core.hooksPath set to '{}', expected '.githooks'", hooks_val));
        }
    }
}

// GitHub Branch Protection Health Probe
if is_github && gh_authenticated {
    let prot_check = std::process::Command::new("gh")
        .args(["api", &format!("repos/{}/branches/main/protection", repo_str)])
        .output();

    if let Ok(prot_out) = prot_check {
        if !prot_out.status.success() {
            findings.push(format!("branch-protection: main branch protection missing or unconfigured on {}", repo_str));
        }
    }
} else if is_github {
    println!("doctor-info: gh CLI unauthenticated or offline, skipping branch protection probe");
}
```

### C. Top-Loaded Hard Invariants Header (`AGENTS.md`)
```markdown
# AGENTS.md — AI Agent Operating Directives

## ⚡ Hard-Gate Invariant Index (Non-Negotiable Delivery Rules)

All AI agents MUST enforce these hard invariants deterministically at every session start:
1. **Never Direct Commit to `main`**: All changes MUST be committed on feature branches (`feat/*` or `fix/*`) and opened as a PR (`gh pr create`).
2. **100% Green CI Matrix Gate**: NEVER merge a PR until ALL GitHub Actions CI matrix jobs pass (`gh pr checks --watch`).
3. **Atomic File Writes**: Mutations to `state.json` or `opencode.json` MUST use `crate::state::write_atomic`.
4. **Preserve User Configs**: NEVER overwrite unmanaged custom plugins or custom skills in `opencode.json`.
5. **No Dummy Fallbacks**: NEVER comment out failing assertions, mask errors with empty catches, or ignore CLI errors.
6. **OpenSpec Required**: NO code changes without formal spec in `openspec/changes/<feature_name>/`.
7. **Strict Exit Codes**: Map all errors to `CeError` enum exit codes (`0` Success, `1` Runtime, `2` Usage, `3` State, `4` IO, `5` Network, `6` Verification).
```
