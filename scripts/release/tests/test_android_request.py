# ABOUTME: Check which v* tags publish an Android APK, against real isolated Git repositories.
# ABOUTME: Cover the APK baseline, source change detection, version codes, and moved or unmerged tags.
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('android_request', Path(__file__).resolve().parents[1] / 'android-request.py')
request = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(request)


class AndroidRequestTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='spaceport-apk-')
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.git('init', '-b', 'main')
        self.git('config', 'user.name', 'Release test')
        self.git('config', 'user.email', 'release@example.invalid')
        self.git('remote', 'add', 'origin', str(self.repo))

    def git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.repo), *args], stderr=subprocess.DEVNULL, text=True).strip()

    def commit(self, path, text, tag):
        file = self.repo / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(text)
        self.git('add', '-A')
        self.git('commit', '-m', f'change {path}')
        self.git('tag', tag)
        return self.git('rev-parse', 'HEAD')

    def plan(self, tag, sha, published):
        return request.plan(self.repo, tag, sha, published)

    def test_first_apk_builds_and_unchanged_android_skips(self):
        first = self.commit('mobile/android/app/Main.kt', 'one', 'v20261120.1')
        self.assertEqual(self.plan('v20261120.1', first, []),
                         dict(tag='v20261120.1', version='20261120.1', sha=first, changed='true',
                              version_code='2026112001', baseline=''))
        second = self.commit('crates/desktop.rs', 'desktop only', 'v20261121.1')
        self.assertEqual(self.plan('v20261121.1', second, ['v20261120.1'])['changed'], 'false')

    def test_android_or_shared_controller_change_builds(self):
        self.commit('mobile/android/app/Main.kt', 'one', 'v20261120.1')
        for path, tag in (('mobile/android/app/Main.kt', 'v20261121.1'), ('mobile/rust/src/lib.rs', 'v20261121.2')):
            with self.subTest(path=path):
                sha = self.commit(path, tag, tag)
                result = self.plan(tag, sha, ['v20261120.1'])
                self.assertEqual((result['changed'], result['baseline']), ('true', 'v20261120.1'))

    def test_baseline_is_the_newest_earlier_tag_with_an_apk(self):
        self.commit('mobile/android/app/Main.kt', 'one', 'v20261120.1')
        # This tag changed Android but never got an APK, so the next tag must still build.
        self.commit('mobile/android/app/Main.kt', 'two', 'v20261120.2')
        sha = self.commit('docs/notes.md', 'notes', 'v20261121.1')
        result = self.plan('v20261121.1', sha, ['v20261120.1', 'v20261122.1', 'v20261121.1', 'ios-0.1-build.14'])
        self.assertEqual((result['changed'], result['baseline']), ('true', 'v20261120.1'))

    def test_version_code_orders_by_date_then_counter(self):
        self.assertLess(request.version_code('20261126.99'), request.version_code('20261127.1'))
        self.assertEqual(request.version_code('20261126.2'), 2026112602)
        with self.assertRaisesRegex(ValueError, 'counter'):
            request.version_code('20261126.100')

    def test_moved_unmerged_and_malformed_tags_refuse(self):
        sha = self.commit('mobile/android/app/Main.kt', 'one', 'v20261120.1')
        for tag in ('v20261120.0', 'v20261120.1;echo', 'ios-0.1-build.14', ''):
            with self.subTest(tag=tag), self.assertRaisesRegex(ValueError, 'tag'):
                self.plan(tag, sha, [])
        with self.assertRaisesRegex(ValueError, 'commit'):
            self.plan('v20261120.1', 'a' * 40, [])
        self.git('checkout', '-b', 'feature')
        branch = self.commit('mobile/android/app/Main.kt', 'two', 'v20261121.1')
        with self.assertRaisesRegex(ValueError, 'main'):
            self.plan('v20261121.1', branch, [])


if __name__ == '__main__':
    unittest.main()
