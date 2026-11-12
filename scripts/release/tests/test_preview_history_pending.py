"""ABOUTME: Reserve tagged previews whose GitHub run title is not initialized.
ABOUTME: Keep uncertain submissions and malformed metadata from freeing versions.
"""
import unittest

import test_preview_history as fixtures


class PendingPreviewHistory(unittest.TestCase):
    def pending(self):
        # Shape observed on GitHub run 43098075293, before any job was created.
        return dict(id=19, display_title="Merge pull request #5: Bound notarization uploads",
                    event="push", head_branch="v20261111.3", head_sha=fixtures.SHA,
                    status="queued", conclusion=None)

    def check(self, pending, version="20261109.1"):
        page = dict(total_count=2, workflow_runs=[fixtures.run(version=version), pending])
        fixtures.history.check(version, fixtures.SHA, "20", "1", lambda _: page)

    def test_pending_tag_does_not_block_a_distinct_fresh_version(self):
        self.check(self.pending())

    def test_pending_tag_still_reserves_its_version(self):
        with self.assertRaisesRegex(ValueError, "already reserved"):
            self.check(self.pending(), version="20261111.3")

    def test_unrecognized_history_requires_a_valid_push_tag_and_commit(self):
        for changes in ({"event": "workflow_dispatch"}, {"head_branch": "main"},
                        {"head_branch": "v20261111.03"}, {"head_sha": "unknown"}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.check({**self.pending(), **changes})
