"""ABOUTME: Verify Spaceport's organization move and Windows performance pause.
ABOUTME: Exercise real workflow conditions and the aggregate shell gate offline.
"""
import importlib.util
import json
from pathlib import Path
import unittest

import test_ci_policy as policy

condition_matches = policy.condition_matches

ROOT = Path(__file__).resolve().parents[3]
REPO = "2389-research/spaceport"
SPEC = importlib.util.spec_from_file_location("check_ci_transition", ROOT / "scripts/release/check-ci.py")
CI = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CI)


class SpaceportReleaseTransition(unittest.TestCase):
    def test_new_repository_runs_windows_checks_with_performance_disabled(self):
        # A collaborator's push of main runs every job; the CI policy names no
        # login, so the same push from any of them qualifies.
        for actor in ("harperreed", "clintecker"):
            context = policy.CiPolicy().context(**{
                "github.repository": REPO, "github.event_name": "push",
                "github.ref": "refs/heads/main", "github.actor": actor,
                "github.triggering_actor": actor,
            })
            for job in ("zizmor", "commits", "checks", "checks-passed", "build"):
                self.assertTrue(condition_matches(job, context), (job, actor))
            for event in ("push", "pull_request", "workflow_dispatch"):
                for job in ("windows", "windows-native", "windows-package"):
                    self.assertTrue(condition_matches(job, {**context, "github.event_name": event}),
                                    (job, event, actor))

    def test_restored_windows_gate_requires_success_and_preserves_other_failures(self):
        context = policy.CiPolicy().context(**{
            "github.event_name": "push", "needs.commits.outputs.code": "true",
            "needs.checks.result": "success", "needs.commits.result": "success",
            "needs.windows.result": "success", "needs.windows-native.result": "success",
            "needs.windows-package.result": "success", "needs.build-macos.result": "skipped",
        })
        for changes, expected in (({}, True), ({"needs.checks.result": "failure"}, False),
                                  ({"needs.commits.result": "failure"}, False),
                                  ({"needs.commits.outputs.code": ""}, False),
                                  ({"needs.windows.result": "skipped"}, False),
                                  ({"needs.windows-native.result": "failure"}, False),
                                  ({"needs.windows-package.result": "cancelled"}, False),
                                  ({"github.event_name": "pull_request",
                                    "needs.build-macos.result": "failure"}, False)):
            with self.subTest(changes=changes):
                self.assertEqual(policy.aggregate_succeeds({**context, **changes}), expected)

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
        for channel, release in (("macos-preview", "macos-preview-feed"),
                                 ("production", "macos-stable-feed")):
            self.assertEqual(config["feeds"][channel]["release"], release)
            self.assertEqual(config["feeds"][channel]["url"],
                             f"https://github.com/{REPO}/releases/download/{release}/appcast.xml")
        workflow = (ROOT / ".github/workflows/macos-preview.yml").read_text()
        self.assertIn(f"github.repository == '{REPO}'", workflow)
        self.assertIn(f"--wait --repo {REPO}\n", workflow)
        self.assertNotIn("--actor", workflow)
        self.assertNotIn("harperreed/spaceport", workflow)
