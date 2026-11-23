"""ABOUTME: Guards removal of the daemon's plugin-capable link activation.
ABOUTME: Leaves the read-only link resolve and this client's own openers.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
GPUI = ROOT / "crates/spaceport-gpui/src"


class LinkActivationAbsence(unittest.TestCase):
    def test_the_client_cannot_send_link_activation(self):
        method = (ROOT / "crates/spaceport-client/src/method.rs").read_text()
        self.assertIn('PaneLinkResolve => "pane.link.resolve"', method)
        self.assertNotIn("pane.link.activate", method)

    def test_no_source_names_activation_or_a_plugin_link_handler(self):
        for source in sorted((ROOT / "crates").rglob("*.rs")):
            content = source.read_text()
            for name in ("PaneLinkActivate", "PluginLinkFailed", "link_handlers"):
                self.assertNotIn(name, content, str(source.relative_to(ROOT)))

    def test_read_only_link_owners_exist(self):
        # The resolve request, and the local readings a click opens from.
        for owner in ("links.rs", "terminal/links.rs", "window/file_links.rs", "window/links.rs"):
            self.assertTrue((GPUI / owner).exists(), owner)

    def test_a_click_opens_through_the_local_readings_only(self):
        window = (GPUI / "window/links.rs").read_text()
        # The click path is the activation entry, the typed target's mapping,
        # and the opener dispatch: one span, sliced from the window source.
        click = (
            window.split("fn activate_terminal_link", 1)[1].split("fn open_link", 1)[0]
            + window.split("fn click(self)", 1)[1].split("    fn link_cell_at", 1)[0]
        )
        # The click path names its own openers, and asks the daemon nothing.
        for opener in ("ClickTarget::Web", "ClickTarget::File", "ClickTarget::External"):
            self.assertIn(opener, click)
        self.assertFalse("request_link(" in click, "the click path asks the daemon")


if __name__ == "__main__":
    unittest.main()
