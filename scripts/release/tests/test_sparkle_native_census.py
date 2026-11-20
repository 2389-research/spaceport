# ABOUTME: Read real ps census rows without rejecting valid negative uids.
# ABOUTME: Never count another account's process as an owned fixture process.
import importlib.util
import os
from pathlib import Path
import sys
import tempfile
import unittest


def load():
    path = Path(__file__).resolve().parents[1] / 'test-sparkle-native.py'
    spec = importlib.util.spec_from_file_location('native_fixture', path)
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    return native


class CensusRowTests(unittest.TestCase):
    def setUp(self):
        self.native = load()
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.helper = self.root / 'bin/helper'

    def census(self, rows):
        return self.native.census_matches('\n'.join(rows) + '\n', self.root, [])

    def test_negative_nobody_uid_row_is_neither_malformed_nor_owned(self):
        # macOS stamps `nobody` (4294967294) as uid -2, so ps prints the uid signed.
        self.assertEqual(self.census(['   598     -2 /usr/sbin/distnoted',
                                      f'{os.getpid():6d} {os.getuid():6d} {sys.executable}']), [])

    def test_negative_uid_row_naming_the_fixture_path_is_not_ours_to_refuse(self):
        self.assertEqual(self.census([f'   598     -2 {self.helper}']), [])

    def test_own_process_running_the_fixture_executable_is_matched(self):
        self.assertEqual(self.census([f'{os.getpid() + 1:6d} {os.getuid():6d} {self.helper}']), [os.getpid() + 1])

    def test_own_census_pid_and_paths_outside_the_fixture_are_not_matched(self):
        outside = Path(tempfile.gettempdir()) / 'unrelated-helper'
        self.assertEqual(self.census([f'{os.getpid():6d} {os.getuid():6d} {self.helper}',
                                      f'{os.getpid() + 1:6d} {os.getuid():6d} {outside}']), [])

    def test_malformed_rows_still_fail_the_census(self):
        for row in ('   598 /usr/sbin/distnoted', '  pid     -2 /usr/sbin/distnoted',
                    '   598 nobody /usr/sbin/distnoted', '   598      - /usr/sbin/distnoted',
                    '   598    2.5 /usr/sbin/distnoted'):
            with self.subTest(row=row), self.assertRaisesRegex(ValueError, 'unexpected row'):
                self.census([row])

    def test_relative_executable_name_never_matches_a_fixture_prefix(self):
        self.assertEqual(self.census([f'{os.getpid() + 1:6d} {os.getuid():6d} helper']), [])


@unittest.skipUnless(sys.platform == 'darwin', 'Requires the macOS process census')
class LiveCensusTests(unittest.TestCase):
    def test_live_census_accepts_every_row_of_the_real_process_table(self):
        native = load()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            self.assertEqual(native.fixture_processes(root, [root / 'bin/helper']), [])
