# ABOUTME: Reproduce Sparkle's installer-start callback before archive verification.
# ABOUTME: Keep exact pre-unarchive refusal proof separate from real install progress.
import importlib.util
from pathlib import Path
import unittest


def load():
    path = Path(__file__).resolve().parents[1] / 'test-sparkle-native.py'
    spec = importlib.util.spec_from_file_location('native_fixture', path)
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    return native


def recorded_refusal():
    # Shape/order from wrong-archive-key-v07pkdo_; private paths and IDs removed.
    names = ('did-finish-loading-appcast', 'did-find-valid-update',
             'user-did-make-choice', 'will-download-update', 'did-download-update',
             'will-extract-update', 'did-extract-update', 'did-abort-with-error')
    result = [{'event': 'sparkle://' + name, 'payload': {'version': '2'}} for name in names]
    result[2]['payload'] = dict(choice='install', version='2', stage='notDownloaded')
    result[-1]['payload'] = dict(code=4005, domain='SUSparkleErrorDomain',
        message='The update is improperly signed and could not be validated.', underlying=[
            dict(code=4005, domain='SUSparkleErrorDomain',
                 message='(Ed)DSA signature validation before unarchiving failed for update /fixture/update.zip'),
            dict(code=3002, domain='SUSparkleErrorDomain',
                 message='EdDSA signature does not match. Data of the update being checked is different than data that has been signed.')])
    return result


class InstallerCallbackTests(unittest.TestCase):
    def test_recorded_pre_unarchive_signature_refusal_passes_despite_start_callback(self):
        for case in ('wrong-archive-key', 'altered-zip'):
            with self.subTest(case=case):
                try:
                    outcome = load().assess(case, recorded_refusal(), True, 1, False, 1)
                except ValueError as error:
                    self.fail(f'Installer startup callback misclassified: {error}')
                self.assertEqual(outcome, 'passed')

    def test_payload_metadata_refusals_can_include_the_same_early_callback(self):
        for case in ('wrong-name-id', 'same-name-wrong-id', 'payload-downgrade'):
            records = recorded_refusal()
            cause = dict(code=4006, domain='SUSparkleErrorDomain') if case == 'payload-downgrade' else dict(
                code=3002, domain='SUSparkleErrorDomain',
                message='No suitable install is found in the update. The update will be rejected.')
            records[-1]['payload']['underlying'] = [dict(code=10, domain='SUSparkleErrorDomain'), cause]
            expected = 'observed-rejection' if case == 'same-name-wrong-id' else 'passed'
            with self.subTest(case=case):
                self.assertEqual(load().assess(case, records, True, 1, False, 1), expected)

    def test_signed_feed_and_selection_refusals_never_allow_install_progress(self):
        feed = [dict(event='sparkle://did-abort-with-error', payload=dict(code=1000,
                     domain='SUSparkleErrorDomain', underlying=[dict(code=3002,
                     domain='SUSparkleErrorDomain', message='EdDSA signature does not match.')]))]
        selection = [dict(event='sparkle://did-finish-loading-appcast'),
                     dict(event='sparkle://did-not-find-update',
                          payload=dict(reason='onNewerThanLatestVersion', reasonCode=2))]
        for case, records in (('wrong-feed-key', feed), ('downgrade', selection)):
            for name in ('will-install-update', 'will-relaunch-application', 'will-install-update-on-quit'):
                invalid = records + [dict(event='sparkle://' + name)]
                with self.subTest(case=case, event=name), self.assertRaises(ValueError):
                    load().assess(case, invalid, True, 1, False, 0)

    def test_start_callback_does_not_override_missing_or_unrelated_refusal(self):
        for change in ('network', 'cause', 'prevalidation_message'):
            records = recorded_refusal()
            payload = records[-1]['payload']
            if change == 'network':
                payload.update(code=2001, underlying=[])
            elif change == 'cause':
                payload['underlying'][-1]['code'] = 2001
            else:
                payload['underlying'][0]['message'] = 'Unrelated installer failure'
            with self.subTest(change=change), self.assertRaises(ValueError):
                load().assess('wrong-archive-key', records, True, 1, False, 1)

    def test_actual_install_or_relaunch_advancement_remains_forbidden(self):
        for name in ('will-install-update', 'will-relaunch-application', 'will-install-update-on-quit'):
            records = recorded_refusal()
            records.insert(-1, dict(event='sparkle://' + name, payload={'version': '2'}))
            with self.subTest(event=name), self.assertRaises(ValueError):
                load().assess('wrong-archive-key', records, True, 1, False, 1)
        for stage in ('downloaded', 'installing'):
            records = recorded_refusal()
            records.insert(-1, dict(event='sparkle://user-did-make-choice',
                                   payload=dict(choice='install', stage=stage, version='2')))
            with self.subTest(stage=stage), self.assertRaises(ValueError):
                load().assess('wrong-archive-key', records, True, 1, False, 1)

    def test_download_identity_and_host_invariants_still_apply(self):
        for unchanged, launches, exited, downloads in ((True, 1, False, 0),
                    (False, 1, False, 1), (True, 2, False, 1), (True, 1, True, 1)):
            with self.subTest(values=(unchanged, launches, exited, downloads)), self.assertRaises(ValueError):
                load().assess('wrong-archive-key', recorded_refusal(), unchanged, launches, exited, downloads)
