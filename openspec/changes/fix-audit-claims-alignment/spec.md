# Specification: Audit claims alignment

## Requirements

### R1. Identifiable native backup snapshots
WHEN `ce-ai install` backs up an existing Kimi, AGY, or FX configuration
THEN the persisted backup filename MUST identify its harness and
`ce-ai backups list --harness <name>` MUST discover it.

### R2. Lossless Kimi uninstall restoration
WHEN a Kimi configuration contains user-owned MCP entries before installation
AND CE installation overwrites a companion-name entry
THEN `ce-ai uninstall --harness kimi` MUST restore the original configuration
from its matching pre-install backup before removing managed assets.

### R3. Truthful resume boundary
WHEN users inspect CLI help or public workflow documentation
THEN it MUST describe `workflow resume` as rehydrating local checkpoint,
repository, manifest, and OpenSpec context, without claiming an Engram memory
read.

### R4. Truthful enforcement boundary
WHEN documentation describes checkpoints or `gate check`
THEN it MUST distinguish transition/artifact validation from execution of test
or knowledge-capture evidence.

### R5. Truthful model mutation documentation
WHEN documentation describes `models set`
THEN it MUST NOT claim that the command creates a pre-mutation harness-config
backup unless the implementation does so.

### R6. AGY cleanup after restoration
WHEN `ce-ai uninstall --harness agy` restores a matching configuration backup
THEN it MUST still remove the CE-managed legacy `antigravity.json` artifact.

### R7. Separate companion ownership
WHEN user documentation describes Engram memory behavior
THEN it MUST identify Engram as a separately installed/configured sidecar and
MUST NOT attribute its MCP calls or storage to the `ce-ai` binary.
