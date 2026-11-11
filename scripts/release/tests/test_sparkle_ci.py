"""ABOUTME: Protect Sparkle preview signing and publication ordering in CI.
ABOUTME: Keep the private update key out of builds and publish only verified bytes.
"""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]


class SparkleCiTests(unittest.TestCase):
    def test_preview_signs_feed_after_final_image_verification(self):
        source = (ROOT / ".github/workflows/macos-preview.yml").read_text()
        build, rest = source.split("\n  build:", 1)[1].split("\n  sign:", 1)
        signing, publishing = rest.split("\n  publish:", 1)
        self.assertIn("sparkle_framework.py acquire", build)
        self.assertNotIn("SPARKLE_PRIVATE_KEY", build)
        self.assertIn("environment: macos-preview", signing)
        self.assertLess(signing.index("verify-macos-preview.sh"), signing.index("sparkle-feed.py create"))
        self.assertIn("SPARKLE_PRIVATE_KEY: ${{ secrets.SPARKLE_PRIVATE_KEY }}", signing)
        self.assertNotIn("SPARKLE_PRIVATE_KEY", publishing)
        self.assertLess(publishing.index("gh release create"),
                        publishing.index("publish-sparkle-feed.py"))

    def test_preview_build_explicitly_selects_optional_feature_and_excludes_key(self):
        source = (ROOT / "scripts/release/build-macos.sh").read_text()
        self.assertIn("sheprd-gpui/macos-updater", source)
        self.assertIn("-u SPARKLE_PRIVATE_KEY", source)
        self.assertIn("SPARKLE_FRAMEWORK_PATH", source)

    def test_macos_all_features_ci_has_framework_for_linking_and_raw_cli(self):
        source = (ROOT / ".github/workflows/ci.yml").read_text()
        checks = source.split("\n  checks:", 1)[1].split("\n  windows:", 1)[0]
        self.assertIn("sparkle_framework.py acquire", checks)
        self.assertIn("DYLD_FRAMEWORK_PATH", checks)
        self.assertIn("SPARKLE_FRAMEWORK_PATH", checks)
        self.assertLess(checks.index("sparkle_framework.py acquire"),
                        checks.index("cargo clippy"))
