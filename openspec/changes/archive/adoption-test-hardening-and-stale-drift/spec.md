# Spec: Adoption v2 Test Hardening & Stale-Block Doctor Signal

## ADDED Requirements

### Scenario 1: LF-only v1→v2 replacement
WHEN a project's AGENTS.md contains a hand-written v1 block with LF endings plus surrounding user content
THEN re-running init-prj replaces only the marker-delimited region with the v2 block, preserving user content and LF endings.

### Scenario 2: Malformed block fails closed
WHEN AGENTS.md contains a begin marker without an end marker
THEN init-prj exits non-zero with a runtime error and does not write partial content.

### Scenario 3: Header/body/state sha256 triangle
WHEN any adoption block is written
THEN the header `sha256=` equals the SHA256 of the body between markers AND equals `state.json`'s `block_sha256`.

### Scenario 4: Stale-version signal in doctor
WHEN doctor inspects a project whose on-disk block header declares a version older than BLOCK_VERSION and its body hash differs from the current template
THEN doctor reports a targeted finding naming the stale version and the exact upgrade command (`re-run ce-ai init-prj --tier <tier>`), not generic drift.

### Scenario 5: Tampered v2 body still reports drift
WHEN a v2 block's body is mutated without changing its declared version
THEN doctor reports the existing generic "block SHA drift" finding.
