"""ABOUTME: Guards against automatic deletion of merged worktree checkouts.
ABOUTME: Keeps the explicit delete dialog and its target checks reachable.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
GUI = ROOT / "crates/sheprd-gpui"


class MergedCleanupAbsence(unittest.TestCase):
    def test_no_automatic_removal_route_or_setting(self):
        self.assertFalse((GUI / "src/merged_cleanup.rs").exists())
        for relative, marker in (
            ("src/window.rs", "update_merged_cleanup"),
            ("src/connection.rs", "request_cleanup"),
            ("src/state.rs", "cleanup_response"),
            ("src/config.rs", "remove_merged_worktrees"),
            ("src/menu/worktree_shortcut.rs", "ask_merged_removal"),
            ("src/settings_window/pages/general.rs", "general-merged-worktrees"),
        ):
            with self.subTest(relative=relative):
                self.assertNotIn(marker, (GUI / relative).read_text())

    def test_explicit_removal_still_requires_the_dialog_route(self):
        workspace = (GUI / "src/menu/workspace.rs").read_text()
        requests = (GUI / "src/menu/workspace/requests.rs").read_text()
        self.assertIn("fn open_workspace_dialog(", workspace)
        self.assertIn("Method::WorktreeRemove", workspace)
        self.assertIn("fn submit_workspace_dialog(", requests)


if __name__ == "__main__":
    unittest.main()
