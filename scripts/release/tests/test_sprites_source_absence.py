"""ABOUTME: Ensures the retired Sprites provider has no desktop route.
ABOUTME: Keeps Azure relay and Core machine onboarding in the inventory.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class SpritesSourceAbsence(unittest.TestCase):
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
                    self.assertFalse("sprites" in path.read_text().lower(), str(path))

    def test_provider_assets_and_claims_are_absent_but_shared_routes_remain(self):
        for name in ("settings-cloud-sprites", "error-remote-sprites"):
            self.assertFalse((ROOT / f"crates/sheprd-gpui/locales/en-US/{name}.ftl").exists())
        self.assertFalse((ROOT / "assets/icons/cloud/sprites.svg").exists())
        self.assertFalse((ROOT / "crates/sheprd-remote/src/cloud/socket.rs").exists())
        for path in (ROOT / "website/site/index.html", ROOT / "website/site/assets/features.js"):
            self.assertFalse("Sprites" in path.read_text(), str(path))
        for path in (ROOT / "docs/sandbox.md", ROOT / "crates/sheprd-gpui/README.md"):
            self.assertFalse("sprite" in path.read_text().lower(), str(path))
        for path in (ROOT / "crates/sheprd-gpui/locales/en-US").glob("*.ftl"):
            with self.subTest(locale=path.name):
                self.assertNotIn("sprite", path.read_text().lower(), path.name)
        self.assertNotIn("crate::sprites::relay::run_helper", (ROOT / "crates/sheprd-gpui/src/app.rs").read_text())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/menu/devices/add_device/core.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-remote/src/cloud/relay.rs").exists())


if __name__ == "__main__":
    unittest.main()
