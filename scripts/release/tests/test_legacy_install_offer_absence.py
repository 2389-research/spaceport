"""ABOUTME: Keeps the Herdr 1 install offer out of Spaceport's desktop UI.
ABOUTME: Checks the menu, launch prompt, action, and English text together.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class LegacyInstallOfferAbsenceTests(unittest.TestCase):
    def test_no_legacy_install_offer_route(self):
        sources = [
            "crates/sheprd-gpui/src/actions.rs",
            "crates/sheprd-gpui/src/core_terminals/fallback.rs",
            "crates/sheprd-gpui/src/endpoint/polling.rs",
            "crates/sheprd-gpui/src/menu/chrome.rs",
            "crates/sheprd-gpui/src/menu/chrome/keys.rs",
            "crates/sheprd-gpui/src/menu/page.rs",
            "crates/sheprd-gpui/src/menus.rs",
            "crates/sheprd-gpui/src/window.rs",
            "crates/sheprd-gpui/src/window/render.rs",
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
        locale = (ROOT / "crates/sheprd-gpui/locales/en-US/menu-chrome.ftl").read_text()
        self.assertNotIn("Install Herdr", locale)
        self.assertNotIn("menu-chrome-install-", locale)


if __name__ == "__main__":
    unittest.main()
