# OpenSpec Specification: Multi-Harness Operations

## Specifications

### MH-1: Multi-Harness Bulk Sync and Upgrade
- **WHEN** `ce-ai sync` or `ce-ai upgrade` is executed with `--harness all` (or no harness flag),
- **THEN** `ce-ai` MUST iterate through all host-installed harnesses, performing sync/upgrade for each target and outputting itemized progress.

### MH-2: Local Source Upgrade Guard
- **WHEN** `ce-ai upgrade` targets a harness whose installation source is `local`,
- **THEN** `ce-ai` MUST skip the release upgrade with a protective warning unless `--force` is provided.

### MH-3: TUI Global Target Harness Selector
- **WHEN** navigating action tabs in the TUI dashboard,
- **THEN** the header and action panels MUST display `Target Harness: < All Installed / harness_name >` and allow switching using `◄`/`►` or `h`/`l`.
