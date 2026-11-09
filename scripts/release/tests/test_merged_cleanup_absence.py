"""ABOUTME: Guards against automatic deletion of merged worktree checkouts.
ABOUTME: Keeps presentation-group deletion behind its explicit dialog.
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

    def test_group_deletion_keeps_the_dialog_without_checkout_removal(self):
        workspace = (GUI / "src/menu/workspace.rs").read_text()
        groups = (GUI / "src/core_terminals/workspace_ui.rs").read_text()
        self.assertNotIn("Method::WorktreeRemove", workspace)
        self.assertIn("fn open_core_workspace_delete(", groups)
        self.assertIn("fn submit_core_workspace_dialog(", groups)
        self.assertIn("Some(Page::CoreWorkspaceDelete)", groups)
        self.assertIn("edit::Edit::Delete { id: target.id }", groups)


if __name__ == "__main__":
    unittest.main()
