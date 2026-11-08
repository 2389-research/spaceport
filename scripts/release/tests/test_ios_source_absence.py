"""Keep the retired native phone app out of active build paths."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[3]
SHARED_FIXTURES = ROOT / "ios/SheprdRemoteCore/Tests/SheprdRemoteCoreTests/Fixtures"


class IosSourceAbsence(unittest.TestCase):
    def test_only_phone_server_fixtures_remain_under_ios(self):
        files = [path for path in (ROOT / "ios").rglob("*") if path.is_file()]
        source = [path for path in files if not path.is_relative_to(SHARED_FIXTURES)]
        self.assertFalse(source, f"{len(source)} iOS source files remain: {source[:3]}")
        self.assertTrue(all(path.suffix == ".json" for path in files))
        self.assertTrue((SHARED_FIXTURES / "v2/new.json").is_file())

    def test_ios_build_and_icon_hooks_are_absent(self):
        self.assertFalse((ROOT / ".github/workflows/ios.yml").exists())
        recipes = (ROOT / "justfile").read_text()
        self.assertIsNone(re.search(r"(?m)^ios-[\w-]+.*:", recipes))
        for name in ("generate-icons.swift", "check-icons.swift"):
            with self.subTest(name=name):
                self.assertNotIn('"ios/', (ROOT / "scripts" / name).read_text())


if __name__ == "__main__":
    unittest.main()
