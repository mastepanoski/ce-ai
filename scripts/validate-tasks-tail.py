#!/usr/bin/env python3
"""Validate presence of the post-implementation lifecycle tail in OpenSpec tasks.md.

Usage:
    python3 scripts/validate-tasks-tail.py [tasks.md path] [--fix] [--check-code] [--json]

Exit codes:
    0 — Valid lifecycle tail present, successfully fixed, or exempt
    1 — Missing lifecycle tail for code-touching change
    2 — Usage or IO error
"""

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

LIFECYCLE_TAIL_TEMPLATE = """
- [ ] **Unit Final: Lifecycle Completion & Governance Gate**
  - [ ] **Simplification (`ce-simplify-code`)**: Audit changed code for reuse, quality, and efficiency while preserving behavior.
  - [ ] **Revisión formal (`ce-code-review`)**: Run review and record `ce-ai workflow review-receipt`.
  - [ ] **Captura de conocimiento (`ce-compound`)**: Capture learnings in `docs/solutions/<category>/` and monotonically accrete `CONCEPTS.md`.
  - [ ] **Verificación de cierre**: Comply with DoD (`cargo fmt`, `cargo clippy`, `cargo test`, `make e2e`, `doc lint --strict`).
"""


def find_repo_root() -> Path:
    """Find the git root directory or fallback to current working directory."""
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
        return Path(out)
    except Exception:
        return Path(os.getcwd())


def resolve_tasks_path(raw_path: str = None) -> Path:
    """Resolve the target tasks.md file from raw path or active repository state."""
    repo_root = find_repo_root()

    if raw_path:
        p = Path(raw_path)
        if not p.is_absolute():
            p = repo_root / p
        if p.is_dir():
            p = p / "tasks.md"
        return p

    # Auto-discovery in openspec/changes/ (excluding archive)
    changes_dir = repo_root / "openspec" / "changes"
    if changes_dir.is_dir():
        candidate_dirs = [
            d for d in changes_dir.iterdir() if d.is_dir() and d.name != "archive"
        ]
        # Sort by mtime descending
        candidate_dirs.sort(key=lambda d: d.stat().st_mtime, reverse=True)
        for c in candidate_dirs:
            t = c / "tasks.md"
            if t.is_file():
                return t

    return repo_root / "tasks.md"


def touches_code(tasks_file: Path) -> bool:
    """Determine whether the change touches code (src, tests, benches, etc.)."""
    repo_root = find_repo_root()

    # 1. Check git diff against HEAD / origin/main if available
    try:
        diff_files = subprocess.check_output(
            ["git", "status", "--porcelain=v1"],
            cwd=str(repo_root),
            stderr=subprocess.DEVNULL,
            text=True,
        )
        code_exts = {".rs", ".py", ".sh", ".ps1", ".js", ".ts", ".go", ".c", ".cpp"}
        code_dirs = {"src/", "tests/", "benches/", "scripts/"}
        for line in diff_files.splitlines():
            if len(line) > 3:
                filepath = line[3:].strip()
                if any(filepath.startswith(d) for d in code_dirs) or any(
                    filepath.endswith(ext) for ext in code_exts
                ):
                    return True
    except Exception:
        pass

    # 2. Check siblings proposal.md or design.md in same directory
    parent_dir = tasks_file.parent
    for sibling_name in ["proposal.md", "design.md", "spec.md"]:
        sibling = parent_dir / sibling_name
        if sibling.is_file():
            try:
                content = sibling.read_text(encoding="utf-8")
                if re.search(r"src/|tests/|benches/|\.rs\b|\.py\b|\.js\b", content):
                    return True
            except Exception:
                continue

    return False


def analyze_tail(content: str) -> dict:
    """Analyze whether required lifecycle elements are present in content."""
    has_simplify = bool(
        re.search(r"\b(simplify|ce-simplify-code)\b", content, re.IGNORECASE)
    )
    has_review = bool(
        re.search(r"\b(review|ce-code-review|review-receipt)\b", content, re.IGNORECASE)
    )
    has_compound = bool(
        re.search(
            r"\b(compound|ce-compound|docs/solutions|CONCEPTS\.md)\b",
            content,
            re.IGNORECASE,
        )
    )

    missing = []
    if not has_simplify:
        missing.append("simplification (`ce-simplify-code`)")
    if not has_review:
        missing.append("code review (`ce-code-review` / `review-receipt`)")
    if not has_compound:
        missing.append("knowledge capture (`ce-compound` in `docs/solutions/` + `CONCEPTS.md`)")

    return {
        "has_simplify": has_simplify,
        "has_review": has_review,
        "has_compound": has_compound,
        "missing": missing,
        "is_valid": len(missing) == 0,
    }


