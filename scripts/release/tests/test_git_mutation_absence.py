"""ABOUTME: Guards the desktop against native Git reader and writer routes.
ABOUTME: Keeps ordinary branch text while excluding Git workers and UI actions.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
VCS = ROOT / "crates/sheprd-vcs"
GUI = ROOT / "crates/sheprd-gpui/src"


class GitRouteAbsence(unittest.TestCase):
    def test_desktop_has_no_git_worker_or_popup(self):
        page = (GUI / "menu/page.rs").read_text()
        self.assertFalse(VCS.exists())
        self.assertFalse((GUI / "menu/git.rs").exists())
        self.assertNotIn("self.update_git()", (GUI / "window.rs").read_text())
        self.assertFalse("GitCommit," in page)
        self.assertFalse("    Git," in page)

    def test_branch_is_still_a_plain_sidebar_token(self):
        tokens = (GUI / "sidebar/tokens.rs").read_text()
        self.assertIn("SpaceToken::Branch => text(context.branch?", tokens)
        self.assertNotIn("SpaceToken::GitStatus", tokens)


if __name__ == "__main__":
    unittest.main()
