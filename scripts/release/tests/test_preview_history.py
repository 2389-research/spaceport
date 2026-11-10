# ABOUTME: Test conservative hosted preview version reservations over paged run history.
# ABOUTME: These pure boundary fixtures never call GitHub or read credentials.
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('preview_history', Path(__file__).resolve().parents[1] / 'preview-history.py')
history = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(history)
SHA = 'a' * 40


def run(run_id=20, version='20261109.1'):
    return dict(id=run_id, display_title=f'macOS preview {version} at {SHA}')


class PreviewHistoryTests(unittest.TestCase):
    def check(self, pages, attempt='1'):
        return history.check('20261109.1', SHA, '20', attempt, lambda page: pages[page - 1])

    def test_current_run_and_distinct_counter_are_allowed(self):
        self.check([dict(total_count=2, workflow_runs=[run(), run(19, '20261109.10')])])

    def test_tag_and_manual_share_version_reservations(self):
        tagged = dict(run(), display_title=f'macOS preview v20261109.1 at {SHA}')
        self.check([dict(total_count=1, workflow_runs=[tagged])])
        with self.assertRaisesRegex(ValueError, 'already reserved'):
            self.check([dict(total_count=2, workflow_runs=[tagged, run(19)])])

    def test_invalid_version_tags_cannot_poison_later_valid_requests(self):
        for tag in ('v1.2.3', 'vbad', 'v20261109.0', 'v20261109.1;echo'):
            with self.subTest(tag=tag):
                invalid = dict(run(19), display_title=f'macOS preview {tag} at {SHA}')
                self.check([dict(total_count=2, workflow_runs=[run(), invalid])])

    def test_current_invalid_title_and_malformed_history_still_refuse(self):
        for title in (f'macOS preview vbad at {SHA}', 'macOS preview vbad at unknown',
                      f'other preview vbad at {SHA}', f'macOS preview bad token at {SHA}'):
            with self.subTest(title=title), self.assertRaises(ValueError):
                self.check([dict(total_count=1, workflow_runs=[dict(run(), display_title=title)])])

    def test_previous_same_version_refuses_regardless_of_conclusion(self):
        for conclusion in ('success', 'cancelled', 'failure', None):
            with self.subTest(conclusion=conclusion), self.assertRaisesRegex(ValueError, 'already reserved'):
                self.check([dict(total_count=2, workflow_runs=[run(), dict(run(19), conclusion=conclusion)])])

    def test_reruns_refuse_before_reading_history(self):
        with self.assertRaisesRegex(ValueError, 'rerun'):
            self.check([], attempt='2')

    def test_pagination_is_complete_and_duplicate_on_later_page_refuses(self):
        first = [run()] + [run(100 + n, f'20261108.{n + 1}') for n in range(99)]
        self.check([dict(total_count=101, workflow_runs=first), dict(total_count=101, workflow_runs=[run(300, '20261108.100')])])
        with self.assertRaisesRegex(ValueError, 'already reserved'):
            self.check([dict(total_count=101, workflow_runs=first), dict(total_count=101, workflow_runs=[run(19)])])

    def test_incomplete_or_changed_history_fails_closed(self):
        for page in (dict(total_count=2, workflow_runs=[run()]),
                     dict(total_count=1001, workflow_runs=[run()]),
                     dict(total_count=2, workflow_runs=[run(), run()]),
                     dict(total_count=1, workflow_runs=[run(19, '20261108.1')]),
                     dict(total_count=1, workflow_runs=[dict(run(), display_title='unknown')]), {}):
            with self.subTest(page=page), self.assertRaises(ValueError):
                self.check([page])

    def test_api_unavailable_propagates_failure(self):
        def unavailable(_):
            raise ValueError('GitHub history unavailable')
        with self.assertRaisesRegex(ValueError, 'unavailable'):
            history.check('20261109.1', SHA, '20', '1', unavailable)
