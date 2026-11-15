"""ABOUTME: Verify Spaceport's organization move and temporary Windows CI pause.
ABOUTME: Exercise real workflow conditions and the aggregate shell gate offline.
"""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
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
    def test_new_repository_runs_checks_but_pauses_every_windows_job(self):
        context = policy.CiPolicy().context(**{
            "github.repository": REPO, "github.event_name": "push",
            "github.ref": "refs/heads/main", "github.actor": "harperreed",
            "github.triggering_actor": "harperreed",
        })
        for job in ("zizmor", "commits", "checks", "checks-passed", "build"):
            self.assertTrue(condition_matches(job, context), job)
        for event in ("push", "pull_request", "workflow_dispatch"):
            for job in ("windows", "windows-native", "windows-package"):
                self.assertFalse(condition_matches(job, {**context, "github.event_name": event}), job)

    def test_paused_windows_gate_requires_skips_and_preserves_other_failures(self):
        script = JOBS["checks-passed"].split("        run: |\n", 1)[1]
        script = "\n".join(line[10:] for line in script.splitlines() if line.startswith("          "))
        env = {**os.environ, "CODE": "true", "RESULT": "success", "COMMITS": "success",
               "WINDOWS": "skipped", "NATIVE": "skipped", "PACKAGE": "skipped",
               "WINDOWS_ENABLED": "false", "PACKAGE_INPUTS": "true", "EVENT": "push",
               "MACOS_RELEASE": "skipped"}
        for changes, expected in (({}, True), ({"RESULT": "failure"}, False),
                                  ({"COMMITS": "failure"}, False), ({"CODE": ""}, False),
                                  ({"WINDOWS": "success"}, False), ({"NATIVE": "failure"}, False),
                                  ({"PACKAGE": "cancelled"}, False),
                                  ({"EVENT": "pull_request", "MACOS_RELEASE": "failure"}, False)):
            with self.subTest(changes=changes):
                result = subprocess.run(["bash", "-e", "-c", script], capture_output=True,
                                        env={**env, **changes})
                self.assertEqual(result.returncode == 0, expected, result.stderr.decode())

    def test_release_gate_requires_all_remaining_platform_jobs(self):
        # The paused Windows jobs are not part of the gate at all.
        self.assertFalse([name for name in CI.REQUIRED_JOBS if name.startswith("Windows ")])
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
