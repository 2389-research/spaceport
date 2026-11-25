"""ABOUTME: Keeps the removed Herdr 1 install modal's routes out of Spaceport.
ABOUTME: D70's Local card links to the Herdr website without restoring this modal.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class LegacyInstallOfferAbsenceTests(unittest.TestCase):
    def test_no_legacy_install_offer_route(self):
        sources = [
            "crates/spaceport-gpui/src/actions.rs",
            "crates/spaceport-gpui/src/core_terminals/fallback.rs",
            "crates/spaceport-gpui/src/endpoint/polling.rs",
            "crates/spaceport-gpui/src/menu/chrome.rs",
            "crates/spaceport-gpui/src/menu/chrome/keys.rs",
            "crates/spaceport-gpui/src/menu/page.rs",
            "crates/spaceport-gpui/src/menus.rs",
            "crates/spaceport-gpui/src/window.rs",
            "crates/spaceport-gpui/src/window/render.rs",
        ]
        text = "\n".join((ROOT / path).read_text() for path in sources)
        for route in (
            "ShowHerdrNotDetected",
            "show_install_modal",
            "needs_herdr_install",
            "install_warning_shown",
            "Page::Install",
            'id("menu-install")',
        ):
            self.assertFalse(route in text, f"{route} remains")
        locale = (ROOT / "crates/spaceport-i18n/locales/en-US/menu-chrome.ftl").read_text()
        self.assertNotIn("Install Herdr", locale)
        self.assertNotIn("menu-chrome-install-", locale)


if __name__ == "__main__":
    unittest.main()
