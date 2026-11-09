"""ABOUTME: Guards the desktop against native Git write actions.
ABOUTME: Keeps read-only checkout probes and workspace close safety reachable.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
VCS = ROOT / "crates/sheprd-vcs/src/git.rs"
GUI = ROOT / "crates/sheprd-gpui/src"


class GitMutationAbsence(unittest.TestCase):
    def test_popup_and_worker_have_no_write_route(self):
        worker = VCS.read_text()
        popup = (GUI / "menu/git.rs").read_text()
        page = (GUI / "menu/page.rs").read_text()
        for marker in ("pub enum Action", "Job::Run", "Completion::Action", "fn perform(", "fn commit(", "fn push(", "fn create_pull_request("):
            with self.subTest(marker=marker):
                self.assertFalse(marker in worker, marker)
        for marker in ("activate_git_row", "submit_git_commit", "render_git_menu", "render_git_commit"):
            with self.subTest(marker=marker):
                self.assertFalse(marker in popup, marker)
        self.assertFalse("GitCommit," in page)
        self.assertFalse("    Git," in page)

    def test_retained_read_routes_remain(self):
        worker = VCS.read_text()
        for marker in ("pub fn close_status(", "Job::Status", "Job::Dirty", "Job::Counts", "Job::Ahead"):
            with self.subTest(marker=marker):
                self.assertIn(marker, worker)
        popup = (GUI / "menu/git.rs").read_text()
        self.assertIn("fn update_git(", popup)
        self.assertIn("fn git_input(", popup)


if __name__ == "__main__":
    unittest.main()
