"""ABOUTME: Load the pinned update framework from isolated Cargo test children.
ABOUTME: Verify staging preserves sandbox isolation and refuses conflicting files.
"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / 'scripts/release/sparkle_framework.py'


@unittest.skipUnless(sys.platform == 'darwin' and os.environ.get('SPARKLE_TEST_SDK'),
                     'requires the pinned native SDK; macOS CI supplies SPARKLE_TEST_SDK')
class TestRuntime(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='spaceport-test-runtime-')
        self.addCleanup(self.temp.cleanup)
        self.profile = Path(self.temp.name) / 'target/debug'
        (self.profile / 'deps').mkdir(parents=True)
        self.sdk = Path(os.environ['SPARKLE_TEST_SDK'])

    def stage(self):
        return subprocess.run([sys.executable, str(SCRIPT), 'stage-tests', str(self.profile),
                               '--sdk', str(self.sdk)], capture_output=True, text=True, timeout=60)

    def test_cargo_executables_load_with_an_empty_environment(self):
        for directory in [self.profile, self.profile / 'deps']:
            probe = directory / 'probe'
            built = subprocess.run([
                '/usr/bin/clang', '-x', 'objective-c', '-', '-o', str(probe),
                '-F', str(self.sdk), '-framework', 'Foundation', '-framework', 'Sparkle',
                '-Wl,-rpath,@executable_path/../Frameworks',
            ], input='#import <Sparkle/Sparkle.h>\nint main(void) {\n'
                     'return NSClassFromString(@"SPUStandardUpdaterController") ? 0 : 1;\n}\n',
                capture_output=True, text=True, timeout=60)
            self.assertEqual(built.returncode, 0, built.stderr)
        for _ in range(2):
            staged = self.stage()
            self.assertEqual(staged.returncode, 0, staged.stderr)
        for directory in [self.profile, self.profile / 'deps']:
            child = subprocess.run([str(directory / 'probe')], env={}, capture_output=True,
                                   text=True, timeout=30)
            self.assertEqual(child.returncode, 0, child.stderr)

    def test_existing_framework_is_preserved_and_reported(self):
        destination = self.profile / 'Frameworks/Sparkle.framework'
        destination.parent.mkdir()
        destination.write_bytes(b'other build output')
        result = self.stage()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('already exists', result.stderr)
        self.assertEqual(destination.read_bytes(), b'other build output')
