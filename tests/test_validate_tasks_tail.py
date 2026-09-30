#!/usr/bin/env python3
"""Unit tests for scripts/validate-tasks-tail.py."""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve().parent.parent / "scripts" / "validate-tasks-tail.py"


class TestValidateTasksTail(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.dir_path = Path(self.temp_dir.name)

    def tearDown(self):
        self.temp_dir.cleanup()

    def run_validator(self, *args):
        cmd = [sys.executable, str(SCRIPT_PATH)] + list(args)
        proc = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        return proc.returncode, proc.stdout, proc.stderr

    def test_missing_tail_fails(self):
        tasks = self.dir_path / "tasks.md"
        tasks.write_text("# Tasks\n\n- [ ] **Unit 1:** Implement feature\n  - [ ] Write code\n")

        code, out, err = self.run_validator(str(tasks), "--check-code")
        self.assertEqual(code, 1)
        self.assertIn("missing post-implementation lifecycle units", err)
        self.assertIn("ce-simplify-code", err)
        self.assertIn("ce-code-review", err)
        self.assertIn("ce-compound", err)

    def test_valid_tail_passes(self):
        tasks = self.dir_path / "tasks.md"
        tasks.write_text(
            "# Tasks\n\n- [ ] **Unit 1:** Implement feature\n"
            "- [ ] **Unit 2:** ce-simplify-code refactoring\n"
            "- [ ] **Unit 3:** ce-code-review & review-receipt\n"
            "- [ ] **Unit 4:** ce-compound to docs/solutions and CONCEPTS.md\n"
        )

        code, out, err = self.run_validator(str(tasks), "--check-code")
        self.assertEqual(code, 0)
        self.assertIn("is valid — all lifecycle units present", out)

    def test_fix_appends_and_validates(self):
        tasks = self.dir_path / "tasks.md"
        tasks.write_text("# Tasks\n\n- [ ] **Unit 1:** Implement feature\n")

        # 1. Run with --fix
        code, out, err = self.run_validator(str(tasks), "--fix", "--check-code")
        self.assertEqual(code, 0)
        self.assertIn("appended lifecycle tail", out)

        # 2. Re-run without --fix -> must pass now
        code2, out2, err2 = self.run_validator(str(tasks), "--check-code")
        self.assertEqual(code2, 0)
        self.assertIn("is valid", out2)

    def test_fix_is_idempotent(self):
        tasks = self.dir_path / "tasks.md"
        tasks.write_text("# Tasks\n\n- [ ] **Unit 1:** Implement feature\n")

        # Fix first time
        self.run_validator(str(tasks), "--fix", "--check-code")
        content_first = tasks.read_text()

        # Fix second time
        code, out, err = self.run_validator(str(tasks), "--fix", "--check-code")
        self.assertEqual(code, 0)
        content_second = tasks.read_text()

        self.assertEqual(content_first, content_second)

    def test_json_output(self):
        tasks = self.dir_path / "tasks.md"
        tasks.write_text("# Tasks\n\n- [ ] **Unit 1:** Only coding\n")

        code, out, err = self.run_validator(str(tasks), "--check-code", "--json")
        self.assertEqual(code, 1)
        data = json.loads(out)
        self.assertEqual(data["status"], "invalid")
        self.assertEqual(len(data["missing"]), 3)


if __name__ == "__main__":
    unittest.main()
