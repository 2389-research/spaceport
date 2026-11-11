# ABOUTME: Cover strict native evidence and failure cleanup with owned processes.
# ABOUTME: Data tests describe SDK errors; they do not claim native installation.
import contextlib
import io
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import sys
import uuid

SOURCE = Path(__file__).resolve().parents[1] / 'test-sparkle-native.py'


def load():
    spec = importlib.util.spec_from_file_location('native_fixture', SOURCE)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def event(name, payload=None):
    return {'event': 'sparkle://' + name, 'payload': payload or {}}


def error(code, message='', underlying=()):
    return dict(code=code, domain='SUSparkleErrorDomain', message=message, underlying=list(underlying))


def evidence(case):
    if case in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key'):
        return [event('did-abort-with-error', error(1000, underlying=[error(3002, 'EdDSA signature does not match.')]))], 0
    if case == 'downgrade':
        reason = dict(reason='onNewerThanLatestVersion', reasonCode=2)
        return [event('did-finish-loading-appcast'), event('did-not-find-update', reason),
                event('did-abort-with-error', dict(error(1001), noUpdate=reason))], 0
    prefix = [event('did-finish-loading-appcast'), event('did-find-valid-update'),
              event('will-download-update'), event('did-download-update')]
    if case in ('altered-zip', 'wrong-archive-key'):
        cause = error(3002, 'EdDSA signature does not match.')
        return prefix + [event('will-extract-update'), event('did-abort-with-error', error(4005, underlying=[error(4005, '(Ed)DSA signature validation before unarchiving failed for update /fixture/update.zip'), cause]))], 1
    if case in ('wrong-name-id', 'same-name-wrong-id'):
        cause = error(3002, 'No suitable install is found in the update. The update will be rejected.')
        return prefix + [event('will-extract-update'), event('did-abort-with-error', error(4005, underlying=[error(10), cause]))], 1
    return prefix + [event('will-extract-update'), event('did-abort-with-error', error(4005, underlying=[error(10), error(4006)]))], 1


class StrictEvidenceTests(unittest.TestCase):
    def test_aged_bad_feed_is_an_explicit_read_only_fixture(self):
        native = load()
        self.assertIn('aged-wrong-feed-key', native.CASES)
        self.assertEqual(native.case_check_mode('aged-wrong-feed-key'), 'information')
        self.assertEqual(native.case_versions('aged-wrong-feed-key'), ('1', '2', '2'))

    def test_expected_sdk_errors_and_stages(self):
        native = load()
        for case in native.CASES:
            if case.startswith('valid-'):
                continue
            with self.subTest(case=case):
                records, downloads = evidence(case)
                expected = 'observed-rejection' if case == 'same-name-wrong-id' else 'passed'
                self.assertEqual(native.assess(case, records, True, 1, False, downloads), expected)

    def test_unrelated_aborts_never_pass_any_negative_case(self):
        native = load()
        for case in native.CASES:
            if case.startswith('valid-'):
                continue
            for payload in (error(5), error(2001), dict(code=-1005, domain='NSURLErrorDomain')):
                with self.subTest(case=case, payload=payload), self.assertRaises(ValueError):
                    native.assess(case, [event('did-abort-with-error', payload)], True, 1, False, 0)

    def test_archive_errors_require_download_and_stage(self):
        native = load()
        for case in ('altered-zip', 'wrong-archive-key', 'wrong-name-id', 'same-name-wrong-id', 'payload-downgrade'):
            records, _ = evidence(case)
            with self.subTest(case=case), self.assertRaises(ValueError):
                native.assess(case, records, True, 1, False, 0)
            with self.subTest(case=case), self.assertRaises(ValueError):
                native.assess(case, records[-1:], True, 1, False, 1)

    def test_installation_start_invalidates_signature_refusal(self):
        for case in ('altered-zip', 'wrong-archive-key'):
            records, downloads = evidence(case)
            records.insert(-1, event('will-install-update'))
            with self.subTest(case=case), self.assertRaises(ValueError):
                load().assess(case, records, True, 1, False, downloads)

    def test_wrong_domain_or_cause_never_passes_expected_stage(self):
        for case in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key', 'altered-zip', 'wrong-archive-key',
                     'wrong-name-id', 'same-name-wrong-id', 'payload-downgrade'):
            for field, value in (('domain', 'NSURLErrorDomain'), ('code', 2001)):
                records, downloads = evidence(case)
                records[-1]['payload']['underlying'][-1][field] = value
                with self.subTest(case=case, field=field), self.assertRaises(ValueError):
                    load().assess(case, records, True, 1, False, downloads)

    def test_wrong_no_update_reason_is_rejected(self):
        with self.assertRaises(ValueError):
            load().assess('downgrade', [event('did-not-find-update', dict(reason='onLatestVersion', reasonCode=1))], True, 1, False, 0)

    def test_signature_code_without_mismatch_cause_is_rejected(self):
        for case in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key', 'altered-zip', 'wrong-archive-key'):
            records, downloads = evidence(case)
            records[-1]['payload']['underlying'][-1]['message'] = 'Invalid arguments to verifyStream'
            with self.subTest(case=case), self.assertRaises(ValueError):
                load().assess(case, records, True, 1, False, downloads)


@unittest.skipUnless(sys.platform == 'darwin', 'Requires macOS public defaults API')
class FailureCleanupTests(unittest.TestCase):
    def test_missing_installed_metadata_retains_failure_and_reaps_owned_child(self):
        self.run_filesystem_failure(missing_plist=True)

    def test_request_evidence_failure_still_reaps_child_and_cleans_preferences(self):
        self.run_filesystem_failure(missing_plist=False)

    def run_filesystem_failure(self, missing_plist):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            root.chmod(0o700)
            (root / 'launches').mkdir()
            executable = root / 'installed/Fixture.app/Contents/MacOS/sparkle_fixture'
            executable.parent.mkdir(parents=True)
            executable.write_text('#!/bin/sh\nexec /bin/sleep 60\n')
            executable.chmod(0o700)
            if not missing_plist:
                (executable.parents[1] / 'Info.plist').write_bytes(b'fixture metadata')
                (root / 'requests.json').mkdir()  # Real evidence-write filesystem failure.
            domain = 'ai.2389.spaceport.fixture.' + uuid.uuid4().hex
            manifest = dict(root=str(root), app=str(executable.parents[2]), executable=str(executable),
                            case='altered-feed', check_mode='information', environment=native.fixture_environment(root),
                            bundle_id=domain, payload_id=domain, port=0, archive='update.zip', before={})
            (root / 'manifest.json').write_text(json.dumps(manifest))
            children = []
            popen = subprocess.Popen
            def launch(*args, **kwargs):
                child = popen(*args, **kwargs)
                if args[0] == [str(executable)]:
                    children.append(child)
                return child
            try:
                with contextlib.redirect_stdout(io.StringIO()), patch.object(native.subprocess, 'Popen', side_effect=launch):
                    with self.assertRaises((ValueError, OSError)):
                        native.run(root, 0.1)
                self.assertEqual(len(children), 1)
                self.assertIsNotNone(children[0].poll(), 'owned sleeping child must be reaped despite missing Info.plist')
                result = json.loads((root / 'result.json').read_text())
                self.assertEqual(result['status'], 'failed')
                self.assertIn('diagnostic_errors', result)
                self.assertTrue((root / 'quit').exists())
                self.assertTrue((root / 'preferences-cleanup.json').exists())
            finally:
                for child in children:
                    if child.poll() is None:
                        child.terminate()
                        child.wait(timeout=5)
