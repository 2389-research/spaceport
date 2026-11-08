"""ABOUTME: Ensures the retired Coder cloud provider has no desktop route.
ABOUTME: Keeps shared SSH and Core onboarding code in the source inventory.
"""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[3]


class CoderSourceAbsence(unittest.TestCase):
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
                        re.search(
                            r"(?<![a-z])coder",
                            path.read_text().lower().replace("coderabbit", ""),
                        ),
                        str(path),
                    )

    def test_provider_assets_are_absent_but_shared_routes_remain(self):
        self.assertFalse((ROOT / "assets/icons/cloud/coder.svg").exists())
        for name in ("devices-coder", "settings-cloud-coder", "error-remote-coder"):
            self.assertFalse((ROOT / f"crates/sheprd-gpui/locales/en-US/{name}.ftl").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/menu/devices/add_device/core.rs").exists())
        ssh = (ROOT / "crates/sheprd-client/src/ssh.rs").read_text()
        self.assertIn("pub fn connect_command", ssh)


if __name__ == "__main__":
    unittest.main()
