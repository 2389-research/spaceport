# ABOUTME: Exercise the iOS workflow's source selection against real branches and tags.
# ABOUTME: A release tag may select an older main commit, but never a side branch.
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[3]
WORKFLOW = ROOT / '.github/workflows/ios-testflight.yml'


class BuildSource(unittest.TestCase):
    def test_tag_selects_its_main_commit_before_any_build(self):
        step = WORKFLOW.read_text().split('id: request\n', 1)[1]
        command = textwrap.dedent(step.split('run: |\n', 1)[1].split('\n      - ', 1)[0])
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)

            def git(*args):
                return subprocess.check_output(['git', *args], cwd=repo, text=True,
                                               stderr=subprocess.PIPE).strip()

            git('init', '-b', 'main')
            git('config', 'user.name', 'Release test')
            git('config', 'user.email', 'release@example.invalid')
            scripts = repo / 'scripts/release'
            scripts.mkdir(parents=True)
            shutil.copy(ROOT / 'scripts/release/common.sh', scripts)
            git('add', 'scripts')
            git('commit', '-m', 'Release base')
            first = git('rev-parse', 'HEAD')
            tag = 'v20261127.1'
            git('tag', '-a', tag, '-m', 'Preview')
            git('switch', '-c', 'side')
            git('commit', '--allow-empty', '-m', 'Unmerged change')
            side = git('rev-parse', 'HEAD')
            git('switch', 'main')
            git('commit', '--allow-empty', '-m', 'Later main')
            latest = git('rev-parse', 'HEAD')
            git('update-ref', 'refs/remotes/origin/main', latest)

            def select(sha):
                env = dict(os.environ, GITHUB_EVENT_NAME='workflow_run', INPUT_BRANCH=tag,
                           INPUT_SHA=sha, GITHUB_OUTPUT=str(repo / 'outputs'))
                return subprocess.run(['bash', '-c', command], cwd=repo, env=env, capture_output=True)

            def manual(supplied_tag, version='20261127.1'):
                output = repo / 'manual-outputs'
                output.write_text('')
                env = dict(os.environ, GITHUB_EVENT_NAME='workflow_dispatch', INPUT_TAG=supplied_tag,
                           INPUT_VERSION=version, GITHUB_OUTPUT=str(output))
                result = subprocess.run(['bash', '-c', command], cwd=repo, env=env, capture_output=True)
                return result.returncode, output.read_text().splitlines()

            self.assertNotEqual(manual(tag)[0], 0, 'Manual tag is older than the selected main commit')
            self.assertNotEqual(manual('v20261127.2', '20261127.2')[0], 0, 'Missing tag')
            self.assertNotEqual(manual('', '20261127.100')[0], 0, 'Counter does not fit the build number')
            self.assertEqual(manual('', '20261127.99'), (0, ['version=20261127.99', 'tag=', 'sha=' + latest]))
            git('tag', '-f', '-a', tag, latest, '-m', 'Current main')
            self.assertEqual(manual(tag), (0, ['version=20261127.1', 'tag=' + tag, 'sha=' + latest]))

            self.assertNotEqual(select(side).returncode, 0, 'Tag points elsewhere')
            git('tag', '-f', tag, side)
            self.assertNotEqual(select(side).returncode, 0, 'Tag selects an unmerged commit')
            self.assertEqual(git('rev-parse', 'HEAD'), latest)
            git('tag', '-f', '-a', tag, first, '-m', 'Preview')
            result = select(first)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(git('rev-parse', 'HEAD'), first)
