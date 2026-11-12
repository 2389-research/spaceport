"""ABOUTME: Exercise executable staging with real host tools and cache reuse.
ABOUTME: Verify cache paths without assuming the installer's binary directory.
"""
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / 'scripts/release/cache-release-tools.sh'


class CachedTools(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='spaceport-tools-')
        self.addCleanup(self.temp.cleanup)
        self.cache = Path(self.temp.name) / 'cached tools'

    def stage(self, *tools):
        return subprocess.run(['/bin/bash', str(SCRIPT), str(self.cache), *tools],
                              capture_output=True, text=True)

    def test_installed_binary_runs_from_the_cache_directory(self):
        expected = subprocess.check_output(['jq', '--version'])
        result = self.stage('jq')
        self.assertEqual(result.returncode, 0, result.stderr)
        cached = self.cache / 'jq'
        self.assertEqual(cached.read_bytes(), Path(shutil.which('jq')).read_bytes())
        self.assertEqual(subprocess.check_output([str(cached), '--version']), expected)

    def test_warm_binary_keeps_its_inode_and_timestamp(self):
        self.assertEqual(self.stage('jq').returncode, 0)
        cached = self.cache / 'jq'
        before = cached.stat()
        result = self.stage('jq')
        self.assertEqual(result.returncode, 0, result.stderr)
        after = cached.stat()
        self.assertEqual((before.st_ino, before.st_mtime_ns), (after.st_ino, after.st_mtime_ns))

    def test_missing_tool_reports_failure_without_creating_a_binary(self):
        result = self.stage('spaceport-nonexistent-release-tool')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Release tool not found:', result.stderr)
        self.assertFalse((self.cache / 'spaceport-nonexistent-release-tool').exists())

    def test_missing_arguments_report_usage(self):
        result = subprocess.run(['/bin/bash', str(SCRIPT)], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Usage:', result.stderr)
