"""ABOUTME: Ensures the retired Boat provider has no desktop route.
ABOUTME: Keeps shared SSH and Core onboarding in the source inventory.
"""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[3]


class BoatSourceAbsence(unittest.TestCase):
    def test_provider_has_no_desktop_or_remote_source_route(self):
        for crate in ("sheprd-client", "sheprd-remote", "sheprd-gpui"):
            root = ROOT / "crates" / crate
            source = (
                path
                for path in (root / "src").rglob("*.rs")
                if "tests" not in path.parts and "tests" not in path.stem
            )
            for path in [root / "Cargo.toml", *source]:
                with self.subTest(path=path.relative_to(ROOT)):
                    self.assertIsNone(
                        re.search(r"\bboat\b", path.read_text().lower()), str(path)
                    )

    def test_provider_locale_and_site_claims_are_absent_but_shared_routes_remain(self):
        for name in ("settings-cloud-boat", "error-remote-boat"):
            self.assertFalse((ROOT / f"crates/sheprd-gpui/locales/en-US/{name}.ftl").exists())
        for path in (ROOT / "website/site/index.html", ROOT / "website/site/assets/features.js"):
            self.assertNotIn("Boat", path.read_text())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/menu/devices/add_device/core.rs").exists())
        ssh = (ROOT / "crates/sheprd-client/src/ssh.rs").read_text()
        self.assertIn("pub fn connect_command", ssh)


if __name__ == "__main__":
    unittest.main()
