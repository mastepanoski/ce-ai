# Brainstorm: Cursor sessionStart Lifecycle Hook Integration

## Context
Cursor supports lifecycle hooks via `.cursor/hooks.json`. In Cursor v0.45+, `sessionStart` fires in both the desktop editor and the Cursor CLI when a composer session starts.

## Requirements
1. The hook must invoke `ce-ai workflow resume --json`.
2. The JSON payload must include `additional_context` so Cursor injects the live RepoState directly into the conversation's initial context.
3. User hooks and settings must be preserved on write.
4. Clean removal on de-adoption.
