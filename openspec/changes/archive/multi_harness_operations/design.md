# OpenSpec Design: Multi-Harness Execution & TUI Target Management

## Architecture & Data Flow

```
[ CLI / TUI Command Dispatch ]
        │
        ├── Target Resolution: "all" or specific harness
        │
        ├── Harness Probe: Iterate over state.installed_harnesses + host CE installations
        │
        ├── Safety Check: If source == local and upgrading -> prompt or require --force
        │
        └── Execution: Apply install/sync/upgrade per harness target
```

## Data Schema & Struct Updates
- `upgrade::Args`: Add `force: bool` flag to allow overriding local source upgrade protection.
- `sync::run`: Accepts target harness filter (defaults to all active installed harnesses).
- `tui::App`: Global `selected_harness_target_idx` allowing selection of `All Installed` or any individual host harness.
