"""Reject the removed folder-open CLI and Explorer registration in shipped source."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]


class CliOpenAbsenceTests(unittest.TestCase):
    def test_installers_do_not_register_folder_open_verbs(self):
        for name in (
            "scripts/release/sheprd.iss",
            "site/install.ps1",
            "website/site/install.ps1",
            "website/herdrui/public/install.ps1",
        ):
            with self.subTest(name=name):
                source = (ROOT / name).read_text(encoding="utf-8")
                for kind in ('Directory\\shell\\HerdrUI', 'Directory\\Background\\shell\\HerdrUI'):
                    self.assertFalse(kind in source, f"{name} registers {kind}")

    def test_windows_smoke_does_not_require_the_removed_verb(self):
        source = (ROOT / "scripts/release/windows-installer.py").read_text(encoding="utf-8")
        self.assertFalse("folder_verbs" in source)
        self.assertFalse("FOLDER_VERBS" in source)


if __name__ == "__main__":
    unittest.main()
