"""ABOUTME: Checks that excluded native Teleport UI cannot start or render.
ABOUTME: Keeps Core machine catalog and exact group window routes visible.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class TeleportNativeAbsence(unittest.TestCase):
    def test_native_migration_ui_and_marks_are_absent(self):
        for relative in (
            "crates/sheprd-gpui/src/menu/teleport.rs",
            "crates/sheprd-gpui/src/teleport/ui.rs",
            "crates/sheprd-remote/src/teleport/marks.rs",
        ):
            self.assertFalse((ROOT / relative).exists(), relative)
        for relative, excluded in (
            ("crates/sheprd-gpui/src/menu/workspace.rs", "WorkspaceMenuAction::Teleport"),
            ("crates/sheprd-gpui/src/menu/workspace.rs", "WorkspaceMenuAction::TeleportBack"),
            ("crates/sheprd-gpui/src/menu/workspace.rs", "WorkspaceMenuAction::GoToTeleported"),
            ("crates/sheprd-gpui/src/menu/workspace.rs", "WorkspaceMenuAction::ClearTeleported"),
            ("crates/sheprd-gpui/src/menu/chrome.rs", "Page::Teleport"),
            ("crates/sheprd-gpui/src/window/construct.rs", "Marks::start()"),
            ("crates/sheprd-gpui/src/window.rs", "self.poll_teleport("),
            ("crates/sheprd-gpui/src/sidebar/workspaces.rs", "crate::teleport::Marks"),
        ):
            self.assertNotIn(excluded, (ROOT / relative).read_text(), relative)
        for source in (ROOT / "crates/sheprd-gpui/src").rglob("*.rs"):
            self.assertNotIn("teleported.json", source.read_text(), source)

    def test_retained_machine_and_exact_group_window_seams_remain(self):
        catalog = (ROOT / "crates/sheprd-gpui/src/core_terminals.rs").read_text()
        launch = (ROOT / "crates/sheprd-gpui/src/app.rs").read_text()
        self.assertIn("fn machine_catalog", catalog)
        self.assertIn("WindowLaunch::Core(key)", launch)
        self.assertIn("core_group_workspace(key, cx)", launch)
        for relative in ("window/target_follow.rs", "pane_menu/carry.rs"):
            self.assertFalse((ROOT / "crates/sheprd-gpui/src" / relative).exists())


if __name__ == "__main__":
    unittest.main()
