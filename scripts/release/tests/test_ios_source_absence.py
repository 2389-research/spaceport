"""ABOUTME: Keeps retired phone source and fixtures out of the desktop fork.
ABOUTME: Checks mobile build hooks without touching installed services.
"""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[3]


class IosSourceAbsence(unittest.TestCase):
    def test_no_phone_source_or_fixtures_remain_under_ios(self):
        files = [path for path in (ROOT / "ios").rglob("*") if path.is_file()]
        self.assertFalse(files, f"{len(files)} iOS files remain: {files[:3]}")

    def test_ios_build_and_icon_hooks_are_absent(self):
        self.assertFalse((ROOT / ".github/workflows/ios.yml").exists())
        recipes = (ROOT / "justfile").read_text()
        self.assertIsNone(re.search(r"(?m)^ios-[\w-]+.*:", recipes))
        for name in ("generate-icons.swift", "check-icons.swift"):
            with self.subTest(name=name):
                self.assertNotIn('"ios/', (ROOT / "scripts" / name).read_text())


if __name__ == "__main__":
    unittest.main()
