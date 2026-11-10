"""ABOUTME: Check the installer's smoke policy for externally owned URL schemes.
ABOUTME: Keep absent registrations absent and preserve existing link handlers.
"""
import importlib.util
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("windows_installer_links", ROOT / "scripts/release/windows-installer.py")
INSTALLER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(INSTALLER)


class InstallerLinkOwnership(unittest.TestCase):
    def test_absent_handlers_and_existing_owners_are_preserved(self):
        for handlers in (
            {"spaceport": None, "sheprd": None, "herdr-gpui": None},
            {"spaceport": None, "sheprd": '"C:\\Upstream\\sheprd.exe" "%1"',
             "herdr-gpui": '"C:\\Older\\herdr-gpui.exe" "%1"'},
        ):
            with self.subTest(handlers=handlers):
                INSTALLER.verify_link_handlers(handlers, dict(handlers))

    def test_install_and_uninstall_must_not_claim_or_delete_handlers(self):
        before = {"spaceport": None, "sheprd": '"C:\\Upstream\\sheprd.exe" "%1"',
                  "herdr-gpui": None}
        for scheme, value in (("spaceport", '"C:\\Preview\\spaceport-launcher.exe" "%1"'),
                              ("herdr-gpui", '"C:\\Preview\\spaceport-launcher.exe" "%1"'),
                              ("sheprd", None),
                              ("sheprd", '"C:\\Preview\\spaceport-launcher.exe" "%1"')):
            with self.subTest(scheme=scheme, value=value), self.assertRaisesRegex(ValueError, scheme):
                INSTALLER.verify_link_handlers(before, {**before, scheme: value})

    def test_smoke_tracks_the_app_and_inherited_scheme_boundaries(self):
        source = (ROOT / "crates/sheprd-gpui/src/link.rs").read_text()
        scheme = re.search(r'const SCHEME: &str = "([^"]+)";', source)[1]
        self.assertIn(scheme, INSTALLER.LINK_SCHEMES)
        self.assertIn("sheprd", INSTALLER.LINK_SCHEMES)
        self.assertIn("herdr-gpui", INSTALLER.LINK_SCHEMES)
        setup = INSTALLER.SCRIPT.read_text()
        for scheme in INSTALLER.LINK_SCHEMES:
            with self.subTest(scheme=scheme):
                self.assertNotIn(f'Root: HKCU; Subkey: "Software\\Classes\\{scheme}"', setup)


if __name__ == "__main__":
    unittest.main()
