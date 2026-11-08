"""ABOUTME: Guards removal of the embedded px0 editor from desktop routes.
ABOUTME: Keeps shared tab, file-opening, and chat source owners visible.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class Px0SourceAbsence(unittest.TestCase):
    def test_no_live_editor_route_remains(self):
        for name in (
            "crates/sheprd-gpui/src/main.rs",
            "crates/sheprd-gpui/src/app.rs",
            "crates/sheprd-gpui/src/browser/view.rs",
            "crates/sheprd-gpui/src/cli/control.rs",
            "crates/sheprd-gpui/src/config.rs",
            "crates/sheprd-gpui/src/control/client.rs",
            "crates/sheprd-gpui/src/control/server/tabs.rs",
            "crates/sheprd-gpui/src/group_menu.rs",
            "crates/sheprd-gpui/src/menus.rs",
            "crates/sheprd-gpui/src/settings_window.rs",
            "crates/sheprd-gpui/src/settings_window/catalog.rs",
            "crates/sheprd-gpui/src/window/commands.rs",
            "crates/sheprd-commands/src/lib.rs",
            "crates/sheprd-control-protocol/src/protocol/tabs.rs",
        ):
            with self.subTest(path=name):
                self.assertNotIn("Px0", (ROOT / name).read_text(), name)
        for name in (
            "crates/sheprd-gpui/src/px0.rs",
            "crates/sheprd-gpui/src/browser/px0.rs",
            "crates/sheprd-gpui/src/browser/editor.rs",
        ):
            self.assertFalse((ROOT / name).exists(), name)

    def test_retained_file_chat_and_core_routes_exist(self):
        for name in (
            "crates/sheprd-gpui/src/window/file_links.rs",
            "crates/sheprd-gpui/src/browser/store.rs",
            "crates/sheprd-gpui/src/core_terminals.rs",
        ):
            self.assertTrue((ROOT / name).exists(), name)


if __name__ == "__main__":
    unittest.main()
