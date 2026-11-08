"""Keep mobile publication out of the desktop fork's release routes."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class MobilePublication(unittest.TestCase):
    def test_testflight_upload_has_no_entrypoint(self):
        recipes = (ROOT / "justfile").read_text()

        self.assertFalse((ROOT / ".github/workflows/ios.yml").exists())
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
