# Specification: Deterministic Tasks Tail Validator

## Requirements

### R1. Deterministic Validation
- **WHEN** `validate-tasks-tail.py` is invoked on a `tasks.md` associated with code changes
- **AND** any of the lifecycle elements (simplification, review, compound) are missing
- **THEN** it MUST exit with code 1 and output the specific missing items.

### R2. Idempotent Auto-Fix
- **WHEN** `validate-tasks-tail.py --fix` is invoked on a `tasks.md` missing lifecycle elements
- **THEN** it MUST append the missing lifecycle tail unit to `tasks.md`
- **AND** it MUST exit with code 0.
- **AND** a second invocation with `--fix` MUST NOT duplicate the appended block.

### R3. Documentation Exemption
- **WHEN** `validate-tasks-tail.py` is invoked on a change modifying only documentation or configuration
- **THEN** it MUST report the change as exempt and exit with code 0.

### R4. Zero Dependencies
- **WHEN** `validate-tasks-tail.py` is executed on any system with standard Python 3.8+
- **THEN** it MUST execute without requiring any third-party pip packages.
