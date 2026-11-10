# ABOUTME: Select changelog boundaries from published releases and real Git ancestry.
# ABOUTME: Keep failed tags and unrelated histories out of preview release boundaries.
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('preview_notes_base', Path(__file__).resolve().parents[1] / 'preview-notes-base.py')
notes = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(notes)


class PreviewNotesBase(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='spaceport-notes-base-')
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.git('init', '-b', 'main')
        self.git('config', 'user.name', 'Release test')
        self.git('config', 'user.email', 'release@example.invalid')
        self.git('commit', '--allow-empty', '-m', 'baseline')
        self.baseline = self.git('rev-parse', 'HEAD')
        self.git('commit', '--allow-empty', '-m', 'feat: first preview')
        self.git('tag', 'v20261110.1')
        self.first = self.git('rev-parse', 'HEAD')
        self.git('commit', '--allow-empty', '-m', 'feat: nebula')
        self.git('tag', 'v20261110.2')
        self.git('commit', '--allow-empty', '-m', 'fix: release')

    def git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.repo), *args],
                                       env={**os.environ, 'GIT_CONFIG_NOSYSTEM': '1',
                                            'GIT_CONFIG_GLOBAL': os.devnull},
                                       stderr=subprocess.DEVNULL, text=True, timeout=30).strip()

    def test_first_release_uses_fork_baseline_even_with_unpublished_tags(self):
        self.assertEqual(notes.select_base(self.repo, self.baseline, []), self.baseline)

    def test_only_published_ancestor_tags_bound_the_next_notes(self):
        self.assertEqual(notes.select_base(self.repo, self.baseline, ['v20261110.1']), self.first)
        self.git('checkout', '-b', 'other', self.baseline)
        self.git('commit', '--allow-empty', '-m', 'unrelated')
        self.git('tag', 'v20261110.9')
        self.git('checkout', 'main')
        self.assertEqual(notes.select_base(self.repo, self.baseline,
                                          ['v20261110.9', 'v20261110.1']), self.first)

    def test_latest_calendar_version_is_selected(self):
        second = self.git('rev-parse', 'v20261110.2')
        self.assertEqual(notes.select_base(self.repo, self.baseline,
                                          ['v20261110.1', 'v20261110.2']), second)

    def test_missing_published_tag_and_invalid_baseline_fail_closed(self):
        with self.assertRaises(subprocess.CalledProcessError):
            notes.select_base(self.repo, self.baseline, ['v20261110.99'])
        with self.assertRaises(ValueError):
            notes.select_base(self.repo, 'not a commit', [])
