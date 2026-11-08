"""Keep mobile publication out of the desktop fork's release routes."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class MobilePublication(unittest.TestCase):
    def test_testflight_upload_has_no_entrypoint(self):
        workflow = (ROOT / ".github/workflows/ios.yml").read_text()
        recipes = (ROOT / "justfile").read_text()

        self.assertIn("  check:\n", workflow)
        self.assertIn("CODE_SIGNING_ALLOWED=NO", workflow)
        self.assertNotIn("  testflight:\n", workflow)
        self.assertNotIn("ios-testflight:", recipes)
        for name in (
            "app_store_connect.py",
            "next-build.py",
            "testflight.sh",
            "testflight_notes.py",
        ):
            with self.subTest(name=name):
                self.assertFalse((ROOT / "ios/tools" / name).exists())


if __name__ == "__main__":
    unittest.main()
