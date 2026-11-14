"""CI eligibility and credential boundaries; GitHub enforces fork approval online."""

import ast
import itertools
import os
from pathlib import Path
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[3]
WORKFLOW = (ROOT / ".github/workflows/ci.yml").read_text()
SECTIONS = re.split(r"^  ([a-z-]+):\n", WORKFLOW.split("\njobs:\n", 1)[1], flags=re.M)
JOBS = dict(zip(SECTIONS[1::2], SECTIONS[2::2]))
PR_JOBS = tuple(job for job in JOBS if job != "build")
REPOSITORY = "2389-research/spaceport"
OWNER = "harperreed"
# The repositories this was forked from, and one stranger's fork: none may run a job.
OTHER_REPOSITORIES = ("sheprd-labs/sheprd", "penso/herdr-gpui", "larry0x/spaceport")
# Paused until native Windows performance is fixed; the repository variable
# WINDOWS_ENABLED turns these back on without editing the workflow.
WINDOWS_JOBS = ("windows", "windows-native", "windows-package")
# Jobs that never run on main, where they would hold the macOS runners.
PR_ONLY_JOBS = ("build-macos",)


def condition_matches(job, context, jobs=JOBS):
    # Interpret only the boolean/equality subset used by these job conditions.
    # Unsupported syntax fails rather than executing workflow text as Python.
    expression = re.search(r"^    if: >-\n((?:      .+\n)+)", jobs[job], re.M)[1]
    expression = " ".join(expression.split()).replace("&&", "and").replace("||", "or")

    def evaluate(node):
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return node.value
        if isinstance(node, ast.Attribute):
            return context[ast.unparse(node)]
        if isinstance(node, ast.BoolOp):
            values = [evaluate(value) for value in node.values]
            if isinstance(node.op, ast.And):
                return all(values)
            if isinstance(node.op, ast.Or):
                return any(values)
        if isinstance(node, ast.Compare) and len(node.ops) == 1:
            if isinstance(node.ops[0], ast.Eq):
                return evaluate(node.left) == evaluate(node.comparators[0])
            if isinstance(node.ops[0], ast.NotEq):
                return evaluate(node.left) != evaluate(node.comparators[0])
        if (isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                and node.func.id == "always" and not node.args and not node.keywords):
            return True
        raise AssertionError(f"Unsupported CI expression: {ast.dump(node)}")

    return evaluate(ast.parse(expression, mode="eval").body)


