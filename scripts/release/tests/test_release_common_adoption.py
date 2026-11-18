# ABOUTME: Check the remaining release scripts load the one shared release helper.
# ABOUTME: Keep each script's own refusal message while its format rules are shared.
import contextlib
import importlib.util
import io
from pathlib import Path
import re
import unittest
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1]
ADOPTERS = ('check-ci', 'package-windows', 'preview-notes-base', 'sparkle-feed', 'windows-installer')
VERSION_REFUSAL = ('Version must be YYYYMMDD.COUNTER (no v prefix, counter from 1, '
                   'no leading zeros)')
SHA_REFUSAL = 'sha must be a full lowercase commit SHA'


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), SCRIPTS / f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def refusals():
    """Each script's own format check, called as its own code calls it."""
    return {
        'package-windows': (
            lambda module: module.package('v20261110.1', 'x86_64-pc-windows-msvc', Path('binary'),
                                          Path('launcher'), Path('output'), Path('notices')),
            VERSION_REFUSAL),
        'preview-notes-base': (
            lambda module: module.select_base(Path('.'), 'not a commit', []),
            'Invalid fork baseline SHA'),
        'sparkle-feed': (
            lambda module: module.version_tuple('v20261110.1'),
            'Version must be YYYYMMDD.COUNTER without leading zeros'),
        'windows-installer': (
            lambda module: module.iscc_arguments('v20261110.1', 'x86_64-pc-windows-msvc', 'source', 'out'),
            VERSION_REFUSAL),
    }


class SharedAdoptionTests(unittest.TestCase):
    def test_release_scripts_load_the_one_shared_helper(self):
        for name in ADOPTERS:
            with self.subTest(script=name):
                self.assertEqual(Path(load(name).common.__file__).resolve(),
                                 SCRIPTS / '_release_common.py')

    def test_each_script_keeps_its_own_refusal_message(self):
        for name, (refusal, message) in refusals().items():
            with self.subTest(script=name), self.assertRaisesRegex(ValueError, re.escape(message)):
                refusal(load(name))

    def test_refusals_route_through_the_shared_helper(self):
        for name, (refusal, message) in refusals().items():
            module = load(name)
            shared = 'is_commit' if name == 'preview-notes-base' else 'is_version'
            with self.subTest(script=name), \
                    mock.patch.object(module.common, shared, return_value=False), \
                    self.assertRaisesRegex(ValueError, re.escape(message)):
                refusal(module)

    def test_check_ci_keeps_its_own_sha_refusal_before_asking_github(self):
        check_ci = load('check-ci')
        with mock.patch('sys.argv', ['check-ci.py', 'v' + 'a' * 39]), \
                mock.patch.object(check_ci, 'require_ci') as require, \
                contextlib.redirect_stderr(io.StringIO()) as errors, \
                self.assertRaises(SystemExit) as error:
            check_ci.main()
        self.assertEqual(error.exception.code, 2)
        self.assertIn(SHA_REFUSAL, errors.getvalue())
        require.assert_not_called()

    def test_check_ci_sha_check_routes_through_the_shared_helper(self):
        check_ci = load('check-ci')
        with mock.patch.object(check_ci.common, 'is_commit', return_value=False), \
                mock.patch('sys.argv', ['check-ci.py', 'a' * 40]), \
                mock.patch.object(check_ci, 'require_ci') as require, \
                contextlib.redirect_stderr(io.StringIO()), \
                self.assertRaises(SystemExit) as error:
            check_ci.main()
        self.assertEqual(error.exception.code, 2)
        require.assert_not_called()


if __name__ == '__main__':
    unittest.main()
