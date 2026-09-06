# Design: Auto-configure RTK Hook Injection for Natively-Supported Harnesses

## 1. Module Architecture (`src/harness/rtk.rs`)

A dedicated module encapsulating all RTK-specific support matrices, command builders, filesystem checks, and opt-out evaluation:

```rust
pub fn is_rtk_supported(kind: HarnessKind) -> bool;
pub fn is_rtk_opted_out(skip_rtk_flag: bool, skip_companions_flag: bool) -> bool;
pub fn is_rtk_available() -> bool;
pub fn is_rtk_hook_configured(home: &Path, harness: HarnessKind) -> bool;
pub fn configure_rtk_hook(home: &Path, harness: HarnessKind, dry_run: bool, quiet: bool) -> Result<bool, CeError>;
pub fn unconfigure_rtk_hook(home: &Path, harness: HarnessKind, dry_run: bool, quiet: bool) -> Result<bool, CeError>;
```

### Support Matrix Definition
```rust
pub fn is_rtk_supported(kind: HarnessKind) -> bool {
    matches!(
        kind,
        HarnessKind::Claude | HarnessKind::Cursor | HarnessKind::Copilot | HarnessKind::Codex
    )
}
```

### Opt-Out Evaluation
Checks boolean CLI flags and environment variables (`"1"`, `"true"`, `"yes"`):
- `skip_rtk_flag` OR `std::env::var("CE_AI_SKIP_RTK")`
- `skip_companions_flag` OR `std::env::var("CE_AI_SKIP_COMPANIONS")`

### Command Builder & Process Execution
Ensures target directories (`$HOME/.claude`, `$HOME/.cursor`, `$HOME/.copilot`, `$HOME/.codex`) exist prior to execution to satisfy RTK's filesystem prerequisites:
- Claude: `rtk init -g --auto-patch --agent claude`
- Cursor: `rtk init -g --auto-patch --agent cursor`
- Copilot: `rtk init -g --copilot`
- Codex: `rtk init -g --codex`
- Environment: Sets `HOME` (and `USERPROFILE` on Windows) to the provided `home` path for hermetic isolation.

---

## 2. CLI Interface Extensions

### `ce-ai install` (`src/commands/install.rs`)
```rust
#[derive(clap::Args, Debug, Clone)]
pub struct Args {
    // ... existing flags ...

    /// Skip auto-configuring RTK hook injection.
    #[arg(long, default_value_t = false)]
    pub skip_rtk: bool,

    /// Skip configuring all companion tools (both MCP and hooks).
    #[arg(long, default_value_t = false)]
    pub skip_companions: bool,
}
```

### `ce-ai init-prj` (`src/commands/init_prj.rs`)
```rust
#[derive(clap::Args, Debug, Clone)]
pub struct Args {
    // ... existing flags ...

    /// Skip auto-configuring RTK hook injection.
    #[arg(long, default_value_t = false)]
    pub skip_rtk: bool,

    /// Skip configuring all companion tools (both MCP and hooks).
    #[arg(long, default_value_t = false)]
    pub skip_companions: bool,
}
```

---

## 3. Integration Points

### 1. `install.rs`
Upon installing a harness:
- Evaluate `is_rtk_opted_out(args.skip_rtk, args.skip_companions)`.
- If opted out: log notice if `!quiet` and skip.
- If not opted out:
  - If `is_rtk_supported(*harness_kind)`: invoke `configure_rtk_hook(&home_dir, *harness_kind, ctx.dry_run, ctx.quiet)`.
  - If not supported: if `!ctx.quiet`: log `"rtk: hook injection not supported for {harness_kind}, skipping"`.

### 2. `init_prj.rs`
Upon adopting a project:
- Evaluate `is_rtk_opted_out(args.skip_rtk, args.skip_companions)`.
- If not opted out, iterate detected supported harnesses in the project/state:
  - For each detected supported harness: invoke `configure_rtk_hook`.

### 3. `uninstall.rs`
Upon uninstalling a harness:
- If `is_rtk_supported(harness_kind)`: invoke `unconfigure_rtk_hook(&home_dir, harness_kind, ctx.dry_run, ctx.quiet)`.

### 4. `audit.rs`
Refactor `CliCompressionDetector`:
- For supported harnesses (`Claude`, `Cursor`, `Copilot`, `Codex`):
  - Missing RTK binary or missing hook -> `AuditStatus::Warn`.
  - Configured hook -> `AuditStatus::Pass`.
- For unsupported harnesses (`Opencode`, `Pi`, `Custom`, etc.):
  - -> `AuditStatus::Info` (explicitly noting that RTK hook injection is not supported for that harness).

### 5. `doctor.rs`
- Check installed supported harnesses from `state.installed_harnesses`.
- If a supported harness lacks the RTK hook:
  - Emits `doctor-warn` advisory (and appends to `findings` when `--strict` is enabled).
- If RTK is installed, print advisory on stdout alterations for wrapped subcommands like `gh issue view --comments`.