class CiPolicy(unittest.TestCase):
    def context(self, **overrides):
        return {
            "github.repository": REPOSITORY,
            "github.event_name": "pull_request",
            "github.ref": "refs/pull/456/merge",
            "github.actor": "larry0x",
            "github.triggering_actor": "larry0x",
            "github.event.pull_request.user.login": "larry0x",
            "github.event.pull_request.head.repo.full_name": "larry0x/spaceport",
            "needs.commits.outputs.code": "true",
            "needs.commits.outputs.windows_package": "true",
            "vars.WINDOWS_ENABLED": "",
            **overrides,
        }

    def runs(self, job, context):
        return condition_matches(job, context)

    def test_every_job_names_only_this_repository(self):
        # A guard naming another repository is skipped here and reports green.
        for name, job in JOBS.items():
            self.assertIn(f"github.repository == '{REPOSITORY}'", job, name)
            for other in OTHER_REPOSITORIES[:2]:
                self.assertNotIn(other, job, name)

    def test_fork_pr_can_run_after_github_approval_or_owner_rerun(self):
        # Approval does not rewrite actor/author/source repo. An owner rerun
        # changes triggering_actor only; neither case may skip the PR checks.
        for actor in ("larry0x", OWNER):
            context = self.context(**{"github.triggering_actor": actor})
            for job in PR_JOBS:
                with self.subTest(actor=actor, job=job):
                    self.assertEqual(self.runs(job, context), job not in WINDOWS_JOBS)
            self.assertFalse(self.runs("build", context))

    def test_owner_pr_still_runs_checks_but_not_release_build(self):
        context = self.context(**{
            "github.actor": OWNER, "github.triggering_actor": OWNER,
            "github.event.pull_request.user.login": OWNER,
            "github.event.pull_request.head.repo.full_name": REPOSITORY,
        })
        for job in PR_JOBS:
            self.assertEqual(self.runs(job, context), job not in WINDOWS_JOBS, job)
        self.assertFalse(self.runs("build", context))

    def test_windows_jobs_run_only_when_the_repository_variable_enables_them(self):
        self.assertTrue(set(WINDOWS_JOBS) <= set(JOBS))
        for value, enabled in (("true", True), ("", False), ("false", False), ("1", False)):
            context = self.context(**{"vars.WINDOWS_ENABLED": value})
            for job in WINDOWS_JOBS:
                with self.subTest(value=value, job=job):
                    self.assertEqual(self.runs(job, context), enabled)
        # The variable gates only the Windows jobs.
        context = self.context(**{"vars.WINDOWS_ENABLED": "true"})
        for job in PR_JOBS:
            self.assertTrue(self.runs(job, context), job)

    def test_non_pr_events_require_owner_main_and_expected_event(self):
        for event, ref, actor, triggering_actor in itertools.product(
                ("push", "workflow_dispatch", "pull_request_target", "workflow_run"),
                ("refs/heads/main", "refs/heads/topic", "refs/tags/v1"),
                ("penso", OWNER, "larry0x"), ("penso", OWNER, "larry0x")):
            context = self.context(**{
                "github.event_name": event, "github.ref": ref,
                "github.actor": actor, "github.triggering_actor": triggering_actor,
            })
            allowed = (event in ("push", "workflow_dispatch") and ref == "refs/heads/main"
                       and actor == OWNER and triggering_actor == OWNER)
            for job in JOBS:
                with self.subTest(job=job, context=context):
                    self.assertEqual(self.runs(job, context),
                                     allowed and job not in PR_ONLY_JOBS and job not in WINDOWS_JOBS)

    def test_other_repositories_cannot_run_jobs(self):
        for repository, event, actor in itertools.product(
                OTHER_REPOSITORIES, ("pull_request", "push", "workflow_dispatch"), ("penso", OWNER)):
            context = self.context(**{
                "github.repository": repository, "github.event_name": event,
                "github.ref": "refs/heads/main", "github.actor": actor,
                "github.triggering_actor": actor, "vars.WINDOWS_ENABLED": "true",
            })
            for job in JOBS:
                self.assertFalse(self.runs(job, context), (repository, job, event))

    def test_supply_chain_runs_for_spaceport_and_no_other_repository(self):
        # A skipped job reports green, so a guard naming another repository
        # turns cargo-deny and the advisory scan off without any signal.
        text = (ROOT / ".github/workflows/supply-chain.yml").read_text()
        sections = re.split(r"^  ([a-z-]+):\n", text.split("\njobs:\n", 1)[1], flags=re.M)
        jobs = dict(zip(sections[1::2], sections[2::2]))
        self.assertEqual(set(jobs), {"zizmor", "cargo-deny"})
        spaceport = REPOSITORY
        for repository, event, ref, actor in itertools.product(
                (REPOSITORY, *OTHER_REPOSITORIES),
                ("pull_request", "push", "workflow_dispatch", "schedule", "pull_request_target"),
                ("refs/heads/main", "refs/heads/topic"),
                ("harperreed", "penso", "larry0x")):
            context = self.context(**{
                "github.repository": repository, "github.event_name": event,
                "github.ref": ref, "github.actor": actor, "github.triggering_actor": actor,
            })
            # The scheduled scan has no meaningful actor: GitHub reports
            # whoever last edited the cron line.
            expected = repository == spaceport and (
                event == "pull_request"
                or (event == "schedule" and ref == "refs/heads/main")
                or (event in ("push", "workflow_dispatch") and ref == "refs/heads/main"
                    and actor == "harperreed"))
            for job in jobs:
                with self.subTest(job=job, context=context):
                    self.assertEqual(condition_matches(job, context, jobs), expected)

    def test_path_selection_only_skips_platform_jobs(self):
        for code in ("false", ""):
            context = self.context(**{"needs.commits.outputs.code": code})
            for job in PR_JOBS:
                self.assertEqual(condition_matches(job, context),
                                 job not in ("checks", "build-macos", *WINDOWS_JOBS),
                                 (job, code))

    def test_required_gate_rejects_failures_skips_and_missing_detection(self):
        # Run the actual shell gate, including the docs-only exception.
        script = JOBS["checks-passed"].split("        run: |\n", 1)[1]
        script = "\n".join(line[10:] for line in script.splitlines() if line.startswith("          "))

        def gate(code, checks, windows, native, commits, package_inputs="false", package="skipped",
                 event="push", macos=None):
            if macos is None:
                macos = "success" if code == "true" and event == "pull_request" else "skipped"
            return subprocess.run(
                ["bash", "-e", "-c", script], capture_output=True,
                env={**os.environ, "CODE": code, "RESULT": checks, "WINDOWS": windows,
                     "NATIVE": native, "COMMITS": commits, "WINDOWS_ENABLED": "true",
                     "PACKAGE_INPUTS": package_inputs, "PACKAGE": package,
                     "EVENT": event, "MACOS_RELEASE": macos},
            ).returncode == 0

        for code, checks, windows, commits in itertools.product(
                ("true", "false", ""), ("success", "failure", "cancelled", "skipped"),
                ("success", "failure", "cancelled", "skipped"), ("success", "failure", "skipped")):
            expected = commits == "success" and (
                (code == "true" and checks == windows == "success")
                or (code == "false" and checks == windows == "skipped"))
            # The native Windows job follows the same rule as the Windows check.
            self.assertEqual(gate(code, checks, windows, windows, commits), expected,
                             (code, checks, windows, commits))
        for native in ("failure", "cancelled", "skipped"):
            self.assertFalse(gate("true", "success", "success", native, "success"), native)
        self.assertFalse(gate("false", "skipped", "skipped", "success", "success"))
        # The Windows package job runs only when its inputs change, and must
        # then succeed; missing detection fails like the platform jobs'.
        for inputs, package in itertools.product(
                ("true", "false", ""), ("success", "failure", "cancelled", "skipped")):
            expected = ((inputs == "true" and package == "success")
                        or (inputs == "false" and package == "skipped"))
            self.assertEqual(gate("true", "success", "success", "success", "success", inputs, package),
                             expected, (inputs, package))
        # The macOS release builds run, and must pass, on pull requests with
        # code changes only; anywhere else they must have been skipped.
        for code, event, macos in itertools.product(
                ("true", "false"), ("pull_request", "push", "workflow_dispatch"),
                ("success", "failure", "cancelled", "skipped")):
            platform = "success" if code == "true" else "skipped"
            expected = macos == ("success" if code == "true" and event == "pull_request" else "skipped")
            self.assertEqual(gate(code, platform, platform, platform, "success", event=event, macos=macos),
                             expected, (code, event, macos))

    def test_ci_has_no_privileged_trigger_or_credentials(self):
        triggers = WORKFLOW.split("\npermissions:", 1)[0]
        self.assertEqual(re.findall(r"^  (\w+):", triggers, re.M),
                         ["push", "pull_request", "workflow_dispatch"])
        self.assertIn("\npermissions: {}\n", WORKFLOW)
        for name, job in JOBS.items():
            with self.subTest(job=name):
                permissions = re.search(r"^    permissions:([^\n]*\n(?:      [^\n]*\n)*)", job, re.M)[1]
                self.assertIn(permissions.strip(), ("{}", "contents: read"))
                for forbidden in ("secrets.", "secrets[", "secrets:", "environment:", "id-token:"):
                    self.assertNotIn(forbidden, job)
                if "actions/checkout@" in job:
                    self.assertIn("persist-credentials: false", job)
                if "Swatinem/rust-cache@" in job:
                    self.assertIn("save-if: ${{ github.ref == 'refs/heads/main' }}", job)
        # Keep runners a fixed hosted set; never accept a PR-controlled label.
        self.assertEqual(re.findall(r"^    runs-on: (.+)$", WORKFLOW, re.M), [
            "ubuntu-latest", "ubuntu-24.04", "${{ matrix.runner }}",
            "${{ matrix.runner }}", "${{ matrix.runner }}", "windows-2025",
            "ubuntu-24.04", "${{ matrix.runner }}", "macos-15",
        ])
        self.assertEqual(re.findall(r"^        runner: (.+)$", WORKFLOW, re.M), [
            "[macos-15, ubuntu-24.04, ubuntu-24.04-arm]",
            "[windows-2025, windows-11-arm]",
            "[windows-2025, windows-11-arm]",
        ])
        self.assertEqual(re.findall(r"^          - runner: (.+)$", WORKFLOW, re.M),
                         ["ubuntu-24.04", "ubuntu-24.04-arm"])
