"""ABOUTME: Exercise cold, warm and corrupted SDK preparation with pinned native bytes.
ABOUTME: Keep cached SDK validation real and preserve failed inputs for diagnosis.
"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / 'scripts/release/prepare-sparkle-sdk.sh'


class PreparationArguments(unittest.TestCase):
    def test_missing_directory_reports_usage(self):
        result = subprocess.run(['bash', str(SCRIPT)], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Usage:', result.stderr)


@unittest.skipUnless(sys.platform == 'darwin' and os.environ.get('SPARKLE_TEST_SDK'),
                     'requires the pinned native SDK; macOS CI supplies SPARKLE_TEST_SDK')
class NativeSdkPreparation(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='spaceport-sdk-cache-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.sdk = self.root / 'cached sdk'
        self.reference = Path(os.environ['SPARKLE_TEST_SDK'])
        self.archives = list(self.reference.glob('Sparkle-*.tar.xz'))
        self.assertEqual(len(self.archives), 1, 'explicit SDK must contain its pinned archive')

    def prepare(self, archive=False):
        command = ['bash', str(SCRIPT), str(self.sdk)]
        if archive:
            command.append(str(self.archives[0]))
        result = subprocess.run(command, capture_output=True, text=True, timeout=60)
        return result

    def acquire(self):
        result = self.prepare(archive=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str(self.sdk.resolve()))
        return self.sdk / self.archives[0].name

    def test_cold_archive_becomes_a_valid_sdk(self):
        archive = self.acquire()
        self.assertEqual(archive.read_bytes(), self.archives[0].read_bytes())
        self.assertTrue((self.sdk / 'Sparkle.framework/Versions/B/Sparkle').is_file())

    def test_warm_sdk_keeps_existing_files(self):
        archive = self.acquire()
        before = archive.stat()
        result = self.prepare()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str(self.sdk.resolve()))
        after = archive.stat()
        self.assertEqual((after.st_ino, after.st_mtime_ns), (before.st_ino, before.st_mtime_ns))

    def test_bad_archive_refuses_without_removing_the_cache(self):
        archive = self.acquire()
        archive.write_bytes(b'corrupt cached archive')
        result = self.prepare()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('checksum mismatch', result.stderr)
        self.assertEqual(archive.read_bytes(), b'corrupt cached archive')
        self.assertTrue((self.sdk / 'Sparkle.framework').is_dir())

    def test_changed_framework_refuses_without_repair(self):
        self.acquire()
        library = self.sdk / 'Sparkle.framework/Versions/B/Sparkle'
        with library.open('ab') as stream:
            stream.write(b'changed cache')
        result = self.prepare()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Sparkle', result.stderr)
        self.assertTrue(library.read_bytes().endswith(b'changed cache'))
