"""ABOUTME: Checks that excluded native Teleport UI cannot start or render.
ABOUTME: Keeps Core machine catalog and existing exact-target follow routes visible.
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

    def test_retained_machine_and_follow_seams_remain(self):
        catalog = (ROOT / "crates/sheprd-gpui/src/core_terminals.rs").read_text()
        follow = (ROOT / "crates/sheprd-gpui/src/window/target_follow.rs").read_text()
        carry = (ROOT / "crates/sheprd-gpui/src/pane_menu/carry.rs").read_text()
        self.assertIn("fn machine_catalog", catalog)
        self.assertIn("endpoint_id", follow)
        self.assertIn("pane_id", follow)
        self.assertIn("Follow::pane", carry)


if __name__ == "__main__":
    unittest.main()
