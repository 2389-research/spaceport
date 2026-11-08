"""ABOUTME: Ensures the retired OpenSandbox provider has no desktop route.
ABOUTME: Keeps shared relay and Core machine onboarding in the inventory.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class OpenSandboxSourceAbsence(unittest.TestCase):
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
                    self.assertFalse("opensandbox" in path.read_text().lower(), str(path))

    def test_provider_assets_and_claims_are_absent_but_shared_routes_remain(self):
        for name in ("settings-cloud-opensandbox", "error-remote-opensandbox"):
            self.assertFalse((ROOT / f"crates/sheprd-gpui/locales/en-US/{name}.ftl").exists())
        self.assertFalse((ROOT / "assets/icons/cloud/opensandbox.svg").exists())
        for path in (ROOT / "website/site/index.html", ROOT / "website/site/assets/features.js"):
            self.assertFalse("OpenSandbox" in path.read_text(), str(path))
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/menu/devices/add_device/core.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-remote/src/cloud/socket.rs").exists())


if __name__ == "__main__":
    unittest.main()
