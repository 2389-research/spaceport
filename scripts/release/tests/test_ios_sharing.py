"""ABOUTME: Exercise TestFlight profile selection and the exported app's extension roster.
ABOUTME: Use disposable projects and plists without a signing identity or an Apple upload.
"""
import base64
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tempfile
import unittest
import uuid

RELEASE = Path(__file__).resolve().parents[1]
APP = 'ai.2389.spaceport.mobile'
ACTIVITY = APP + '.agent-status'
SHARE = APP + '.share'
GROUP = 'group.ai.2389.spaceport.mobile.widget'


@unittest.skipUnless(sys.platform == 'darwin', 'requires Apple provisioning tools')
class ShareSigning(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        scripts = self.root / 'scripts/release'
        scripts.mkdir(parents=True)
        for name in ('sign-ios.sh', 'common.sh'):
            shutil.copy(RELEASE / name, scripts)
        self.sign = ['/bin/bash', scripts / 'sign-ios.sh', '20261101.1', self.root]
        apple = self.root / 'mobile/apple'
        project = apple / 'SpaceportMobile.xcodeproj'
        project.mkdir(parents=True)
        (project / 'project.pbxproj').touch()
        library = self.root / 'mobile/.build/SpaceportMobileCore.xcframework'
        library.mkdir(parents=True)
        (library / 'Info.plist').touch()
        self.spec = apple / 'project.yml'
        self.spec.write_text(
            f'targets:\n  SpaceportMobile:\n    PRODUCT_BUNDLE_IDENTIFIER: {APP}\n'
            f'  AgentStatusWidget:\n    PRODUCT_BUNDLE_IDENTIFIER: {ACTIVITY}\n'
            f'    SPACEPORT_WIDGET_GROUP: {GROUP}\n'
        )
        self.env = {
            'PATH': os.environ['PATH'], 'HOME': str(self.root), 'SPACEPORT_APPLE_TEAM_ID': 'FAKETEAM00',
            **dict.fromkeys(('APPLE_API_ISSUER_ID', 'APPLE_API_KEY_ID', 'APPLE_API_PRIVATE_KEY',
                             'IOS_DISTRIBUTION_P12_BASE64', 'IOS_DISTRIBUTION_P12_PASSWORD'), '!'),
        }

    def add_share_target(self):
        with self.spec.open('a') as spec:
            spec.write(f'  ShareFile:\n    PRODUCT_BUNDLE_IDENTIFIER: {SHARE}\n')

    def test_checkout_with_share_target_requires_its_profile_before_signing(self):
        self.add_share_target()
        for form in ('B64', 'UUID'):
            run = subprocess.run(self.sign, env={
                **self.env, 'SPACEPORT_MOBILE_PROFILE_' + form: 'fixture',
                'SPACEPORT_ACTIVITY_PROFILE_' + form: 'fixture',
            }, capture_output=True, text=True)
            self.assertNotEqual(run.returncode, 0)
            self.assertIn('SPACEPORT_SHARE_PROFILE_' + form, run.stderr)

    def test_archive_and_export_map_every_target_and_clean_up_owned_profiles(self):
        # Intercept keychain mutations and archive creation. The script writes and reads real plists.
        tools = self.root / 'bin'
        tools.mkdir()
        security = tools / 'security'
        security.write_text("""#!/bin/sh
case "$1" in
    cms) cat "$4" ;;
    find-identity) printf '  1) %040d "Apple Distribution: Fixture (FAKETEAM00)"\n  1 valid identities found\n' 0 ;;
esac
""")
        security.chmod(0o755)
        xcode = tools / 'xcodebuild'
        xcode.write_text("""#!/bin/sh
set -eu
case "$1" in
    archive)
        printf '%s\n' "$@" > "$HOME/archive.args"
        env | cut -d= -f1 > "$HOME/archive.env"
        ;;
    -exportArchive)
        while [ "$1" != -exportOptionsPlist ]; do shift; done
        cp "$2" "$HOME/export.plist"
        exit 93
        ;;
esac
""")
        xcode.chmod(0o755)
        ids = dict(zip(('MOBILE', 'ACTIVITY', 'SHARE'), (APP, ACTIVITY, SHARE)))
        uuids = {key: str(uuid.uuid4()) for key in ids}
        blobs = {key: base64.b64encode(plistlib.dumps({'UUID': value})).decode()
                 for key, value in uuids.items()}
        installed = self.root / 'Library/MobileDevice/Provisioning Profiles'
        installed.mkdir(parents=True)
        preexisting = installed / (uuids['MOBILE'] + '.mobileprovision')
        preexisting.write_bytes(base64.b64decode(blobs['MOBILE']))
        for share in (False, True):
            if share:
                self.add_share_target()
            expected = list(ids) if share else ['MOBILE', 'ACTIVITY']
            for form, values in [('B64', blobs), ('UUID', uuids)]:
                with self.subTest(share=share, form=form):
                    env = {**self.env, 'PATH': f"{tools}:{self.env['PATH']}",
                           'IOS_DISTRIBUTION_P12_BASE64': base64.b64encode(b'fixture').decode(),
                           'SPACEPORT_SHARE_PROFILE_B64': blobs['SHARE']}
                    env.update({f'SPACEPORT_{key}_PROFILE_{form}': values[key] for key in expected})
                    run = subprocess.run(self.sign, env=env, capture_output=True, text=True)
                    self.assertEqual(run.returncode, 93, run.stderr)
                    args = (self.root / 'archive.args').read_text().splitlines()
                    self.assertEqual({arg for arg in args if arg.startswith('SPACEPORT_')},
                                     {f'SPACEPORT_{key}_PROFILE={uuids[key]}' for key in expected})
                    self.assertEqual('-allowProvisioningUpdates' in args, form == 'UUID')
                    with (self.root / 'export.plist').open('rb') as file:
                        self.assertEqual(plistlib.load(file)['provisioningProfiles'],
                                         {ids[key]: uuids[key] for key in expected})
                    variables = (self.root / 'archive.env').read_text().splitlines()
                    self.assertFalse(any(name.endswith('_PROFILE_B64') for name in variables))
                    self.assertEqual(set(installed.iterdir()), {preexisting})

    def test_export_requires_exact_declared_extensions(self):
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / 'SpaceportMobile.app'
            extension = app / 'PlugIns/AgentStatusWidget.appex'
            extension.mkdir(parents=True)
            for path, identity in [(app, APP), (extension, ACTIVITY)]:
                (path / 'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier': identity}))
            command = ['/bin/bash', RELEASE / 'verify-ios-app.sh', app, 'FAKETEAM00', '42', GROUP, APP, ACTIVITY]
            missing = subprocess.run(command + [SHARE], text=True, capture_output=True)
            self.assertNotEqual(missing.returncode, 0)
            self.assertIn('Missing exported extension ' + SHARE, missing.stderr)
            unexpected = app / 'PlugIns/ShareFile.appex'
            unexpected.mkdir()
            (unexpected / 'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier': SHARE}))
            extra = subprocess.run(command, text=True, capture_output=True)
            self.assertNotEqual(extra.returncode, 0)
            self.assertIn('Unexpected exported extension ' + SHARE, extra.stderr)
