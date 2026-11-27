"""ABOUTME: Contracts for the iOS TestFlight route, never a real Apple upload.
ABOUTME: Check who receives secrets, what is verified before upload, and its refusals.
"""
import os
from pathlib import Path
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / 'scripts/release/sign-ios.sh'
WORKFLOW = (ROOT / '.github/workflows/ios-testflight.yml').read_text()


def jobs(text):
    sections = re.split(r'^  ([a-z-]+):\n', text.split('\njobs:\n', 1)[1], flags=re.M)
    return dict(zip(sections[1::2], sections[2::2]))


class TestflightRoute(unittest.TestCase):
    def test_only_the_signing_job_holds_credentials(self):
        route = jobs(WORKFLOW)
        self.assertEqual(set(route), {'build', 'sign', 'publish'})
        for name, job in route.items():
            if name != 'sign':
                self.assertNotIn('secrets.', job, name)
                self.assertNotIn('environment:', job, name)
        self.assertIn('environment: ios-testflight', route['sign'])
        for secret in ('IOS_DISTRIBUTION_P12_BASE64', 'IOS_DISTRIBUTION_P12_PASSWORD',
                       'APPLE_API_PRIVATE_KEY', 'APPLE_API_KEY_ID', 'APPLE_API_ISSUER_ID',
                       'SPACEPORT_MOBILE_PROFILE_B64', 'SPACEPORT_ACTIVITY_PROFILE_B64',
                       'SPACEPORT_SHARE_PROFILE_B64'):
            self.assertIn(f'{secret}: ${{{{ secrets.{secret} }}}}', route['sign'])
        self.assertIn('SPACEPORT_APPLE_TEAM_ID: ${{ vars.SPACEPORT_APPLE_TEAM_ID }}', route['sign'])

    def test_the_route_rides_the_macos_release_and_publishes_no_release_of_its_own(self):
        for text in ("workflows: ['macOS signed preview']", 'workflow_dispatch:',
                     'github.event.workflow_run.conclusion == \'success\'',
                     'github.event.workflow_run.event == \'push\'',
                     "github.repository == '2389-research/spaceport'"):
            self.assertIn(text, WORKFLOW)
        for text in ('gh release create', 'pull_request:', 'github.actor', 'github.triggering_actor'):
            self.assertNotIn(text, WORKFLOW)
        route = jobs(WORKFLOW)
        # A skipped publish is green, so the release attachment is the only writer.
        self.assertIn('contents: write', route['publish'])
        self.assertNotIn('contents: write', route['build'] + route['sign'])
        self.assertIn('check-tag-unmoved.sh', route['publish'])
        self.assertIn('gh release upload', route['publish'])
        self.assertIn('sha256sum -c "Spaceport-$VERSION-ios.ipa.sha256"', route['publish'])
        self.assertIn("\"$GITHUB_STEP_SUMMARY\"", route['publish'])

    def test_the_signing_job_uses_only_this_runs_checksummed_build_inputs(self):
        sign = jobs(WORKFLOW)['sign']
        self.assertIn('shasum -a 256 -c SHA256SUMS', sign)
        self.assertIn('name: ios-input-${{ github.run_id }}', sign)
        self.assertIn('bash scripts/release/sign-ios.sh "$VERSION" target/ios-release', sign)

    def test_no_simulator_suite_is_claimed(self):
        build = jobs(WORKFLOW)['build']
        self.assertIn('./mobile/build-apple --release', build)
        # The iOS verification target needs a disposable SSH host and a desktop
        # simulator, so this route records the gate as skipped instead of faking it.
        self.assertIn('SKIPPED GATE', build)
        self.assertIn('TODO(ios-simulator-gate)', build)
        executed = [line for line in build.splitlines()
                    if 'xcodebuild test' in line and not line.lstrip().startswith('#')]
        self.assertEqual(executed, [])


class SignerScript(unittest.TestCase):
    def test_credentials_are_read_only_after_validation(self):
        script = SCRIPT.read_text()
        self.assertTrue(script.startswith('#!/usr/bin/env bash\n'))
        self.assertIn('set +x\n', script)
        self.assertIn("trap 'printf \"sign-ios.sh: failed at line %d\\n\" \"$LINENO\" >&2' ERR", script)
        # The App Group the embedded profiles and both signed bundles must grant.
        self.assertIn('app_group=group.ai.2389.spaceport.mobile.widget', script)
        self.assertIn('bundle_id=ai.2389.spaceport.mobile\n', script)
        self.assertIn('extension_id=ai.2389.spaceport.mobile.agent-status', script)

    def test_the_upload_follows_its_own_verification(self):
        script = SCRIPT.read_text()
        verifier = (SCRIPT.parent / 'verify-ios-app.sh').read_text()
        for check in ('codesign --verify --deep --strict', 'bundle/embedded.mobileprovision',
                      '.["get-task-allow"] == false'):
            self.assertIn(check, verifier)
        self.assertIn('manageAppVersionAndBuildNumber', script)
        self.assertIn('xcrun altool --upload-app --type ios --file "$ipa"', script)
        self.assertLess(script.index('bash "$release_root/scripts/release/verify-ios-app.sh"'),
                        script.index('xcrun altool --upload-app'))
        self.assertIn('security delete-keychain "$keychain"', script)

    def test_usage_and_version_refusals(self):
        script = SCRIPT.read_text()
        calls = [
            (['bash', str(SCRIPT)], 'Usage: bash sign-ios.sh VERSION OUTPUT_DIR'),
            (['bash', str(SCRIPT), '1.2', str(ROOT)], 'Version must be YYYYMMDD.COUNTER'),
        ]
        for command, message in calls:
            with self.subTest(command=command):
                # A credential-free run: no Apple identity or profile is involved.
                result = subprocess.run(command, capture_output=True, text=True,
                                        env={key: value for key, value in os.environ.items()
                                             if not key.startswith(('APPLE_API_', 'IOS_', 'SPACEPORT_'))})
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(message, result.stderr)
        self.assertIn('version_check "$1"', script)


if __name__ == '__main__':
    unittest.main()
