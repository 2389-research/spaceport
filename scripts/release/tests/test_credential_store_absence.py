"""ABOUTME: Keeps the unused provider credential store out of the application.
ABOUTME: Protects the retained release policy and platform dependency owners.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class CredentialStoreAbsence(unittest.TestCase):
    def test_store_has_no_source_or_package_route(self):
        self.assertFalse((ROOT / "crates/sheprd-credentials").exists())
        for manifest in [ROOT / "Cargo.toml", *(ROOT / "crates").glob("*/Cargo.toml")]:
            with self.subTest(manifest=manifest.relative_to(ROOT)):
                self.assertNotIn("sheprd-credentials", manifest.read_text())
        self.assertNotIn('name = "sheprd-credentials"', (ROOT / "Cargo.lock").read_text())

    def test_store_has_no_error_logging_or_translation_route(self):
        source = ROOT / "crates/sheprd-gpui/src"
        for file in source.rglob("*.rs"):
            with self.subTest(source=file.relative_to(ROOT)):
                self.assertNotIn("sheprd_credentials", file.read_text())
        for file in (ROOT / "crates/sheprd-gpui/locales").rglob("*.ftl"):
            with self.subTest(locale=file.relative_to(ROOT)):
                self.assertNotIn("error-credentials-", file.read_text())


if __name__ == "__main__":
    unittest.main()
