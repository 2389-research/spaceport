# ABOUTME: Exercise native fixture path, version, and outcome rules with real files.
# ABOUTME: Reject missing error evidence, replacements, and sandbox escapes.
import importlib.util
import plistlib
from pathlib import Path
import tempfile
import unittest

SOURCE = Path(__file__).resolve().parents[1] / 'test-sparkle-native.py'


def load():
    spec = importlib.util.spec_from_file_location('native_fixture', SOURCE)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class NativeFixtureTests(unittest.TestCase):
    def test_driver_exists(self):
        self.assertTrue(SOURCE.is_file(), 'native fixture driver must exist')

    def test_information_checks_never_replace_manual_install_cases(self):
        native = load()
        for case in native.CASES:
            expected = 'information' if case in ('altered-feed', 'wrong-feed-key', 'aged-wrong-feed-key', 'downgrade') else 'manual'
            self.assertEqual(native.case_check_mode(case), expected)
        with self.assertRaises(ValueError):
            native.case_check_mode('unknown')

    def test_cases_choose_real_payload_versions(self):
        native = load()
        self.assertEqual(native.case_versions('valid-zip'), ('1', '2', '2'))
        self.assertEqual(native.case_versions('downgrade'), ('2', '1', '1'))
        self.assertEqual(native.case_versions('payload-downgrade'), ('2', '1', '3'))
        with self.assertRaises(ValueError):
            native.case_versions('unknown')

    def test_owned_path_rejects_symlink_escape(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / 'inside').mkdir()
            self.assertEqual(native.owned_path(root, root / 'inside'), root / 'inside')
            (root / 'escape').symlink_to(root.parent, target_is_directory=True)
            with self.assertRaises(ValueError):
                native.owned_path(root, root / 'escape')

    def test_environment_is_complete_and_excludes_inherited_secrets(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            env = native.fixture_environment(root)
            for key in ('HOME', 'CFFIXED_USER_HOME', 'TMPDIR', 'XDG_CONFIG_HOME',
                        'XDG_DATA_HOME', 'XDG_STATE_HOME', 'XDG_CACHE_HOME', 'XDG_RUNTIME_DIR'):
                self.assertTrue(Path(env[key]).is_relative_to(root))
                self.assertEqual(Path(env[key]).stat().st_mode & 0o777, 0o700)
            self.assertNotIn('SPARKLE_PRIVATE_KEY', env)

    def test_negative_requires_error_evidence(self):
        native = load()
        self.assertIsNone(native.assess('altered-feed', [], True, 1, False, 0))
        with self.assertRaises(ValueError):
            native.assess('altered-feed', [{'event': 'sparkle://did-abort-with-error', 'payload': {}}], True, 1, False, 0)

    def test_feed_rejection_requires_zero_downloads(self):
        native = load()
        with self.assertRaises(ValueError):
            native.assess('wrong-feed-key', [{'event': 'sparkle://did-abort-with-error', 'payload': {}}], True, 1, False, 1)

    def test_negative_rejects_replacement_or_relaunch(self):
        native = load()
        for unchanged, launches in ((False, 1), (True, 2)):
            with self.assertRaises(ValueError):
                native.assess('altered-zip', [{'event': 'sparkle://did-abort-with-error', 'payload': {}}], unchanged, launches, False, 1)

    def test_extract_callback_requires_specific_abort_evidence(self):
        native = load()
        with self.assertRaises(ValueError):
            native.assess('wrong-archive-key', [{'event': 'sparkle://did-extract-update', 'payload': {}}, {'event': 'sparkle://did-abort-with-error', 'payload': {}}], True, 1, False, 1)

    def test_defaults_absence_recognizes_actual_os_diagnostics(self):
        native = load()
        for message in (b"Error: Domain 'fixture' not found.\n", b'Domain fixture does not exist\n'):
            self.assertTrue(native.preferences_missing(1, message))
        self.assertFalse(native.preferences_missing(0, b''))
        self.assertFalse(native.preferences_missing(1, b'Permission denied'))

    def test_preferences_value_cleanup_accepts_only_empty_dictionary(self):
        native = load()
        for fmt in (plistlib.FMT_XML, plistlib.FMT_BINARY):
            self.assertTrue(native.preferences_values_absent(plistlib.dumps({}, fmt=fmt)))
            self.assertFalse(native.preferences_values_absent(plistlib.dumps({'fixture': True}, fmt=fmt)))
        with self.assertRaises(ValueError):
            native.preferences_values_absent(plistlib.dumps([]))

    def test_cleanup_failure_cannot_retain_a_passed_result(self):
        native = load()
        result = native.final_result({'status': 'passed'}, 'owned process did not exit')
        self.assertEqual(result['status'], 'failed')
        self.assertEqual(result['cleanup_error'], 'owned process did not exit')

    def test_preferences_path_requires_exact_fixture_domain(self):
        native = load()
        home = Path('/fixture-home')
        domain = 'ai.2389.spaceport.fixture.' + 'a' * 32
        self.assertEqual(native.preferences_path(domain, home), home / 'Library/Preferences' / (domain + '.plist'))
        for invalid in ('ai.2389.spaceport', domain + '/../../other', domain + '.other', 'com.apple.finder'):
            with self.assertRaises(ValueError):
                native.preferences_path(invalid, home)

    def test_preferences_claim_refuses_preexisting_files(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory).resolve()
            domain = 'ai.2389.spaceport.fixture.' + 'a' * 32
            path = native.preferences_path(domain, home)
            path.parent.mkdir(parents=True)
            record = native.preferences_record(domain, home)
            self.assertTrue(record['absent_before'])
            path.write_bytes(b'prior data')
            with self.assertRaises(ValueError):
                native.preferences_record(domain, home)

    def test_quit_request_writes_only_private_root_sentinel(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            native.request_quit(root)
            self.assertEqual((root / 'quit').read_text(), 'quit\n')
            self.assertEqual((root / 'quit').stat().st_mode & 0o777, 0o600)

    def test_quit_request_refuses_redirecting_even_inside_owned_root(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / 'quit').symlink_to(root / 'other')
            with self.assertRaises(ValueError):
                native.request_quit(root)
            self.assertFalse((root / 'other').exists())

    def test_quit_request_refuses_symlink_escape(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            outside = root.parent / (root.name + '-outside')
            (root / 'quit').symlink_to(outside)
            with self.assertRaises(ValueError):
                native.request_quit(root)
            self.assertFalse(outside.exists())

    def test_preferences_cleanup_rejects_unowned_or_unproved_records(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory).resolve()
            domain = 'ai.2389.spaceport.fixture.' + 'a' * 32
            record = native.preferences_record(domain, home)
            self.assertEqual(native.preferences_cleanup_path(record, home, {domain}), native.preferences_path(domain, home))
            for changed in (dict(record, absent_before=False), dict(record, path=str(home / 'other'))):
                with self.assertRaises(ValueError):
                    native.preferences_cleanup_path(changed, home, {domain})
            with self.assertRaises(ValueError):
                native.preferences_cleanup_path(record, home, set())

    def test_native_temp_requires_exact_private_owner_and_path(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory).resolve()
            owner = path.stat().st_uid
            native.validate_native_temp(path, path, owner, owner, 0o700)
            for actual, actual_owner, mode in ((path.parent, owner, 0o700),
                                               (path, owner + 1, 0o700),
                                               (path, owner, 0o755)):
                with self.assertRaises(ValueError):
                    native.validate_native_temp(actual, path, actual_owner, owner, mode)

    def test_relaunch_must_report_ready_from_the_new_pid(self):
        native = load()
        self.assertFalse(native.relaunch_ready([{'pid': 1, 'event': 'fixture://ready'}], 2))
        self.assertTrue(native.relaunch_ready([{'pid': 2, 'event': 'fixture://ready'}], 2))

    def test_valid_case_requires_exit_and_relaunch(self):
        native = load()
        self.assertIsNone(native.assess('valid-zip', [], False, 1, False, 1))
        self.assertEqual(native.assess('valid-zip', [], False, 2, True, 1), 'passed')

    def test_signed_feed_verifies_exact_tool_generated_bytes(self):
        native = load()
        import base64
        signature = base64.b64encode(bytes(range(64))).decode()
        content = b'<rss/>\n'
        signed = content + ('<!-- sparkle-signatures:\nedSignature: ' + signature + '\nlength: 7\n-->\n').encode()
        self.assertEqual(native.signed_feed_content(signed), content)
        with self.assertRaises(ValueError):
            native.signed_feed_content(signed.replace(b'length: 7', b'length: 8'))

    def test_xml_tool_signature_comes_from_embedded_metadata(self):
        native = load()
        import base64
        signature = base64.b64encode(bytes(range(64))).decode()
        with tempfile.TemporaryDirectory() as directory:
            feed = Path(directory) / 'feed.xml'
            feed.write_text('<rss/>\n<!-- sparkle-signatures:\nedSignature: ' + signature + '\nlength: 7\n-->\n')
            self.assertEqual(native.signature_result(b'', feed), signature)

    def test_installed_state_detects_plist_only_replacement(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / 'Fixture.app'
            (app / 'Contents/MacOS').mkdir(parents=True)
            (app / 'Contents/MacOS/sparkle_fixture').write_bytes(b'same executable')
            info = app / 'Contents/Info.plist'
            info.write_bytes(b'version 1')
            before = native.installed_state(app)
            info.write_bytes(b'version 2')
            after = native.installed_state(app)
            self.assertEqual(before['executable'], after['executable'])
            self.assertNotEqual(before, after)

    def test_downgrade_needs_no_update(self):
        native = load()
        self.assertIsNone(native.assess('downgrade', [{'event': 'sparkle://did-finish-update-cycle', 'payload': {}}], True, 1, False, 0))
        recorded = [{'event': 'sparkle://did-finish-loading-appcast'},
                    {'event': 'sparkle://did-not-find-update',
                     'payload': {'reason': 'onNewerThanLatestVersion', 'reasonCode': 2}}]
        self.assertEqual(native.assess('downgrade', recorded, True, 1, False, 0), 'passed')


if __name__ == '__main__':
    unittest.main()
