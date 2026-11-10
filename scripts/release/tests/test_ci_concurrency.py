# ABOUTME: Keep running main CI available to tagged release qualification.
# ABOUTME: Check the actual workflow expression while preserving PR cancellation.
import re
import unittest

from scripts.release.tests.test_ci_policy import WORKFLOW, condition_matches


class CiConcurrency(unittest.TestCase):
    def test_main_qualification_survives_new_pushes(self):
        setting = re.search(r'^  cancel-in-progress: (.+)$', WORKFLOW, re.M)[1]
        self.assertTrue(setting.startswith('${{ ') and setting.endswith(' }}'))
        jobs = {'cancel': '    if: >-\n      ' + setting[4:-3] + '\n'}
        for ref, cancel in (('refs/heads/main', False), ('refs/pull/42/merge', True),
                            ('refs/heads/topic', True)):
            with self.subTest(ref=ref):
                self.assertEqual(condition_matches('cancel', {'github.ref': ref}, jobs), cancel)
