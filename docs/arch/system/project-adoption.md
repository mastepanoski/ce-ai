---
title: "Project Adoption & De-Adoption Engine"
domain: project-adoption
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Project Adoption & De-Adoption Engine

## 1. Overview & Architectural Boundaries

The project adoption engine (`ce-ai init-prj` / `ce-ai deinit-prj`) allows external software repositories to adopt Compound Engineering workflows non-destructively through delimited marker blocks, supporting multi-tier governance.

## 2. Capabilities & Requirements

### R1. Non-Destructive Marker Injection (`init-prj`)
WHEN adopting a project via `ce-ai init-prj`  
THEN the system MUST inject delimited instruction blocks (`<!-- ce-ai:block begin ... -->` ... `<!-- ce-ai:block end -->`) into `AGENTS.md` and generate derived stubs (`CLAUDE.md`, `GEMINI.md`, etc.) without overwriting existing user instructions.

### R2. Adoption Tiers
WHEN adopting a project  
THEN `ce-ai init-prj` MUST support:
- `--tier minimal`: Lightweight directives with permissive gate checks.
- `--tier full`: Complete 7-stage workflow directives and active gate enforcement.

### R3. Clean De-Adoption (`deinit-prj`)
WHEN removing adoption via `ce-ai deinit-prj`  
THEN the system MUST surgically remove managed marker blocks, restore pre-adoption files if backups exist, and leave user customizations completely intact.

### R4. Dry-Run Honesty
WHEN running `ce-ai init-prj --dry-run` or `deinit-prj --dry-run`  
THEN the command MUST write zero bytes to disk while accurately previewing planned changes.

## 3. Data Models & CLI Contracts

- `AdoptionTier`: `Minimal`, `Full`.
- Delimited marker syntax: `<!-- ce-ai:block begin v=<version> tier=<tier> sha256=<hash> -->`.
- CLI commands: `ce-ai init-prj [--tier <tier>] [--dry-run]`, `ce-ai deinit-prj [--dry-run]`.

## 4. Invariants & Operational Boundaries

- Reversible: Every file modified by `init-prj` must be cleanly restorable by `deinit-prj`.
- Unmanaged user content outside marker boundaries MUST NEVER be altered or deleted.
