# ABOUTME: Check the shared release helpers' pinned formats and gh invocation contract.
# ABOUTME: These pure fixtures never call GitHub or read credentials.
import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('release_common', SCRIPTS / '_release_common.py')
common = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(common)
SHA = 'a' * 40


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), SCRIPTS / f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class VersionTests(unittest.TestCase):
    def test_calendar_version_needs_eight_digit_date_and_counter_from_one(self):
        for value in ('20261110.1', '20261231.10', '20300101.999'):
            with self.subTest(value=value):
                self.assertTrue(common.is_version(value))

    def test_prefixed_suffixed_padded_or_short_versions_refuse(self):
        for value in ('v20261110.1', '20261110', '20261110.0', '20261110.01', '2026111.1',
                      '202611101.1', '20261110.1.1', '20261110.1-rc1', '20261110.1;echo', ''):
            with self.subTest(value=value):
                self.assertFalse(common.is_version(value))


class CommitTests(unittest.TestCase):
    def test_full_lowercase_commit_is_accepted(self):
        self.assertTrue(common.is_commit(SHA))

    def test_truncated_uppercase_or_nonhex_commits_refuse(self):
        for value in (SHA[:-1], SHA + 'a', SHA.upper(), 'g' * 40, 'v' + SHA[1:], '', 'unknown'):
            with self.subTest(value=value):
                self.assertFalse(common.is_commit(value))


class GhTests(unittest.TestCase):
    def test_gh_returns_stdout_with_captured_stderr_and_bounded_time(self):
        with mock.patch.object(common, 'subprocess') as process:
            process.check_output.return_value = b'{}'
            self.assertEqual(common.gh('api', 'repos/2389-research/spaceport'), b'{}')
            process.check_output.assert_called_once_with(['gh', 'api', 'repos/2389-research/spaceport'],
                                                        stderr=process.PIPE, timeout=120)

    def test_gh_accepts_a_shortened_timeout(self):
        with mock.patch.object(common, 'subprocess') as process:
            common.gh('api', 'x', timeout=30)
            self.assertEqual(process.check_output.call_args.kwargs['timeout'], 30)

    def test_gh_failure_raises_the_subprocess_error(self):
        with mock.patch.object(common, 'subprocess') as process:
            process.check_output.side_effect = subprocess.CalledProcessError(1, 'gh')
            with self.assertRaises(subprocess.SubprocessError):
                common.gh('api', 'x')


class SharedDefinitionTests(unittest.TestCase):
    def test_preview_scripts_use_the_one_shared_definition(self):
        for name in ('preview-request', 'preview-history', 'publish-sparkle-feed'):
            with self.subTest(script=name):
                self.assertEqual(Path(load(name).common.__file__).resolve(),
                                 SCRIPTS / '_release_common.py')

    def test_publish_refuses_a_noncommit_sha_before_reading_the_feed(self):
        publish = load('publish-sparkle-feed')
        with self.assertRaisesRegex(ValueError, 'Full expected source SHA required'):
            publish.publish(SCRIPTS / 'missing-appcast.xml', 'v20261110.1')
