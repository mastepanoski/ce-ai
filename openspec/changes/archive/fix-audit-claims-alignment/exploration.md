# Exploration

## Findings

`backup_file` derives a backup filename from a source path, but Kimi and FX
both produce `mcp.json`; AGY produces `mcp_config.json`. The destination loses
the original directory, while `newest_backup_for_harness` later infers a
harness from that destination filename. Kimi/AGY/FX therefore resolve as
`custom`, so uninstall skips the snapshot and unregisters CE companion names
from the live config.

The existing manifest records a backup path but uninstall chooses the latest
backup by inferred harness name. The narrowest compatible correction is to
make persisted backup filenames carry the harness identity for every native
ambiguous config path, preserving the existing list/restore API and legacy
backup behavior.

`workflow resume` probes local state, Git, manifests, and OpenSpec; it has no
Engram invocation. Checkpoint validation allows only adjacent stage labels,
but does not execute test commands or validate a solution document. The gate
checks OpenSpec artifact presence and emits an Organic LOC advisory, not test
or DoD evidence. Changing docs/help is safer and accurately scopes these
tools; adding a memory or CI runtime is a separate feature.

The AGY uninstall branch previously removed `antigravity.json` only after the
fallback MCP-name cleanup. A valid matching backup short-circuited that branch,
leaving the legacy CE artifact behind. Its removal must be independent from the
backup-vs-fallback choice.

## Alternatives rejected

1. **Restore by arbitrary latest snapshot:** rejected because it can restore a
   different harness's configuration.
2. **Use manifest backup paths only:** rejected for this fix because historical
   manifests and all call sites would need migration; identity-bearing backup
   names fix lookup uniformly and retain CLI backup browsing.
3. **Implement Engram/test execution now:** rejected as an unrelated product
   expansion with credentials, process, and policy implications.
