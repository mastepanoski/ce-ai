# Brainstorm: Antigravity PreInvocation Turn-0 Drift Delivery

## Context
Google Antigravity CLI does not have a literal `SessionStart` event, but its `PreInvocation` hook allows injecting ephemeral messages into the model prompt before each turn.

## Requirements
1. The hook must only inject `RepoState` once per conversation (Turn 0).
2. Uses `conversationId` received over `stdin` to manage a session marker file.
3. Subsequent invocations within the same session return `{}` to prevent duplicate prompt flooding.
