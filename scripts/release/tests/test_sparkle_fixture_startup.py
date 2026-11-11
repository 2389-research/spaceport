# ABOUTME: Verify signed fixture startup and isolation through real macOS processes.
# ABOUTME: Run explicitly with --host and --sdk; no update or native UI is opened.
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
import uuid

HOST = SDK = None
SOURCE = Path(__file__).resolve().parents[1] / 'test-sparkle-native.py'


@unittest.skipUnless(sys.platform == 'darwin', 'Requires macOS LaunchServices')
class FixtureStartupTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if HOST is None or SDK is None:
            raise unittest.SkipTest('Run explicitly with --host and --sdk')
        spec = importlib.util.spec_from_file_location('native_fixture', SOURCE)
        cls.native = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.native)
        evidence = SOURCE.parents[2] / 'target/sparkle-native'
        evidence.mkdir(parents=True, exist_ok=True)
        cls.root = Path(tempfile.mkdtemp(prefix='startup-', dir=evidence)).resolve()
        cls.root.chmod(0o700)
        cls.environment = cls.native.fixture_environment(cls.root)
        cls.app = cls.root / 'installed/Spaceport Fixture.app'
        domain = 'ai.2389.spaceport.fixture.' + uuid.uuid4().hex
        cls.native.bundle(cls.root, cls.app, HOST, SDK, cls.native.IDENTITY, '1', domain,
                          'http://127.0.0.1:9/appcast.xml', 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=',
                          cls.environment, 'information')
        cls.manifest = dict(bundle_id=domain, payload_id=domain)
        cls.native.claim_preferences(cls.root, cls.manifest)
        cls.safe_to_cleanup = True
        print(f'Startup-only evidence: {cls.root}', flush=True)

    @classmethod
    def tearDownClass(cls):
        if not cls.safe_to_cleanup:
            raise RuntimeError(f'Fixture exit unconfirmed; preferences and evidence retained: {cls.root}')
        cls.native.cleanup_preferences(cls.root, cls.manifest)

    def test_direct_start_restores_private_home_before_isolation(self):
        environment = dict(self.environment, HOME=str(self.root))
        executable = self.app / 'Contents/MacOS/sparkle_fixture'
        with subprocess.Popen([executable, '--check-isolation'], env=environment,
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True) as child:
            try:
                stdout, stderr = child.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                child.kill()  # Exact Popen child only; never a PID read from evidence.
                child.wait(timeout=5)
                raise
        (self.root / 'direct.stderr').write_text(stderr)
        self.assertEqual(child.returncode, 0, stderr)
        marker = json.loads(stdout)
        self.assertEqual(marker.get('pid'), child.pid, 'environment restoration must preserve the launched PID')
        self.assertEqual(marker['home'], self.environment['HOME'])
        self.assertEqual(marker['status'], 'isolated')
        self.assertFalse((self.root / 'launches').exists())

    def test_private_directory_permissions_remain_required(self):
        directory = self.root / 'cache'
        directory.chmod(0o755)
        try:
            self.assert_startup_refused(dict(self.environment), 'private owner/mode')
        finally:
            directory.chmod(0o700)

    def test_environment_restart_is_bounded(self):
        environment = dict(self.environment, HOME=str(self.root), SPACEPORT_FIXTURE_REEXEC='1')
        self.assert_startup_refused(environment, 'single re-exec')

    def test_private_directory_symlink_cannot_escape_root(self):
        directory = self.root / 'config'
        saved = self.root / 'config-saved'
        directory.rename(saved)
        directory.symlink_to(self.root.parent, target_is_directory=True)
        try:
            self.assert_startup_refused(dict(self.environment), 'escapes signed root')
        finally:
            directory.unlink()
            saved.rename(directory)

    def assert_startup_refused(self, environment, message):
        executable = self.app / 'Contents/MacOS/sparkle_fixture'
        result = subprocess.run([executable, '--check-isolation'], env=environment,
                                capture_output=True, text=True, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(message, result.stderr)
        self.assertEqual(result.stdout, '')
        self.assertFalse((self.root / 'launches').exists())

    def test_launch_services_restores_private_home_before_isolation(self):
        stdout, stderr = self.root / 'launchservices.stdout', self.root / 'launchservices.stderr'
        self.__class__.safe_to_cleanup = False
        result = subprocess.run(['/usr/bin/open', '-n', '-g', '--stdout', stdout,
                                 '--stderr', stderr, self.app, '--args', '--check-isolation'],
                                capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        # open -W can lose a fast exit before kevent registration (ESRCH).
        # Wait on this run's own diagnostic record and exact-path process exit.
        marker = None
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            try:
                marker = json.loads(stdout.read_text())
            except (FileNotFoundError, json.JSONDecodeError):
                time.sleep(0.05)
                continue
            if self.native.process_path(marker['pid']) != str(self.app / 'Contents/MacOS/sparkle_fixture'):
                self.__class__.safe_to_cleanup = True
                break
            time.sleep(0.05)
        self.assertTrue(self.safe_to_cleanup, 'Diagnostic process exit was not confirmed')
        self.assertEqual(stderr.read_text(), '')
        self.assertEqual(marker['home'], self.environment['HOME'])
        self.assertEqual(marker['status'], 'isolated')
        self.assertFalse((self.root / 'launches').exists())


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__ or 'Test signed fixture startup without opening UI')
    parser.add_argument('--host', required=True, type=Path, help='Compiled sparkle_fixture example')
    parser.add_argument('--sdk', required=True, type=Path, help='Verified pinned Sparkle SDK')
    args, remaining = parser.parse_known_args()
    HOST, SDK = args.host.resolve(), args.sdk.resolve()
    unittest.main(argv=[sys.argv[0], *remaining])
