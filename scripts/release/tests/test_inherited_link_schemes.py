"""ABOUTME: Guards Windows package scripts from claiming inherited URL schemes.
ABOUTME: The development app already accepts only spaceport:// links.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class InheritedLinkSchemes(unittest.TestCase):
    def test_windows_installers_do_not_register_upstream_schemes(self):
        setup = (ROOT / "scripts/release/sheprd.iss").read_text()
        script = (ROOT / "site/install.ps1").read_text()
        for scheme in ("sheprd", "herdr-gpui"):
            with self.subTest(scheme=scheme):
                self.assertFalse(
                    any(f'Root: HKCU; Subkey: "Software\\Classes\\{scheme}' in line
                        for line in setup.splitlines()),
                    scheme,
                )
                self.assertFalse(
                    any(f"HKCU:\\Software\\Classes\\{scheme}'" in line
                        for line in script.splitlines()),
                    scheme,
                )


if __name__ == "__main__":
    unittest.main()
