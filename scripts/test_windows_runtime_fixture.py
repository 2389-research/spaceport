"""ABOUTME: Checks real archive staging and refusal of unsafe Windows ZIP members.
ABOUTME: Keeps live release download checks opt-in and never executes staged runtimes.
"""
import hashlib
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

SCRIPT = Path(__file__).with_name("install-herdr-windows.py")
SPEC = importlib.util.spec_from_file_location("runtime_fixture", SCRIPT)
runtime = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runtime)


class RuntimeFixtureTests(unittest.TestCase):
    @unittest.skipUnless(os.environ.get("SPACEPORT_TEST_RUNTIME_DOWNLOAD") == "1", "requires explicit release download")
    def test_core_cli_stages_the_real_pinned_archive_and_keeps_conpty(self):
        with tempfile.TemporaryDirectory(prefix="spaceport-core-package-") as root:
            destination = Path(root) / "fixture"
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "--core", str(destination)],
                text=True, capture_output=True, timeout=150,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(Path(result.stdout.strip()), (destination / "herdr-core.exe").resolve())
            for member in (
                "herdr-core.exe", "conpty/conpty.dll", "conpty/x64/OpenConsole.exe",
                "LICENSE", "THIRD-PARTY-NOTICES/Microsoft.Windows.Console.ConPTY-LICENSE.txt",
            ):
                self.assertTrue((destination / member).is_file(), member)

    @unittest.skipUnless(os.environ.get("SPACEPORT_TEST_RUNTIME_DOWNLOAD") == "1", "requires explicit release download")
    def test_default_cli_keeps_the_existing_herdr_fixture_contract(self):
        with tempfile.TemporaryDirectory(prefix="spaceport-herdr-package-") as root:
            destination = Path(root) / "fixture"
            result = subprocess.run(
                [sys.executable, str(SCRIPT), str(destination)],
                text=True, capture_output=True, timeout=150,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(Path(result.stdout.strip()), (destination / "herdr.exe").resolve())
            self.assertTrue((destination / "herdr.exe").is_file())

    def archive(self, root, member):
        archive = Path(root) / "fixture.zip"
        with zipfile.ZipFile(archive, "w") as zipped:
            zipped.writestr(member, b"archive validation input")
        return archive, hashlib.sha256(archive.read_bytes()).hexdigest()

    def test_windows_traversal_and_alternate_streams_are_refused_before_extraction(self):
        for member in ("../escape", "..\\escape", "/escape", "C:/escape", "safe:stream", "\\\\host\\share"):
            with self.subTest(member=member), tempfile.TemporaryDirectory() as root:
                archive, digest = self.archive(root, member)
                destination = Path(root) / "output"
                with self.assertRaisesRegex(ValueError, "unsafe archive member"):
                    runtime.unpack(archive, destination, digest, ("herdr-core.exe",))
                self.assertFalse(destination.exists())

    def test_missing_conpty_or_digest_mismatch_leaves_no_extracted_files(self):
        with tempfile.TemporaryDirectory() as root:
            archive, digest = self.archive(root, "herdr-core.exe")
            destination = Path(root) / "output"
            with self.assertRaisesRegex(ValueError, "digest mismatch"):
                runtime.unpack(archive, destination, "0" * 64, ("herdr-core.exe",))
            with self.assertRaisesRegex(ValueError, "missing required file"):
                runtime.unpack(archive, destination, digest, ("herdr-core.exe", "conpty/conpty.dll"))
            self.assertFalse(destination.exists())

    def test_an_existing_destination_is_preserved(self):
        with tempfile.TemporaryDirectory() as root:
            archive, digest = self.archive(root, "herdr-core.exe")
            destination = Path(root) / "output"
            destination.mkdir()
            saved = destination / "keep"
            saved.write_bytes(b"owned data")
            with self.assertRaisesRegex(ValueError, "not empty"):
                runtime.unpack(archive, destination, digest, ("herdr-core.exe",))
            self.assertEqual(saved.read_bytes(), b"owned data")


if __name__ == "__main__":
    unittest.main()
