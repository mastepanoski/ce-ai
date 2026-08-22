---
title: "Skill Registry Engine Code Review Refactorings & Security Boundary Hardening"
date: "2026-08-22"
category: "code-fixes"
module: "src/source/registry.rs"
component: "skill_registry"
severity: "medium"
problem_type: "code_fix"
symptoms:
  - "Tier 3 harness-specific skills bleeding into unrelated host agent harness registries"
  - "Runtime panics on non-ASCII skill descriptions due to UTF-8 byte slicing"
  - "Malformed trigger arrays containing unstripped brackets [a, b]"
  - "Overly permissive R3 authorized security root boundaries"
root_cause: "Unscoped directory scanning, direct byte offset string slicing, unstripped YAML array brackets, and broad authorized root paths"
resolution_type: "code_fix"
---

# Skill Registry Engine Code Review Refactorings & Security Boundary Hardening (Release v1.4.1)

## Problem

After the initial release of the Skill Registry Engine (v1.4.0), a comprehensive 9-reviewer `ce-code-review` panel identified critical architectural, stability, and security vulnerabilities across the codebase:
1. **Cross-Harness Path Bleed**: Tier 3 harness-specific skills were being scanned and loaded into unrelated host harnesses due to missing harness isolation in directory scanning.
2. **Scope Metadata Loss**: When higher-precedence skills overrode existing entries in `SkillRegistry`, the original scope metadata was dropped or overwritten incorrectly during precedence resolution.
3. **UTF-8 Byte Slicing Panic Risks**: String truncation for display formatting relied on raw byte indexing (`&str[..37]`), which risks panicking at runtime when slicing multibyte Unicode characters or emojis.
4. **Inline Array Bracket Corruption**: Metadata values containing inline array brackets (e.g. `[brainstorm]`) were misparsed or rendered with corrupted leading/trailing bracket characters.
5. **Overly Permissive Security Boundaries**: `collect_authorized_roots` granted filesystem access permissions to overly broad parent directory trees rather than restricting authorization strictly to targeted harness subdirectories (`~/.config/opencode`, `~/.config/<harness>`), while silently swallowing `set_permissions` error results.
6. **Non-Hermetic Integration Tests**: CLI integration tests in `tests/cli.rs` inherited the ambient process working directory instead of isolating execution within temporary test directories.

## Symptoms

- **Harness Cross-Contamination**: Skills defined exclusively for OpenCode were mistakenly discovered, validated, and registered when running under other host harness environments.
- **Runtime Panics on Internationalization**: Any skill containing non-ASCII characters or emojis in its description crashed the process with UTF-8 byte boundary slice panics (e.g., `byte index 37 is not a char boundary`).
- **Malformed Trigger Arrays**: Skill triggers defined with inline brackets like `[brainstorm]` appeared as `[brainstorm]` or contained residual bracket formatting in CLI outputs and model prompts.
- **Overly Broad Permission Scope**: File permission checks (R3 security rules) authorized entire user config directories rather than specific harness skill roots, while permission application failures failed silently without alerting operators.
- **Flaky & Ambient-Dependent CLI Tests**: `cargo test` runs failed inconsistently across environments because CLI tests executed in the global working directory instead of sandbox directories.

## What Didn't Work

- **Un-Scoped Global File Scanning**: Relying on generic directory traversals without filtering by `HarnessKind` allowed Tier 3 skills meant for a specific agent framework to leak across framework boundaries.
- **Naive String Byte Slicing**: Using standard Rust byte slicing (`&desc[..37]`) on UTF-8 strings assumes ASCII encoding, causing fatal runtime crashes when encountering multibyte UTF-8 code points.
- **Unsanitized Value Parsing**: Directly taking YAML/TOML parsed array strings without stripping enclosing bracket characters caused bracket duplication and parsing anomalies.
- **Coarse Authorized Directory Traversal**: Granting filesystem permissions at parent directory levels violated the principle of least privilege, opening wider attack surfaces. Ignoring `Result` from `set_permissions` masked filesystem permission enforcement failures.

