# OpenSpec Technical Design: Project Adoption Engine

## Architecture & Struct Specifications

### 1. Data Schema Updates (`src/state/state.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdoptionTier {
    Full,
    Minimal,
    Orchestrator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectAdoptionEntry {
    pub path: PathBuf,
    pub file: String,
    pub tier: AdoptionTier,
    pub block_version: u32,
    pub block_sha256: String,
    pub created_file: bool,
    pub adopted_at: String,
}
```

Add `projects: Vec<ProjectAdoptionEntry>` to `State` struct in `src/state/state.rs`.

### 2. Subcommand CLI Contracts (`src/main.rs`)

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Adopt a project repository by injecting managed Compound Engineering workflow blocks into AGENTS.md
    InitPrj {
        /// Target project directory path (default: current working directory)
        path: Option<PathBuf>,

        /// Adoption tier: full (7-stage cycle + OpenSpec), minimal (lightweight guidelines), orchestrator (agent-directed variant)
        #[arg(long, default_value = "full")]
        tier: String,

        /// Force overwrite of modified managed blocks if SHA mismatch is detected
        #[arg(long)]
        force: bool,
    },

    /// Remove managed Compound Engineering workflow blocks from a project repository cleanly
    DeinitPrj {
        /// Target project directory path (default: current working directory)
        path: Option<PathBuf>,
    },
}
```

### 3. Subcommand Modules (`src/commands/init_prj.rs` & `src/commands/deinit_prj.rs`)

- **`src/commands/init_prj.rs`**:
  - Resolves target project root using `git rev-parse --show-toplevel`.
  - Reads or creates `AGENTS.md`.
  - Renders tier-specific template (`templates/blocks/full.md`, `minimal.md`, `orchestrator.md`).
  - Computes SHA256 of inner managed block.
  - Injects block enclosed in `<!-- ce-ai:block begin v=1 tier=... -->` and `<!-- ce-ai:block end -->`.
  - Performs atomic write via `crate::state::write_atomic`.
  - Updates adoption registry in `state.json` via `write_atomic`.
  - Generates derived reference stubs (e.g. `CLAUDE.md` containing `@AGENTS.md`).

- **`src/commands/deinit_prj.rs`**:
  - Locates project in adoption registry or searches `AGENTS.md` for `<!-- ce-ai:block -->` markers.
  - Strips marker-enclosed segment from `AGENTS.md`.
  - If `created_file` is true and remaining file content is empty/whitespace only, deletes `AGENTS.md` and derived stubs.
  - Otherwise, saves cleaned `AGENTS.md` atomically.
  - Removes entry from adoption registry in `state.json` via `write_atomic`.

### 4. Harness Adapter Trait Extension (`src/harness/mod.rs`)

```rust
pub trait HarnessAdapter {
    fn name(&self) -> &'static str;
    fn is_installed(&self) -> bool;
    fn canonical_instruction_file(&self) -> PathBuf;
    fn derived_stub_files(&self) -> Vec<PathBuf>;
}
```

### 5. TUI Dashboard Integration (`src/tui.rs`)

- Add `[I] Init Prj` and `[D] Deinit Prj` key action shortcuts to `MenuTab::Workflow` and `MenuTab::Status` panels in `src/tui.rs`.
- Dispatch CLI execution through `run_cmd` and display completion notifications in TUI status line.
