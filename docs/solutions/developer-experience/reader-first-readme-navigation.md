---
module: documentation
tags: [readme, onboarding, navigation, documentation-map, zero-step-drift]
problem_type: developer_experience
title: "Reader-First README Navigation"
applies_when: "When simplifying a project README without hiding installation paths or useful guides."
---

# Reader-First README Navigation

## Problem

A concise public positioning pass removed concrete installation options and direct links to useful guides. Replacing them with a link to a documentation directory forced readers to browse raw Markdown filenames and hid important concepts such as Zero-Step Drift Recovery.

## Solution

Keep the value proposition at the top, but preserve a direct, audience-labeled documentation map below it:

- Show macOS/Linux, Windows PowerShell, Homebrew, and source installation commands.
- Link to each guide directly; do not use a directory listing as a navigation destination.
- Preserve discoverability for operational concepts, especially Zero-Step Drift Recovery, installation/coexistence, upgrades, and backup/uninstall.
- Keep maintainer or outreach framing in its dedicated draft, not in reader-facing explanatory guides.

## Verification

Confirm every local Markdown link resolves, all intended install paths remain present, and the README stays within its 100-line budget.
