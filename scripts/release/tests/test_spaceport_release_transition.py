"""ABOUTME: Verify Spaceport's repository identity and restored Windows CI coverage.
ABOUTME: Exercise real workflow conditions and the aggregate shell gate offline.
"""
import importlib.util
import itertools
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

import test_ci_policy as policy

JOBS = policy.JOBS
condition_matches = policy.condition_matches

ROOT = Path(__file__).resolve().parents[3]
REPO = "2389-research/spaceport"
SPEC = importlib.util.spec_from_file_location("check_ci_transition", ROOT / "scripts/release/check-ci.py")
CI = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CI)


class SpaceportReleaseTransition(unittest.TestCase):
    def context(self, **changes):
        return policy.CiPolicy().context(**{
            "github.event_name": "push", "github.ref": "refs/heads/main",
            "github.actor": "harperreed", "github.triggering_actor": "harperreed",
            **changes,
        })

    def test_new_repository_runs_windows_jobs_with_performance_disabled(self):
        context = policy.CiPolicy().context(**{
            "github.repository": REPO, "github.event_name": "push",
            "github.ref": "refs/heads/main", "github.actor": "harperreed",
            "github.triggering_actor": "harperreed",
        })
        for job in ("zizmor", "commits", "checks", "checks-passed", "build"):
            self.assertTrue(condition_matches(job, context), job)
        for event in ("push", "pull_request", "workflow_dispatch"):
            for job in ("windows", "windows-native", "windows-package"):
                self.assertTrue(condition_matches(job, {**context, "github.event_name": event}), job)

    def aggregate(self, context):
        # Resolve the workflow's own env expressions before running its shell.
        # A separately supplied pause value could hide a scheduling mismatch.
        declarations = re.findall(r"^          (\w+): \$\{\{ (.+?) \}\}$",
                                  JOBS["checks-passed"], re.M)
        env = {name: str(policy.expression_value(expression, context)).lower()
               for name, expression in declarations}
        script = JOBS["checks-passed"].split("        run: |\n", 1)[1]
        script = "\n".join(line[10:] for line in script.splitlines() if line.startswith("          "))
        return subprocess.run(["bash", "-e", "-c", script], capture_output=True,
                              env={**os.environ, **env}).returncode == 0

    def test_aggregate_requires_the_jobs_the_real_workflow_schedules(self):
        for event, code, package, performance in itertools.product(
                ("push", "pull_request", "workflow_dispatch"), ("true", "false"),
                ("true", "false"), ("", "false", "true")):
            context = self.context(**{
                "github.event_name": event, "needs.commits.outputs.code": code,
                "needs.commits.outputs.windows_package": package,
                "vars.WINDOWS_ENABLED": performance,
            })
            results = {f"needs.{job}.result": "success" if condition_matches(job, context) else "skipped"
                       for job in ("commits", "checks", "windows", "windows-native",
                                   "windows-package", "build-macos")}
            context.update(results)
            with self.subTest(event=event, code=code, package=package, performance=performance):
                self.assertTrue(self.aggregate(context))
                for key, value in results.items():
                    alternatives = (("failure", "cancelled", "skipped", "") if value == "success"
                                    else ("success", "failure"))
                    for invalid in alternatives:
                        self.assertFalse(self.aggregate({**context, key: invalid}), (key, invalid))

    def test_native_windows_commands_select_all_required_fixtures(self):
        step = re.search(r"      - name: Native window, input, and live-daemon tests\n(.*?)"
                         r"(?=      - name:)", JOBS["windows-native"], re.S)[1]
        script = step.split("        run: |\n", 1)[1]
        script = "\n".join(line[10:] for line in script.splitlines())
        # Capture the actual shell's cargo arguments; do not run Windows binaries here.
        with tempfile.TemporaryDirectory() as directory:
            cargo = Path(directory) / "cargo"
            cargo.write_text('#!/bin/sh\nprintf "%s\\n" "$@"\n')
            cargo.chmod(0o700)
            for runner in ("windows-2025", "windows-11-arm"):
                result = subprocess.run(["bash", "-e", "-c", script], check=True, text=True,
                                        capture_output=True, env={**os.environ, "RUNNER": runner,
                                        "PATH": f"{directory}{os.pathsep}{os.environ['PATH']}"})
                args = result.stdout.splitlines()
                self.assertEqual(args[args.index("--test") + 1], "live_gui")
                harness = args[args.index("--") + 1:]
                self.assertIn("--ignored", harness)
                skipped = [harness[i + 1] for i, arg in enumerate(harness) if arg == "--skip"]
                self.assertEqual(skipped, ["native_gui_live"] if runner == "windows-11-arm" else [])
                options = {"--ignored", "--nocapture", "--test-threads=1", "--skip", *skipped}
                self.assertFalse([arg for arg in harness if arg not in options], "unexpected test filter")

    def test_only_windows_performance_uses_the_repository_pause(self):
        steps = re.split(r"^      - name: ", JOBS["windows-native"], flags=re.M)[1:]
        performance = next(step for step in steps if step.startswith("Release-mode performance budget\n"))
        condition = re.search(r"^        if: (.+)$", performance, re.M)
        self.assertIsNotNone(condition, "performance pause must be explicit")
        for value in ("", "false", "1", "true"):
            self.assertEqual(policy.expression_value(condition[1], self.context(**{
                "vars.WINDOWS_ENABLED": value})), value == "true")
        for step in steps:
            if step != performance:
                self.assertNotIn("vars.WINDOWS_ENABLED", step)
        self.assertIn("HERDR_PERF_P95_MS: '100'", performance)
        self.assertIn("--release", performance)
        self.assertIn("dense_terminal_sidebar_performance", performance)

    def test_release_gate_requires_restored_windows_checks_and_native_jobs(self):
        for runner in ("windows-2025", "windows-11-arm"):
            self.assertIn(f"Windows native GUI ({runner})", CI.REQUIRED_JOBS)
            for features in ("default", "all"):
                self.assertIn(f"Windows check ({runner}, {features})", CI.REQUIRED_JOBS)
        active = [{"name": name, "status": "completed", "conclusion": "success"}
                  for name in CI.REQUIRED_JOBS]
        self.assertFalse(CI.missing_jobs(active))
        for job in active:
            with self.subTest(missing=job["name"]):
                self.assertTrue(CI.missing_jobs([other for other in active if other != job]))

    def test_feed_and_release_workflow_use_transferred_repository(self):
        config = json.loads((ROOT / "scripts/release/sparkle.json").read_text())
        self.assertEqual(config["feed_url"],
                         f"https://github.com/{REPO}/releases/download/macos-preview-feed/appcast.xml")
        workflow = (ROOT / ".github/workflows/macos-preview.yml").read_text()
        self.assertIn(f"github.repository == '{REPO}'", workflow)
        self.assertIn(f"--repo {REPO} --actor harperreed", workflow)
        self.assertNotIn("harperreed/spaceport", workflow)
