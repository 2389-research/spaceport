"""ABOUTME: Check the native Windows accessibility job's actual Cargo invocation.
ABOUTME: Qualify scheduling and fixture selection; hosted UI Automation proves behavior.
"""
import os
from pathlib import Path
import re
import subprocess
import tempfile
import textwrap
import unittest

import test_ci_policy as policy


def cargo_arguments(script):
    # Execute the real command; this recorder does not prove native behavior.
    with tempfile.TemporaryDirectory() as directory:
        cargo = Path(directory) / "cargo"
        cargo.write_text('#!/bin/sh\nprintf "%s\\n" "$@"\n')
        cargo.chmod(0o700)
        result = subprocess.run(
            ["bash", "-e", "-c", script], check=True, text=True,
            capture_output=True,
            env={**os.environ, "PATH": f"{directory}{os.pathsep}{os.environ['PATH']}"},
        )
    return result.stdout.splitlines()


def step_script(step):
    run = re.search(r"^        run: (.+)$", step, re.M)
    if run[1] == "|":
        return textwrap.dedent(step[run.end() + 1:])
    return run[1]


class WindowsAccessibilityCi(unittest.TestCase):
    def test_native_job_reports_the_actual_pipe_owner_boundary(self):
        step = re.search(
            r"      - name: Native pipe owner qualification\n(.*?)"
            r"(?=      - name:)", policy.JOBS["windows-native"], re.S,
        )
        self.assertIsNotNone(step, "native pipe owner qualification must run")
        self.assertIsNone(re.search(r"^        if:", step[1], re.M))
        args = cargo_arguments(step_script(step[1]))
        self.assertEqual(args[:4], ["test", "--locked", "-p", "sheprd-gpui"])
        self.assertEqual(args[args.index("--bin") + 1], "spaceport")
        self.assertEqual(args[args.index("--features") + 1], "integration-test")
        self.assertEqual(
            args[args.index("--") - 1],
            "daemon::windows_tests::daemon_pipe_compares_the_opened_owner_despite_read_write_access",
        )
        self.assertEqual(args[args.index("--") + 1:], ["--exact", "--nocapture", "--test-threads=1"])

    def test_native_x64_runs_every_ignored_accessibility_fixture(self):
        ordinary = self.accessibility_step("Native UI Automation tree and keyboard focus")
        args = cargo_arguments(step_script(ordinary))
        harness = args[args.index("--") + 1:]
        self.assertEqual(args[args.index("--") - 1], "accessibility_windows")
        self.assertEqual(
            harness, ["--ignored", "--nocapture", "--test-threads=1", "--skip", "core::"],
            "ordinary UIA may defer only the Core namespace",
        )
        core = self.accessibility_step("Native Core text and caret qualification", independent=True)
        args = cargo_arguments(step_script(core))
        self.assertEqual(args[args.index("--") - 1], "core::")
        self.assertEqual(
            args[args.index("--") + 1:], ["--ignored", "--nocapture", "--test-threads=1"],
            "the separate Core step must run every deferred fixture",
        )

    def accessibility_step(self, name, independent=False):
        step = re.search(
            rf"      - name: {re.escape(name)}\n(.*?)"
            r"(?=      - name:)", policy.JOBS["windows-native"], re.S,
        )
        self.assertIsNotNone(step, "native UI Automation must run in the desktop job")
        condition = re.search(r"^        if: (.+)$", step[1], re.M)
        self.assertIsNotNone(condition, "native UI Automation requires the supported daemon runner")
        for runner in ("windows-2025", "windows-11-arm"):
            for cancelled in (False, True) if independent else (False,):
                self.assertEqual(
                    policy.expression_value(condition[1], {
                        "matrix.runner": runner, "cancelled()": cancelled,
                    }),
                    runner == "windows-2025" and not cancelled,
                )
        args = cargo_arguments(step_script(step[1]))
        self.assertEqual(args[0], "test")
        self.assertIn("--locked", args)
        self.assertEqual(args[args.index("-p") + 1], "sheprd-gpui")
        self.assertEqual(args[args.index("--features") + 1], "integration-test")
        self.assertEqual(args[args.index("--test") + 1], "accessibility_windows")
        return step[1]
