"""ABOUTME: Verify bounded macOS preview notes from an explicit published base.
ABOUTME: Exercise validation separately from real git-cliff rendering of tagged history.
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from scripts.release.tests.test_changelog import MOCK_GIT_CLIFF, history


VERSION = "20261110.4"
COMMITS = (("feat: upstream excluded", "v20261001.1"),
           ("fix: include the failed preview change", "v20261110.3"),
           ("feat: ship the nebula rocket", "v20261110.4"))


class PreviewChangelogTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="spaceport-preview-notes-")
        self.addCleanup(self.temp.cleanup)
        self.work = Path(self.temp.name)
        self.out = self.work / "out"
        self.out.mkdir()
        tools = self.work / "bin"
        tools.mkdir()
        cliff = tools / "git-cliff"
        cliff.write_text(MOCK_GIT_CLIFF)
        cliff.chmod(0o755)
        self.log = self.work / "cliff-calls.jsonl"
        self.env = {"PATH": str(tools) + os.pathsep + os.environ["PATH"],
                    "HOME": str(self.work), "MOCK_LOG": str(self.log),
                    "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull}
        self.script = history(self.work, self.env, *COMMITS)
        self.repo = self.script.parents[2]
        self.base = self.git("rev-parse", "HEAD~2")

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.repo), *args],
                                       env=self.env, text=True, stderr=subprocess.DEVNULL).strip()

    def generate(self, base=None, extra=()):
        return subprocess.run(["bash", str(self.script), "--macos-preview", VERSION,
                               str(self.out), self.base if base is None else base, *extra],
                              env=self.env, capture_output=True, text=True, timeout=60)

    def test_preview_uses_one_explicit_history_render_without_windows_choice(self):
        result = self.generate()
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual(len(calls), 1, calls)
        call = calls[0]
        self.assertIn(self.base + "..HEAD", call)
        self.assertNotIn("--unreleased", call)
        self.assertEqual(call[call.index("--tag") + 1], "v" + VERSION)
        self.assertEqual(call[call.index("--ignore-tags") + 1], ".*")
        self.assertEqual(call[call.index("--strip") + 1], "header")
        notes = (self.out / "RELEASE_NOTES.md").read_text()
        changelog = (self.out / "CHANGELOG.md").read_text()
        self.assertIn("macOS 14.2+", notes)
        self.assertIn("arm64", notes)
        self.assertIn("signed/notarized", notes)
        self.assertIn("Source: " + self.git("rev-parse", "HEAD"), notes)
        self.assertTrue(notes.endswith(changelog), notes)
        for claim in ("universal", "Linux", "Windows", "update in the app", "Sheprd"):
            self.assertNotIn(claim, notes)

    def test_invalid_or_nonancestor_bases_refuse_before_rendering(self):
        current = self.git("rev-parse", "HEAD")
        self.git("checkout", "--orphan", "unrelated")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                 "commit", "--allow-empty", "-m", "feat: unrelated")
        unrelated = self.git("rev-parse", "HEAD")
        self.git("checkout", "--detach", current)
        for base in ("", "HEAD~2", "A" * 40, "a" * 39, "a" * 40, unrelated):
            with self.subTest(base=base):
                result = self.generate(base)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(self.log.exists())
                self.assertEqual(list(self.out.iterdir()), [])

    def test_preview_refuses_unexpected_arguments_and_existing_outputs(self):
        self.assertNotEqual(self.generate(extra=("v20261110.3",)).returncode, 0)
        self.assertFalse(self.log.exists())
        (self.out / "RELEASE_NOTES.md").write_text("preserve\n")
        self.assertNotEqual(self.generate().returncode, 0)
        self.assertEqual((self.out / "RELEASE_NOTES.md").read_text(), "preserve\n")
        self.assertFalse(self.log.exists())


@unittest.skipUnless(shutil.which("git-cliff"), "git-cliff is not installed")
class RealPreviewChangelogTests(unittest.TestCase):
    def test_filtered_build_history_keeps_the_version_without_fake_entries(self):
        with tempfile.TemporaryDirectory(prefix="spaceport-preview-housekeeping-") as temp:
            work = Path(temp)
            out = work / "out"
            out.mkdir()
            env = {"PATH": os.environ["PATH"], "HOME": temp}
            script = history(work, env, ("feat: published base", "v20261001.1"),
                             ("ci: rebuild the preview", "v" + VERSION))
            repo = script.parents[2]
            base = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD~1"], text=True).strip()
            result = subprocess.run(["bash", str(script), "--macos-preview", VERSION, str(out), base],
                                    env=env, capture_output=True, text=True, timeout=60)
            self.assertEqual(result.returncode, 0, result.stderr)
            notes = (out / "RELEASE_NOTES.md").read_text()
            changelog = (out / "CHANGELOG.md").read_text()
            self.assertIn("## [v" + VERSION + "]", changelog)
            self.assertNotIn("Published base", notes)
            self.assertNotIn("Rebuild the preview", notes)
            self.assertNotIn("\n- ", notes)
            self.assertTrue(notes.endswith(changelog), notes)

    def test_same_commit_rebuild_has_no_entries_even_at_the_root(self):
        for commits in ((("feat: published root", "v" + VERSION),), COMMITS):
            with self.subTest(commits=commits), tempfile.TemporaryDirectory(prefix="spaceport-preview-rebuild-") as temp:
                work = Path(temp)
                out = work / "out"
                out.mkdir()
                env = {"PATH": os.environ["PATH"], "HOME": temp}
                script = history(work, env, *commits)
                repo = script.parents[2]
                base = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip()
                result = subprocess.run(["bash", str(script), "--macos-preview", VERSION, str(out), base],
                                        env=env, capture_output=True, text=True, timeout=60)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual((out / "CHANGELOG.md").read_text(), "")
                notes = (out / "RELEASE_NOTES.md").read_text()
                self.assertIn("Source: " + base, notes)
                self.assertIn("signed/notarized", notes)
                self.assertEqual(notes.split("\n\n", 1)[1], "")

    def test_existing_failed_and_current_tags_do_not_split_the_published_range(self):
        with tempfile.TemporaryDirectory(prefix="spaceport-preview-history-") as temp:
            work = Path(temp)
            out = work / "out"
            out.mkdir()
            env = {"PATH": os.environ["PATH"], "HOME": temp}
            script = history(work, env, *COMMITS)
            repo = script.parents[2]
            base = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD~2"], text=True).strip()
            result = subprocess.run(["bash", str(script), "--macos-preview", VERSION, str(out), base],
                                    env=env, capture_output=True, text=True, timeout=60)
            self.assertEqual(result.returncode, 0, result.stderr)
            notes = (out / "RELEASE_NOTES.md").read_text()
            changelog = (out / "CHANGELOG.md").read_text()
            self.assertIn("## [v20261110.4]", notes)
            self.assertIn("### Fixed\n- Include the failed preview change", notes)
            self.assertIn("### Added\n- Ship the nebula rocket", notes)
            self.assertNotIn("Upstream excluded", notes)
            self.assertNotIn("## [v20261110.3]", notes)
            self.assertNotIn("All notable changes to Sheprd", notes)
            self.assertEqual(notes.split("\n\n", 1)[1], changelog)


if __name__ == "__main__":
    unittest.main()
