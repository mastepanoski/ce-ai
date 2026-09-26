#!/usr/bin/env python3
"""Validate CONCEPTS.md monotonic accretion against git HEAD.

Usage:
    python3 scripts/validate-concepts.py [CONCEPTS.md path] [--allow-shrink] [--json]

Exit codes:
    0 — valid monotonic accretion (or permitted shrinkage)
    1 — destructive clobber detected (entries removed without scrub tag)
    2 — usage error or missing file
"""

import argparse
import json
import os
import re
import subprocess
import sys

PREAMBLE_TITLES = {
    "concepts",
    "overview",
    "table of contents",
    "introduction",
    "glossary",
}

HEADING_RE = re.compile(r"^#{2,4}\s+(.+?)\s*$", re.MULTILINE)
BULLET_ENTRY_RE = re.compile(r"^[-*]\s+\*\*([^*]+?)\*\*:", re.MULTILINE)
SCRUB_TAG_RE = re.compile(r"<!--\s*(?:scrub|retired):\s*([^*>]+?)\s*-->", re.IGNORECASE)


def extract_entries(text: str) -> set[str]:
    """Extract concept entry names from markdown text."""
    entries = set()
    for m in HEADING_RE.finditer(text):
        title = m.group(1).strip()
        if title.lower() not in PREAMBLE_TITLES:
            entries.add(title)
    for m in BULLET_ENTRY_RE.finditer(text):
        term = m.group(1).strip()
        if term.lower() not in PREAMBLE_TITLES:
            entries.add(term)
    return entries


def extract_scrubbed(text: str) -> set[str]:
    """Extract explicitly scrubbed or retired entry names."""
    scrubbed = set()
    for m in SCRUB_TAG_RE.finditer(text):
        for term in m.group(1).split(","):
            t = term.strip()
            if t:
                scrubbed.add(t)
    return scrubbed


def get_git_head_content(file_path: str) -> str | None:
    """Retrieve file content at git HEAD, if available."""
    try:
        real_file_path = os.path.realpath(file_path)
        file_dir = os.path.dirname(real_file_path) or "."
        # Determine repo-relative path
        repo_root = os.path.realpath(
            subprocess.check_output(
                ["git", "rev-parse", "--show-toplevel"],
                stderr=subprocess.DEVNULL,
                cwd=file_dir,
            ).decode("utf-8").strip()
        )

        rel_path = os.path.relpath(real_file_path, repo_root).replace("\\", "/")
        out = subprocess.check_output(
            ["git", "show", f"HEAD:{rel_path}"],
            stderr=subprocess.DEVNULL,
            cwd=repo_root,
        )
        return out.decode("utf-8", errors="replace")
    except Exception:
        return None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate monotonic accretion of CONCEPTS.md")
    parser.add_argument("path", nargs="?", default="CONCEPTS.md", help="Path to CONCEPTS.md")
    parser.add_argument("--allow-shrink", action="store_true", help="Permit entry removal without failing")
    parser.add_argument("--json", action="store_true", help="Emit output as JSON")
    args = parser.parse_args()

    file_path = args.path
    if not os.path.isfile(file_path):
        sys.stderr.write(f"validate-concepts: file not found: {file_path}\n")
        return 2

    try:
        with open(file_path, "r", encoding="utf-8") as f:
            current_text = f.read()
    except Exception as e:
        sys.stderr.write(f"validate-concepts: failed to read {file_path}: {e}\n")
        return 2

    current_entries = extract_entries(current_text)
    scrubbed_entries = extract_scrubbed(current_text)
    head_text = get_git_head_content(file_path)

    if head_text is None:
        # File is untracked or repository has no HEAD
        result = {
            "status": "clean",
            "path": file_path,
            "git_available": False,
            "current_entries_count": len(current_entries),
            "head_entries_count": 0,
            "added": sorted(list(current_entries)),
            "deleted": [],
            "scrubbed": sorted(list(scrubbed_entries)),
        }
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            print(f"CONCEPTS.md: untracked or new file ({len(current_entries)} entries defined)")
        return 0

    head_entries = extract_entries(head_text)
    deleted_raw = head_entries - current_entries
    unscrubbed_deleted = [e for e in deleted_raw if e not in scrubbed_entries]
    added = sorted(list(current_entries - head_entries))
    deleted = sorted(unscrubbed_deleted)

    is_clobbered = bool(deleted) and not args.allow_shrink
    status = "clobbered" if is_clobbered else "clean"

    result = {
        "status": status,
        "path": file_path,
        "git_available": True,
        "current_entries_count": len(current_entries),
        "head_entries_count": len(head_entries),
        "added": added,
        "deleted": deleted,
        "scrubbed": sorted(list(scrubbed_entries)),
    }

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        if is_clobbered:
            sys.stderr.write(
                f"ERROR: CONCEPTS.md clobber detected! {len(deleted)} existing entries were deleted from HEAD:\n"
            )
            for d in deleted:
                sys.stderr.write(f"  - {d}\n")
            sys.stderr.write(
                f"\nEmpirical diff: {len(current_entries)} entries present (was {len(head_entries)}, +{len(added)} added, -{len(deleted)} removed)\n"
            )
            sys.stderr.write(
                "Remediation: Use surgical Edit instead of Write, or add '<!-- scrub: <Term> -->' if deliberate.\n"
            )
        else:
            print(
                f"CONCEPTS.md: valid monotonic accretion — {len(current_entries)} entries (was {len(head_entries)}, +{len(added)} added, {len(deleted_raw - set(deleted))} scrubbed)"
            )

    return 1 if is_clobbered else 0


if __name__ == "__main__":
    sys.exit(main())
