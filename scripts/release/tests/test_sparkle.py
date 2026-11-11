"""ABOUTME: Test Sparkle SDK integrity, bundle policy and nested signing order.
ABOUTME: Fixtures exercise validation without signing credentials or network calls.
"""
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import tempfile
import subprocess
import unittest
from unittest.mock import patch

from sparkle_fixture import framework_fixture, sdk_fixture

SCRIPTS = Path(__file__).resolve().parents[1]


class SparkleTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.config = self.root / 'sparkle.json'
        self.config.write_text(json.dumps(dict(
            sdk_version='2.10.1',
            sdk_sha256='1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27',
            feed_url='https://github.com/harperreed/spaceport/releases/download/macos-preview-feed/appcast.xml',
            public_key='AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=')))
        spec = importlib.util.spec_from_file_location('sparkle_framework', SCRIPTS / 'sparkle_framework.py')
        self.assertTrue(Path(spec.origin).is_file(), 'Sparkle validation helper is missing')
        self.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.module)

    def test_acquire_rejects_tampered_archive_without_output(self):
        archive = self.root / 'bad.tar.xz'
        archive.write_bytes(b'tampered')
        output = self.root / 'target/sdk'
        result = subprocess.run(['python3', str(SCRIPTS / 'sparkle_framework.py'),
                                 'acquire', str(output), '--archive', str(archive),
                                 '--config', str(self.config)], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('checksum', result.stderr)
        self.assertFalse(output.exists())

    def test_sdk_license_must_be_regular_and_match_archive(self):
        scripts = self.root / 'helper'
        scripts.mkdir()
        sdk = self.root / 'sdk'
        sdk_fixture(scripts, sdk)
        spec = importlib.util.spec_from_file_location('license_fixture', scripts / 'sparkle_framework.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        license_path = sdk / 'LICENSE'
        original = license_path.read_bytes()
        for kind in ('tampered', 'missing', 'symlink'):
            with self.subTest(kind=kind):
                license_path.unlink(missing_ok=True)
                if kind == 'tampered':
                    license_path.write_bytes(original + b'tampered')
                elif kind == 'symlink':
                    license_path.symlink_to(sdk / 'Sparkle.framework/Versions/B/Resources/Info.plist')
                with self.assertRaisesRegex(ValueError, 'LICENSE'):
                    module.validate_sdk(sdk, scripts / 'sparkle.json')
                license_path.unlink(missing_ok=True)
                license_path.write_bytes(original)

    def test_missing_sdk_fails_closed(self):
        with self.assertRaisesRegex(ValueError, 'archive'):
            self.module.validate_sdk(self.root, self.config)

    def test_tampered_archive_fails_before_extraction(self):
        (self.root / 'Sparkle-2.10.1.tar.xz').write_bytes(b'tampered archive')
        with self.assertRaisesRegex(ValueError, 'checksum'):
            self.module.validate_sdk(self.root, self.config)

    def test_missing_and_invalid_public_key_fail_closed(self):
        for key in ('', 'not base64', 'YQ=='):
            data = json.loads(self.config.read_text())
            data['public_key'] = key
            self.config.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, 'public key'):
                self.module.read_config(self.config)

    def test_malformed_framework_is_rejected(self):
        framework = self.root / 'Sparkle.framework'
        framework.mkdir()
        with self.assertRaisesRegex(ValueError, 'framework'):
            self.module.validate_framework(framework, 'arm64')

    def test_framework_code_must_be_executable(self):
        framework = self.root / 'Sparkle.framework'
        framework_fixture(framework)
        (framework / 'Versions/B/Autoupdate').chmod(0o644)
        with patch.object(self.module.subprocess, 'check_output', return_value='arm64'):
            with self.assertRaisesRegex(ValueError, 'executable'):
                self.module.validate_framework(framework, 'arm64')

    def test_bundle_policy_requires_signed_feed_and_manual_installation(self):
        app = self.root / 'Spaceport.app'
        (app / 'Contents').mkdir(parents=True)
        info = app / 'Contents/Info.plist'
        info.write_bytes(plistlib.dumps({'CFBundleIdentifier': 'ai.2389.spaceport'}))
        self.module.configure_bundle(app, self.config)
        metadata = plistlib.loads(info.read_bytes())
        self.assertTrue(metadata['SURequireSignedFeed'])
        self.assertTrue(metadata['SUVerifyUpdateBeforeExtraction'])
        self.assertFalse(metadata['SUAutomaticallyUpdate'])
        self.assertNotIn('SUAllowsAutomaticUpdates', metadata)
        self.assertNotIn('SUEnableAutomaticChecks', metadata)
        self.module.validate_policy(app, self.config)
        metadata['SURequireSignedFeed'] = False
        info.write_bytes(plistlib.dumps(metadata))
        with self.assertRaisesRegex(ValueError, 'SURequireSignedFeed'):
            self.module.validate_policy(app, self.config)

    def test_signed_feed_failure_never_expires(self):
        app = self.root / 'Spaceport.app'
        (app / 'Contents').mkdir(parents=True)
        info = app / 'Contents/Info.plist'
        info.write_bytes(plistlib.dumps({'CFBundleIdentifier': 'ai.2389.spaceport'}))
        self.module.configure_bundle(app, self.config)
        metadata = plistlib.loads(info.read_bytes())
        self.assertEqual(metadata.get('SUSignedFeedFailureExpirationInterval'), 0)
        self.module.validate_policy(app, self.config)
        for invalid in (None, 1728000, -1, '0', False):
            with self.subTest(interval=invalid):
                if invalid is None:
                    metadata.pop('SUSignedFeedFailureExpirationInterval', None)
                else:
                    metadata['SUSignedFeedFailureExpirationInterval'] = invalid
                info.write_bytes(plistlib.dumps(metadata))
                with self.assertRaisesRegex(ValueError, 'SUSignedFeedFailureExpirationInterval'):
                    self.module.validate_policy(app, self.config)

    def test_development_bundle_cannot_be_configured(self):
        app = self.root / 'Spaceport.app'
        (app / 'Contents').mkdir(parents=True)
        (app / 'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier': 'spaceport.development'}))
        with self.assertRaisesRegex(ValueError, 'identity'):
            self.module.configure_bundle(app, self.config)


