# Specification: Non-OpenCode Harness Sync and Upgrade

## Requirements

### REQ-1: Clean Sync for Standalone Native Harnesses
WHEN `ce-ai install --harness <kind>` is run for a non-OpenCode harness (e.g. `claude`, `copilot`, `cursor`, `pi`)
AND `ce-ai sync` is executed without OpenCode installed
THEN `ce-ai sync` SHALL exit with code 0
AND the verification matrix SHALL display the registered harness
AND the verification matrix SHALL NOT display an `opencode` row.

### REQ-2: Clean Sync for Standalone Custom Harnesses
WHEN `ce-ai install --harness custom ...` is run without OpenCode installed
AND `ce-ai sync` is executed
THEN `ce-ai sync` SHALL exit with code 0
AND the verification matrix SHALL display `custom` as verified (or failed on drift)
AND the verification matrix SHALL NOT display an `opencode` row.

### REQ-3: Fast-Fail When No Harnesses Are Installed
WHEN `ce-ai sync` is executed in an environment where no harnesses have been installed
THEN `ce-ai sync` SHALL exit with error code 1 (Runtime error)
AND stderr/stdout SHALL state: `"no harnesses installed — run ce-ai install first"`.

### REQ-4: Upgrade Works for Non-OpenCode Harnesses
WHEN `ce-ai upgrade` is executed in an environment where only a non-OpenCode harness is installed
THEN `ce-ai upgrade` SHALL successfully resolve the new release/source and update the installed harness without failing on a missing OpenCode manifest.
