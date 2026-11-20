# ABOUTME: Check Darwin process rows before scoped fixture-cache cleanup.
# ABOUTME: Accept signed UID display while preserving exact paths and malformed-row refusal.
import os
from pathlib import Path
import tempfile
import unittest

from test_sparkle_native_cache import load


class NativeCensusTests(unittest.TestCase):
    def setUp(self):
        self.native = load()
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.cache = self.root / 'cache'
        self.pid = os.getpid() + 1

    def test_signed_uid_retains_active_fixture_and_cache_matches(self):
        for uid in ('-2', '0', '501'):
            for executable in (self.root / 'fixture app', self.cache / 'helper'):
                with self.subTest(uid=uid, executable=executable):
                    row = f'{self.pid} {uid} {executable}\n'
                    self.assertEqual(
                        self.native.parse_fixture_processes(row, self.root, [self.cache]),
                        [self.pid],
                    )

    def test_signed_uid_outside_claims_does_not_match_prefix_neighbor_or_self(self):
        output = (f'{self.pid} -2 {self.root}-neighbor/app\n'
                  f'{os.getpid()} -2 {self.root}/runner\n')
        self.assertEqual(self.native.parse_fixture_processes(output, self.root, [self.cache]), [])

    def test_malformed_uid_or_pid_still_refuses_cleanup(self):
        rows = [f'{self.pid} {uid} {self.root}/app\n'
                for uid in ('-', '--2', '+2', 'unknown', '5.0')]
        rows += [f'-2 501 {self.root}/app\n', f'{self.pid} 501\n']
        for row in rows:
            with self.subTest(row=row):
                with self.assertRaisesRegex(ValueError, 'unexpected row'):
                    self.native.parse_fixture_processes(row, self.root, [self.cache])
