# ABOUTME: Check release request validation against real isolated Git repositories.
# ABOUTME: Cover tags, main ancestry, moved refs, and manual artifact requests.
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('preview_request', Path(__file__).resolve().parents[1] / 'preview-request.py')
request = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(request)


class PreviewRequestTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='spaceport-tag-')
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.git('init', '-b', 'main')
        self.git('config', 'user.name', 'Release test')
        self.git('config', 'user.email', 'release@example.invalid')
        self.git('commit', '--allow-empty', '-m', 'initial')
        self.sha = self.git('rev-parse', 'HEAD')
        self.git('remote', 'add', 'origin', str(self.repo))
        self.env = dict(GITHUB_EVENT_NAME='push', GITHUB_REF='refs/tags/v20261110.3',
                        GITHUB_SHA=self.sha, GITHUB_RUN_ATTEMPT='1')

    def git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.repo), *args], stderr=subprocess.DEVNULL, text=True).strip()

    def validate(self):
        return request.validate(self.env, self.repo)

    def test_lightweight_and_annotated_tags_select_same_commit(self):
        for annotated in (False, True):
            args = ('-a', '-m', 'preview') if annotated else ()
            self.git('tag', *args, 'v20261110.3')
            self.assertEqual(self.validate(), dict(version='20261110.3', sha=self.sha, tag='v20261110.3'))
            self.git('tag', '-d', 'v20261110.3')

    def test_unmerged_commit_and_moved_tag_refuse(self):
        self.git('checkout', '-b', 'feature')
        self.git('commit', '--allow-empty', '-m', 'unmerged')
        self.env['GITHUB_SHA'] = self.git('rev-parse', 'HEAD')
        self.git('tag', 'v20261110.3')
        with self.assertRaisesRegex(ValueError, 'main'):
            self.validate()
        self.git('checkout', 'main')
        self.env['GITHUB_SHA'] = self.sha
        with self.assertRaisesRegex(ValueError, 'tag'):
            self.validate()

    def test_invalid_events_refs_versions_and_reruns_refuse(self):
        self.git('tag', 'v20261110.3')
        for key, value in (('GITHUB_EVENT_NAME', 'pull_request'), ('GITHUB_REF', 'refs/heads/main'),
                           ('GITHUB_REF', 'refs/tags/v20261110.0'), ('GITHUB_REF', 'refs/tags/v20261110.3;echo'),
                           ('GITHUB_SHA', 'a' * 40), ('GITHUB_RUN_ATTEMPT', '2')):
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                request.validate(dict(self.env, **{key: value}), self.repo)

    def test_manual_main_requires_exact_input_sha_and_never_publishes(self):
        self.env.update(GITHUB_EVENT_NAME='workflow_dispatch', GITHUB_REF='refs/heads/main',
                        INPUT_VERSION='20261110.4', INPUT_SHA=self.sha)
        self.assertEqual(self.validate(), dict(version='20261110.4', sha=self.sha, tag=''))
        self.env['INPUT_SHA'] = 'a' * 40
        with self.assertRaises(ValueError):
            self.validate()