def main():
    parser = argparse.ArgumentParser(
        description="Validate presence of lifecycle tail in OpenSpec tasks.md"
    )
    parser.add_argument(
        "path",
        nargs="?",
        help="Path to tasks.md or directory containing tasks.md (defaults to active change)",
    )
    parser.add_argument(
        "--fix",
        action="store_true",
        help="Append standardized lifecycle tail if missing",
    )
    parser.add_argument(
        "--check-code",
        action="store_true",
        help="Enforce lifecycle tail regardless of heuristics",
    )
    parser.add_argument(
        "--allow-exempt",
        action="store_true",
        default=True,
        help="Allow documentation-only changes to omit lifecycle tail (default: True)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit output as JSON",
    )

    args = parser.parse_args()

    tasks_path = resolve_tasks_path(args.path)

    if not tasks_path.is_file():
        err_msg = f"tasks.md not found at {tasks_path}"
        if args.json:
            print(json.dumps({"status": "error", "message": err_msg}))
        else:
            print(f"error: {err_msg}", file=sys.stderr)
        return 2

    try:
        content = tasks_path.read_text(encoding="utf-8")
    except Exception as e:
        err_msg = f"Failed to read {tasks_path}: {e}"
        if args.json:
            print(json.dumps({"status": "error", "message": err_msg}))
        else:
            print(f"error: {err_msg}", file=sys.stderr)
        return 2

    is_code = args.check_code or touches_code(tasks_path)

    if not is_code and args.allow_exempt:
        if args.json:
            print(
                json.dumps(
                    {
                        "status": "exempt",
                        "path": str(tasks_path),
                        "message": "Change does not modify code; lifecycle tail is optional.",
                    }
                )
            )
        else:
            print(
                f"tasks-tail: {tasks_path.name} is exempt (documentation/config change, no code touched)"
            )
        return 0

    analysis = analyze_tail(content)

    if analysis["is_valid"]:
        if args.json:
            print(
                json.dumps(
                    {
                        "status": "valid",
                        "path": str(tasks_path),
                        "missing": [],
                    }
                )
            )
        else:
            print(f"tasks-tail: {tasks_path.name} is valid — all lifecycle units present")
        return 0

    if args.fix:
        # Append lifecycle tail
        new_content = content.rstrip() + "\n" + LIFECYCLE_TAIL_TEMPLATE
        try:
            tasks_path.write_text(new_content, encoding="utf-8")
            if args.json:
                print(
                    json.dumps(
                        {
                            "status": "fixed",
                            "path": str(tasks_path),
                            "appended": analysis["missing"],
                        }
                    )
                )
            else:
                print(
                    f"tasks-tail: appended lifecycle tail to {tasks_path.name} successfully"
                )
            return 0
        except Exception as e:
            err_msg = f"Failed to write to {tasks_path}: {e}"
            if args.json:
                print(json.dumps({"status": "error", "message": err_msg}))
            else:
                print(f"error: {err_msg}", file=sys.stderr)
            return 2

    # Validation failure
    if args.json:
        print(
            json.dumps(
                {
                    "status": "invalid",
                    "path": str(tasks_path),
                    "missing": analysis["missing"],
                }
            )
        )
    else:
        print(
            f"tasks-tail: {tasks_path.name} is missing post-implementation lifecycle units:",
            file=sys.stderr,
        )
        for m in analysis["missing"]:
            print(f"  - Missing {m}", file=sys.stderr)
        print(
            f"\nRemediation: run 'python3 scripts/validate-tasks-tail.py {tasks_path} --fix' to append them.",
            file=sys.stderr,
        )
    return 1


if __name__ == "__main__":
    sys.exit(main())
