"""ABOUTME: Ensures the retired Boxd provider has no desktop route.
ABOUTME: Checks source and package inputs without touching saved user data.
"""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[3]


class BoxdSourceAbsence(unittest.TestCase):
    def test_provider_has_no_desktop_or_remote_source_route(self):
        for crate in ("sheprd-client", "sheprd-remote", "sheprd-gpui"):
            root = ROOT / "crates" / crate
            source = (
                path
                for path in (root / "src").rglob("*.rs")
                if path.name != "tests.rs" and "tests" not in path.parts
            )
            for path in [root / "Cargo.toml", *source]:
                with self.subTest(path=path.relative_to(ROOT)):
                    self.assertIsNone(
                        re.search(r"(?<![a-z])boxd", path.read_text().lower()),
                        str(path),
                    )

    def test_provider_ui_assets_are_absent_but_core_onboarding_remains(self):
        self.assertFalse((ROOT / "assets/icons/cloud/boxd.svg").exists())
        self.assertFalse((ROOT / "crates/sheprd-gpui/locales/en-US/settings-cloud-boxd.ftl").exists())
        self.assertFalse((ROOT / "crates/sheprd-gpui/locales/en-US/error-remote-boxd.ftl").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/menu/devices/add_device/core.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/menu/devices/core_catalog.rs").exists())


if __name__ == "__main__":
    unittest.main()