@unittest.skipUnless(os.environ.get('SPARKLE_TEST_SDK'), 'Requires explicitly selected real pinned Sparkle SDK')
class SparkleNativeSdkTests(unittest.TestCase):
    def test_real_acquisition_validation_and_tamper_rejection(self):
        sdk = Path(os.environ['SPARKLE_TEST_SDK']).resolve()
        with tempfile.TemporaryDirectory(prefix='spaceport-sdk-integration-') as temporary:
            output = Path(temporary) / 'sdk'
            command = ['python3', str(SCRIPTS / 'sparkle_framework.py')]
            result = subprocess.run([*command, 'acquire', str(output), '--archive',
                                     str(sdk / 'Sparkle-2.10.1.tar.xz')], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((output / 'Sparkle.framework/Versions/Current').is_symlink())
            self.assertEqual(os.readlink(output / 'Sparkle.framework/Sparkle'), 'Versions/Current/Sparkle')
            license_path = output / 'LICENSE'
            original_license = license_path.read_bytes()
            license_path.write_bytes(original_license + b'tampered')
            result = subprocess.run([*command, 'validate-sdk', str(output)], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('LICENSE', result.stderr)
            license_path.write_bytes(original_license)
            executable = output / 'Sparkle.framework/Versions/B/Autoupdate'
            executable.write_bytes(executable.read_bytes() + b'tampered')
            result = subprocess.run([*command, 'validate-sdk', str(output)], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('Tampered Sparkle SDK framework member', result.stderr)


if __name__ == '__main__':
    unittest.main()
