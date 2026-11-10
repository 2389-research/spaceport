"""ABOUTME: Isolated macOS preview signing contracts, never native Apple verification.
ABOUTME: Exercise real signer control flow with controlled Apple command stand-ins.
"""
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SCRIPTS = ROOT / 'scripts/release'
UUID = '12345678-1234-1234-1234-123456789abc'


class PreviewSigning(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='spaceport-sign-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.app = self.root / 'Spaceport.app'
        (self.app / 'Contents/MacOS').mkdir(parents=True)
        for binary in ('spaceport', 'sheprd-acp'):
            (self.app / 'Contents/MacOS' / binary).write_bytes(b'never execute\0SPACEPORT_PREVIEW_V1\nid=ai.2389.spaceport\nversion=20261109.1\nprofile=debug\n\0')
        self.plist = self.app / 'Contents/Info.plist'
        self.metadata = dict(CFBundleName='Spaceport', CFBundleDisplayName='Spaceport',
                             CFBundleIdentifier='ai.2389.spaceport', CFBundleExecutable='spaceport',
                             CFBundleVersion='20261109.1', CFBundleShortVersionString='20261109.1',
                             LSMinimumSystemVersion='14.2', SpaceportBuildProfile='debug')
        self.write_plist()
        tools = self.root / 'tools'
        tools.mkdir()
        mock = tools / 'mock.py'
        shutil.copyfile(SCRIPTS / 'tests/mock-tool.py', mock)
        mock.chmod(0o755)
        for tool in ('uname', 'security', 'openssl', 'base64', 'plutil', 'lipo', 'ditto', 'xcrun',
                     'hdiutil', 'codesign', 'spctl'):
            (tools / tool).symlink_to(mock)
        self.env = dict(PATH=str(tools) + os.pathsep + os.environ['PATH'], HOME=str(self.root),
                        MOCK_LOG=str(self.root / 'calls'), APPLE_RETRY_DELAY='0',
                        MACOS_CERTIFICATE_P12_BASE64='ZHVtbXk=', MACOS_CERTIFICATE_PASSWORD='dummy',
                        MACOS_SIGNING_IDENTITY='Developer ID Application: 2389 Research, Inc (HD9NM9NSMK)',
                        APPLE_API_PRIVATE_KEY='dummy key', APPLE_API_KEY_ID='dummy', APPLE_API_ISSUER_ID='dummy',
                        SPACEPORT_NOTARY_RECEIPTS=str(self.root / 'receipts'), MOCK_ARCHS='arm64')

    def write_plist(self):
        self.plist.write_bytes(plistlib.dumps(self.metadata))

    def calls(self):
        log = self.root / 'calls'
        return [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []

    def sign(self, success=False, mode='import'):
        result = subprocess.run(['/bin/bash', str(SCRIPTS / 'sign-macos.sh'), '20261109.1',
                                 str(self.app), str(self.root), 'arm64', mode],
                                env=self.env, capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode == 0, success, result.stderr)
        if success:
            output = self.root / 'Spaceport-20261109.1-arm64-apple-darwin.dmg'
            self.assertTrue(output.is_file(), result.stderr)
            self.assertEqual(result.stdout, str(output) + '\n')
        self.assertFalse(list(self.root.glob('.herdr-sign.*')))
        return result

    def test_success_keeps_receipts_and_restores_keychains(self):
        self.sign(success=True)
        self.assertTrue((self.root / 'Spaceport-20261109.1-arm64-apple-darwin.dmg').exists())
        calls = self.calls()
        submits = [c for c in calls if c[1:3] == ['notarytool', 'submit']]
        self.assertEqual(len(submits), 2)
        self.assertTrue(all('--wait' not in c for c in submits))
        self.assertEqual(len([c for c in calls if c[1:3] == ['notarytool', 'wait']]), 2)
        self.assertEqual(len(list((self.root / 'receipts').glob('*/*/submission.json'))), 2)
        self.assertTrue(all(UUID in p.read_text() for p in (self.root / 'receipts').glob('*/*/submission.json')))
        restores = [c for c in calls if c[:5] == ['security', 'list-keychains', '-d', 'user', '-s']]
        self.assertEqual(restores[-1][5:], ['/mock/login keychain-db', '/mock/system.keychain'])
        signs = [c for c in calls if c[:2] == ['codesign', '--force']]
        self.assertTrue(all('--timestamp' in c for c in signs))
        self.assertLess(next(i for i, c in enumerate(signs) if c[-1].endswith('/sheprd-acp')),
                        next(i for i, c in enumerate(signs) if c[-1].endswith('/Spaceport.app')))

    def test_unknown_submit_is_never_retried_and_blocks_same_artifact(self):
        self.env['MOCK_SUBMIT_UNKNOWN'] = '1'
        self.assertIn('outcome unknown', self.sign().stderr)
        self.assertEqual(len([c for c in self.calls() if c[1:3] == ['notarytool', 'submit']]), 1)
        self.assertTrue(list((self.root / 'receipts').glob('*/*/submit.stderr')))
        self.assertIn('receipt already exists', self.sign().stderr)
        self.assertEqual(len([c for c in self.calls() if c[1:3] == ['notarytool', 'submit']]), 1)

    def test_repeated_input_is_blocked_even_if_signed_output_would_change(self):
        self.env['MOCK_SUBMIT_UNKNOWN'] = '1'
        self.sign()
        before = len([c for c in self.calls() if c[0] == 'codesign'])
        self.sign()
        self.assertEqual(len([c for c in self.calls() if c[0] == 'codesign']), before)

    def test_pending_and_rejected_keep_uuid_and_diagnostics(self):
        for status in ('In Progress', 'Invalid'):
            with self.subTest(status=status):
                self.env['MOCK_NOTARY_STATUS'] = status
                self.env['SPACEPORT_NOTARY_RECEIPTS'] = str(self.root / status)
                self.sign()
                receipt = next((self.root / status).glob('*/*/submission.json'))
                self.assertEqual(json.loads(receipt.read_text())['id'], UUID)
                self.assertTrue((receipt.parent / 'status.json').exists())
                self.assertFalse((self.root / 'Spaceport-20261109.1-arm64-apple-darwin.dmg').exists())
                if status == 'Invalid':
                    self.assertTrue((receipt.parent / 'notary-log.json').exists())

    def test_app_and_team_and_architecture_fail_before_credentials(self):
        for field, value in (('CFBundleIdentifier', 'so.pen.herdr-gpui'), ('CFBundleExecutable', 'sheprd'),
                             ('CFBundleVersion', '20261109.2')):
            before = self.metadata[field]
            self.metadata[field] = value
            self.write_plist()
            self.sign()
            self.metadata[field] = before
        self.write_plist()
        self.env['MACOS_SIGNING_IDENTITY'] = 'Developer ID Application: Other (WRONGTEAM1)'
        self.sign()
        self.env['MACOS_SIGNING_IDENTITY'] = 'Developer ID Application: 2389 Research, Inc (HD9NM9NSMK)'
        self.env['MOCK_ARCHS'] = 'x86_64'
        self.sign()
        self.assertFalse(any(c[0] == 'security' for c in self.calls()))

    def test_malformed_submit_and_tool_failure_keep_original_app(self):
        for response in ('{}', 'not json', '{"id":"wrong"}', '{"id":"x"}\n{"id":"y"}'):
            self.env['SPACEPORT_NOTARY_RECEIPTS'] = str(self.root / str(len(self.calls())))
            self.env['MOCK_NOTARY_JSON'] = response
            self.assertIn('outcome unknown', self.sign().stderr)
        del self.env['MOCK_NOTARY_JSON']
        for tool in ('codesign', 'spctl', 'hdiutil', 'xcrun'):
            self.env['SPACEPORT_NOTARY_RECEIPTS'] = str(self.root / tool)
            self.env['MOCK_FAIL'] = tool
            self.sign()
        self.assertFalse((self.app / 'Contents/_CodeSignature').exists())
        self.assertFalse((self.app / 'Contents/CodeResources').exists())

    def test_extra_or_symlink_executables_fail_before_credentials(self):
        folder = self.app / 'Contents/MacOS'
        (folder / 'extra').symlink_to('spaceport')
        self.sign()
        (folder / 'extra').unlink()
        (folder / 'extra').mkdir()
        self.sign()
        (folder / 'extra').rmdir()
        (folder / 'sheprd-acp').unlink()
        (folder / 'sheprd-acp').symlink_to('spaceport')
        self.sign()
        self.assertFalse(any(c[0] == 'security' for c in self.calls()))

    def test_actual_signature_requires_exact_team_and_identifier(self):
        for key, value in (('MOCK_SIGNED_TEAM', 'OTHERTEAM1'),
                           ('MOCK_SIGNED_ID', 'ai.2389.spaceport.other')):
            with self.subTest(key=key):
                self.env[key] = value
                self.assertIn('Signed app Team or identifier mismatch', self.sign().stderr)
                del self.env[key]
        self.assertFalse(any(c[1:3] == ['notarytool', 'submit'] for c in self.calls()))

    def test_import_refuses_additional_identities(self):
        self.env['MOCK_IDENTITY_COUNT'] = '11'
        self.sign()
        self.assertFalse(any(c[:2] == ['codesign', '--force'] for c in self.calls()))
        self.assertTrue(any(c[:2] == ['security', 'delete-keychain'] for c in self.calls()))

    def test_installed_identity_never_mutates_keychains(self):
        key = self.root / 'fake.p8'
        key.write_text('test fixture')
        self.env['APPLE_API_KEY_PATH'] = str(key)
        self.sign(success=True, mode='installed')
        self.assertFalse(any(c[0] == 'security' for c in self.calls()))
        self.assertFalse(any('--keychain' in c for c in self.calls()))

    def test_cancellation_retains_submit_intent_and_cleans_keychain(self):
        self.env['MOCK_SUBMIT_CANCEL'] = '1'
        self.sign()
        self.assertTrue(list((self.root / 'receipts').glob('*/*/artifact.sha256')))
        self.assertTrue(any(c[:2] == ['security', 'delete-keychain'] for c in self.calls()))


class PreviewRoute(unittest.TestCase):
    def test_manual_artifacts_and_tagged_prereleases_keep_signing_isolated(self):
        workflow = (ROOT / '.github/workflows/macos-preview.yml').read_text()
        for text in ('workflow_dispatch:', 'push:', "tags: ['v*']", "github.repository == 'harperreed/spaceport'",
                     "github.actor == 'harperreed'", "github.triggering_actor == 'harperreed'",
                     'environment: macos-preview', 'contents: read', 'smoke-cli.py',
                     'actions/upload-artifact@', 'actions/download-artifact@'):
            self.assertIn(text, workflow)
        for text in ('pull_request:', 'update-homebrew', 'windows-',
                     'HERDR_RELEASE_VERSION:'):
            self.assertNotIn(text, workflow)
        sections = re.split(r'^  ([a-z-]+):\n', workflow.split('\njobs:\n', 1)[1], flags=re.M)
        jobs = dict(zip(sections[1::2], sections[2::2]))
        for name, job in jobs.items():
            if name != 'publish':
                self.assertNotIn('contents: write', job)
                self.assertNotIn('gh release', job)
            if name != 'sign':
                self.assertNotIn('secrets.', job)
        self.assertIn('needs: [validate, build, ci-gate]', jobs['sign'])
        self.assertIn('--repo harperreed/spaceport --actor harperreed', jobs['ci-gate'])
        self.assertIn("github.event_name == 'push'", jobs['publish'])
        self.assertIn("needs.validate.outputs.tag != ''", jobs['publish'])
        self.assertIn('--verify-tag --prerelease', jobs['publish'])
        self.assertIn('SHA256SUMS', jobs['publish'])
        self.assertIn('generate-changelog.sh --macos-preview', jobs['publish'])
        self.assertIn('preview-notes-base.py', jobs['publish'])
        self.assertNotIn("printf 'Signed and notarized", jobs['publish'])
        self.assertIn('verify-macos-preview.sh', jobs['sign'])
        for name in ('validate', 'sign', 'publish'):
            self.assertIn('preview-request.py', jobs[name])
            self.assertIn('fetch-depth: 0', jobs[name])
        self.assertIn("github.repository == 'sheprd-labs/sheprd'", (ROOT / '.github/workflows/release.yml').read_text())

    def test_metadata_has_no_inherited_owner_or_removed_feature_claim(self):
        metadata = plistlib.loads((ROOT / 'assets/macos/Info.plist').read_bytes())
        self.assertEqual(metadata['CFBundleName'], 'Spaceport')
        self.assertEqual(metadata['CFBundleDisplayName'], 'Spaceport')
        self.assertNotIn('iPhone', metadata['NSBluetoothAlwaysUsageDescription'])
        self.assertNotIn('finds computers', metadata['NSLocalNetworkUsageDescription'])
