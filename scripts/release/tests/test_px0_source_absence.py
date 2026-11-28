"""ABOUTME: Guards removal of the embedded px0 editor from desktop routes.
ABOUTME: Keeps shared tab, file-opening, and chat source owners visible.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class Px0SourceAbsence(unittest.TestCase):
    def test_no_live_editor_route_remains(self):
        for name in (
            "crates/spaceport-gpui/src/main.rs",
            "crates/spaceport-gpui/src/app.rs",
            "crates/spaceport-gpui/src/browser/view.rs",
            "crates/spaceport-gpui/src/cli/control.rs",
            "crates/spaceport-gpui/src/config.rs",
            "crates/spaceport-gpui/src/control/client.rs",
            "crates/spaceport-gpui/src/control/server/tabs.rs",
            "crates/spaceport-gpui/src/group_menu.rs",
            "crates/spaceport-gpui/src/menus.rs",
            "crates/spaceport-gpui/src/settings_window.rs",
            "crates/spaceport-gpui/src/settings_window/catalog.rs",
            "crates/spaceport-gpui/src/window/commands.rs",
            "crates/spaceport-commands/src/lib.rs",
            "crates/spaceport-control-protocol/src/protocol/tabs.rs",
        ):
            with self.subTest(path=name):
                self.assertNotIn("Px0", (ROOT / name).read_text(), name)
        for name in (
            "crates/spaceport-gpui/src/px0.rs",
            "crates/spaceport-gpui/src/browser/px0.rs",
            "crates/spaceport-gpui/src/browser/editor.rs",
        ):
            self.assertFalse((ROOT / name).exists(), name)

    def test_retained_file_chat_and_core_routes_exist(self):
        for name in (
            "crates/spaceport-gpui/src/window/file_links.rs",
            "crates/spaceport-gpui/src/core_terminals.rs",
        ):
            self.assertTrue((ROOT / name).exists(), name)
        # The browser store's home: the app before the browser-crate
        # extraction, the crate after it. Either must exist.
        self.assertTrue(
            (ROOT / "crates/spaceport-gpui/src/browser/store.rs").exists()
            or (ROOT / "crates/spaceport-browser/src/store.rs").exists(),
            "browser store",
        )


if __name__ == "__main__":
    unittest.main()
