"""ABOUTME: Check the native Windows accessibility job's actual Cargo invocation.
ABOUTME: Qualify scheduling and fixture selection; hosted UI Automation proves behavior.
"""
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

import test_ci_policy as policy


class WindowsAccessibilityCi(unittest.TestCase):
    def test_native_x64_runs_every_ignored_accessibility_fixture(self):
        step = re.search(
            r"      - name: Native UI Automation tree and keyboard focus\n(.*?)"
            r"(?=      - name:)", policy.JOBS["windows-native"], re.S,
        )
        self.assertIsNotNone(step, "native UI Automation must run in the desktop job")
        condition = re.search(r"^        if: (.+)$", step[1], re.M)
        self.assertIsNotNone(condition, "native UI Automation requires the supported daemon runner")
        for runner in ("windows-2025", "windows-11-arm"):
            self.assertEqual(
                policy.expression_value(condition[1], {"matrix.runner": runner}),
                runner == "windows-2025",
            )
        script = re.search(r"^        run: (.+)$", step[1], re.M)[1]
        # The shell executes the workflow's command; this recorder is not a native GUI test.
        with tempfile.TemporaryDirectory() as directory:
            cargo = Path(directory) / "cargo"
            cargo.write_text('#!/bin/sh\nprintf "%s\\n" "$@"\n')
            cargo.chmod(0o700)
            result = subprocess.run(
                ["bash", "-e", "-c", script], check=True, text=True,
                capture_output=True,
                env={**os.environ, "PATH": f"{directory}{os.pathsep}{os.environ['PATH']}"},
            )
        args = result.stdout.splitlines()
        self.assertEqual(args[0], "test")
        self.assertIn("--locked", args)
        self.assertEqual(args[args.index("-p") + 1], "sheprd-gpui")
        self.assertEqual(args[args.index("--features") + 1], "integration-test")
        self.assertEqual(args[args.index("--test") + 1], "accessibility_windows")
        harness = args[args.index("--") + 1:]
        self.assertIn("--ignored", harness)
        self.assertIn("--test-threads=1", harness)
        self.assertFalse(
            set(harness) - {"--ignored", "--nocapture", "--test-threads=1"},
            "a filter must not silently omit accessibility fixtures",
        )