## Solution

Release v1.4.1 introduced systematic fixes, refactorings, and security hardenings across all identified areas:

### 1. Harness-Scoped Skill Scanning (`Option<HarnessKind>`)

Updated `scan_skill_directory` and `process_skill_file` to accept an `Option<HarnessKind>` parameter. When scanning Tier 3 skills, directory matching strictly filters out skills that do not match the target host harness framework.

### 2. Scope Metadata Preservation

Guaranteed that when a skill override takes place during registry insertion or precedence evaluation, the target entry's scope metadata is preserved:
```rust
entry.scope = scope;
```

### 3. UTF-8 Character-Aware String Truncation

Replaced raw byte slicing with character iterator collection:
```rust
let truncated_desc: String = skill.description.chars().take(37).collect();
```
This safely handles any UTF-8 sequence, including non-ASCII characters, symbols, and multi-byte emojis, preventing boundary panics.

### 4. Inline Array Bracket Stripping

Cleaned up inline array bracket formatting during value parsing:
```rust
let clean_val = clean_val.trim_start_matches('[').trim_end_matches(']');
```
This guarantees clean extraction of array items regardless of whether input values contain surrounding brackets.

### 5. Narrowed Authorized Roots & Explicit Error Propagation

Restricted `collect_authorized_roots` to specific harness skill subdirectories (such as `~/.config/opencode` and `~/.config/<harness>`). Propagated errors from `set_permissions` explicitly instead of ignoring `Result` values, ensuring filesystem permission failures are caught and surfaced immediately.

### 6. Encapsulated SkillRegistry Helpers & `SkillFrontmatter` Struct

Refactored internal registry operations by encapsulating helper methods `SkillRegistry::sync_registry` and `SkillRegistry::remove`. Introduced a formal `SkillFrontmatter` struct for type-safe parsing and validation of skill metadata.

### 7. Isolated CLI Integration Tests

Updated CLI integration tests in `tests/cli.rs` to set test-local working directories:
```rust
cmd.current_dir(tmp.path());
```
This ensures hermetic test execution that cannot read or mutate host directory states.

## Why This Works

- **Strict Harness Isolation**: `Option<HarnessKind>` filtering guarantees Tier 3 skills are strictly bound to their intended host harness, preventing cross-harness leakage and unintended side effects.
- **Unicode Resilience**: Iterator-based character truncation operates at Unicode scalar value boundaries, eliminating slice panics across internationalized text.
- **Clean Metadata Parsing**: Bracket trimming ensures metadata values render predictably and without formatting artifacts in downstream CLI tools and agent prompts.
- **Principle of Least Privilege**: Restricting security roots to targeted harness subdirectories minimizes the attack surface. Explicit permission error propagation prevents silent security failures.
- **Encapsulated & Hermetic Architecture**: Standardized structs (`SkillFrontmatter`) and helper methods (`sync_registry`, `remove`) reduce code duplication and enforce consistency. Hermetic tests prevent workspace side-effects and flaky CI builds.

## Prevention

To prevent similar regressions in future releases:
1. **Multi-Harness Scoping Protocols**: Always thread harness/framework context through file scanning and registration pipelines. Never assume global visibility for framework-specific assets.
2. **Unicode-Safe String Handling**: Avoid byte slicing (`&str[..N]`) on user-supplied or dynamic strings in Rust. Always use `.chars().take(N)` or `unicode-segmentation` grapheme clusters when truncating text.
3. **Strict Minimum Security Boundaries**: Maintain minimal authorized directory scopes. Never grant wildcard or parent-level permissions when specific subdirectories suffice. Always check and propagate `Result` types on filesystem operations.
4. **Hermetic Test Environments**: Ensure all unit and integration tests execute inside isolated temporary directories (`tempfile::TempDir`) with explicit `.current_dir()` configuration.
