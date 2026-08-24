# Design: Multi-Harness Reconciliation, DeepSeek De-scope & Release Fallback Hardening

## 1. Supported Harness Matrix (10 Native Adapters)
`ce-ai` supports 10 native AI agent harnesses:
- `opencode`: `~/.config/opencode/opencode.json` (OpenCode JSON)
- `claude`: `~/.claude.json` / `~/.claude/settings.json` (Claude JSON)
- `pi`: `~/.pi/agent/skills/` (Pi Skills)
- `cursor`: `~/.cursor/mcp.json` / `~/.cursorrules` / `.cursor/rules/` (Cursor JSON & MDC)
- `copilot`: `~/.config/github-copilot/mcp.json` / `~/.github/copilot-instructions.md` (Copilot JSON & Markdown)
- `codex`: `~/.codex/config.toml` (Codex TOML)
- `grok`: `~/.grok/config.toml` (Grok TOML)
- `kimi`: `~/.kimi-code/mcp.json` (Kimi Code JSON)
- `agy`: `~/.gemini/config/mcp_config.json` (Antigravity JSON)
- `fx`: `~/.fx/mcp.json` (Fx JSON)

## 2. DeepSeek De-scope Handler
- Attempts to run CLI subcommands (`install`, `uninstall`, `sync`, `init-prj`, `deinit-prj`, `tools`) targeting `deepseek` return `CeError::Usage` (exit code 2):
  `"deepseek harness is unsupported during developer preview (DeepSeek Harness 'dsh' uses YAML patch layers under ~/.dsh). Please use a supported native harness (opencode, claude, codex, copilot, cursor, grok, kimi, agy, pi, fx)."`
- `HarnessKind::detect_installed_harnesses` and `HarnessKind::detect_ce_installed_harnesses` filter out `Deepseek` so `install --harness all` never attempts to install DeepSeek.
- `GenericJsonAdapter` removes `HarnessKind::Deepseek` arm.

## 3. GitHub API 403 & Network Error Fallback Architecture
In `src/source/release.rs`:
When `resolve_latest_release` encounters a network send error or non-success HTTP status (403, 429, etc.):
```rust
let response = match request.header(reqwest::header::USER_AGENT, "ce-ai/0.1.0").send() {
    Ok(res) => res,
    Err(err) => {
        eprintln!(
            "notice: GitHub API release query network error: {err}. Falling back to main branch source tarball. Tip: set CE_AI_GITHUB_TOKEN."
        );
        return Ok(None);
    }
};

if !response.status().is_success() {
    eprintln!(
        "notice: GitHub API returned HTTP {} when querying releases. Falling back to main branch source tarball. Tip: set CE_AI_GITHUB_TOKEN.",
        response.status()
    );
    return Ok(None);
}
```

## 4. Audit Configuration Coverage Labeling
In `src/commands/audit.rs`:
- Output summary header: `configuration coverage: {}% ({} pass / {} warn / {} fail)`
- `--fail-under` flag description: `"Fail with exit code 1 if configuration coverage is below this threshold percentage"`
- Threshold error message: `"configuration coverage {}% is below required threshold of {}%"`
