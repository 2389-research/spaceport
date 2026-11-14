"""ABOUTME: Proves packages need no local Core binary as an input.
ABOUTME: Checks that released desktop artifacts do not contain Core.
"""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile


ROOT = Path(__file__).resolve().parents[3]
RELEASE = ROOT / "scripts/release"
VERSION = "20261108.1"


class ExternalCorePackages(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="spaceport-core-package-")
        self.addCleanup(temporary.cleanup)
        self.work = Path(temporary.name)
        self.binary = self.work / "sheprd"
        self.binary.write_bytes(
            b"not executable\0HERDR_BUILD_IDENTITY_V1\nworktree=0\nbranch=\npr=\n\0"
        )
        (self.work / "sheprd-acp").write_bytes(b"chat companion")
        self.notices = self.work / "notices.txt"
        self.notices.write_text("Third-party notice\n")

    def test_windows_archive_needs_no_core_and_contains_no_core(self):
        launcher = self.work / "spaceport-launcher.exe"
        launcher.write_bytes(b"launcher")
        binary = self.work / "spaceport.exe"
        binary.write_bytes(self.binary.read_bytes())
        (self.work / "sheprd-acp.exe").write_bytes(b"chat companion")
        result = subprocess.run(
            [sys.executable, str(RELEASE / "package-windows.py"), VERSION,
             "x86_64-pc-windows-msvc", str(binary), str(launcher),
             str(self.work), str(self.notices)],
            env={"PATH": os.environ["PATH"], "HOME": str(self.work)},
            capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        with zipfile.ZipFile(result.stdout.strip()) as archive:
            names = archive.namelist()
        self.assertFalse(any("herdr-core" in name.lower() for name in names))

    def test_builds_and_installer_do_not_fetch_or_install_core(self):
        for path in ("justfile", ".github/workflows/ci.yml",
                     ".github/workflows/macos-preview.yml", "scripts/release/build-macos.sh"):
            self.assertNotIn("fetch-herdr-core.py", (ROOT / path).read_text(), path)
        self.assertFalse((RELEASE / "fetch-herdr-core.py").exists())
        self.assertNotIn("herdr-core.exe", (RELEASE / "sheprd.iss").read_text())


if __name__ == "__main__":
    unittest.main()
