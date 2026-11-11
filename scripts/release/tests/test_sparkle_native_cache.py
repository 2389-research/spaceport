# ABOUTME: Prove ownership of exact fixture cache namespaces with real files.
# ABOUTME: Refuse cache deletion while a fixture helper or open file still exists.
import importlib.util
import os
from pathlib import Path
import shutil
import select
import subprocess
import sys
import tempfile
import unittest
import uuid


def load():
    path = Path(__file__).resolve().parents[1] / 'test-sparkle-native.py'
    spec = importlib.util.spec_from_file_location('native_fixture', path)
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    return native


class CacheCase(unittest.TestCase):
    def setUp(self):
        self.native = load()
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.home = self.root / 'account'
        (self.home / 'Library/Caches').mkdir(parents=True)
        self.domain = 'ai.2389.spaceport.fixture.' + uuid.uuid4().hex

    def claim(self):
        self.assertTrue(callable(getattr(self.native, 'cache_record', None)),
                        'cache ownership must be claimed before native launch')
        return self.native.cache_record(self.domain, self.home)


class CacheOwnershipTests(CacheCase):
    def test_claim_creates_private_exact_namespace_with_identity_proof(self):
        record = self.claim()
        path = self.home / 'Library/Caches' / self.domain
        self.assertEqual(record['path'], str(path))
        self.assertTrue(record['absent_before'])
        self.assertEqual(path.stat().st_mode & 0o777, 0o700)
        self.assertEqual(record['owner'], os.getuid())
        self.assertEqual((record['device'], record['inode']), (path.stat().st_dev, path.stat().st_ino))

    def test_preexisting_directory_and_symlink_are_never_claimed(self):
        self.assertTrue(callable(getattr(self.native, 'cache_record', None)))
        path = self.home / 'Library/Caches' / self.domain
        for kind in ('directory', 'symlink'):
            with self.subTest(kind=kind):
                if kind == 'directory':
                    path.mkdir()
                    (path / 'prior').write_text('untouched')
                else:
                    path.symlink_to(self.root, target_is_directory=True)
                self.assertTrue(callable(getattr(self.native, 'cache_record', None)))
                with self.assertRaises((ValueError, FileExistsError)):
                    self.native.cache_record(self.domain, self.home)
                if kind == 'directory':
                    self.assertEqual((path / 'prior').read_text(), 'untouched')
                    shutil.rmtree(path)
                else:
                    self.assertTrue(path.is_symlink())
                    path.unlink()

    def test_symlinked_or_writable_ancestors_are_rejected(self):
        caches = self.home / 'Library/Caches'
        caches.rmdir()
        caches.symlink_to(self.root, target_is_directory=True)
        self.assertTrue(callable(getattr(self.native, 'cache_record', None)))
        with self.assertRaises(ValueError):
            self.native.cache_record(self.domain, self.home)
        caches.unlink()
        caches.mkdir(mode=0o777)
        caches.chmod(0o777)
        with self.assertRaises(ValueError):
            self.native.cache_record(self.domain, self.home)

    def test_unowned_real_ancestor_is_rejected_before_any_creation(self):
        system = Path('/usr')
        if system.stat().st_uid == os.getuid():
            self.skipTest('System directory is owned by this test account')
        self.assertTrue(callable(getattr(self.native, 'cache_record', None)))
        with self.assertRaisesRegex(ValueError, 'owned'):
            self.native.cache_record(self.domain, system)

    def test_cleanup_rejects_replaced_or_forged_claim(self):
        record = self.claim()
        path = Path(record['path'])
        path.rename(path.with_name(self.domain + '.old'))
        path.mkdir(mode=0o700)
        (path / 'replacement').write_text('keep')
        with self.assertRaises(ValueError):
            self.native.cleanup_cache_records([record], {self.home}, {self.domain}, self.root)
        self.assertEqual((path / 'replacement').read_text(), 'keep')
        for changed in (dict(record, absent_before=False), dict(record, owner=os.getuid() + 1),
                        dict(record, path=str(self.root)), dict(record, inode=0)):
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                self.native.cleanup_cache_records([changed], {self.home}, {self.domain}, self.root)

    def test_rollback_removes_only_proven_empty_owned_claims(self):
        record = self.claim()
        path = Path(record['path'])
        self.native.rollback_cache_records([record], {self.home}, {self.domain})
        self.assertFalse(path.exists())
        record = self.claim()
        (path / 'unexpected').write_text('retain')
        with self.assertRaises(OSError):
            self.native.rollback_cache_records([record], {self.home}, {self.domain})
        self.assertTrue((path / 'unexpected').exists())

    def test_wrong_owner_is_rejected_with_real_filesystem_metadata_when_available(self):
        record = self.claim()
        path = Path(record['path'])
        foreign = path / 'foreign'
        foreign.write_text('retain')
        try:
            os.chown(foreign, os.getuid() + 1, -1)
        except PermissionError:
            self.skipTest('Changing actual file ownership requires privileges')
        try:
            with self.assertRaises(ValueError):
                self.native.cleanup_cache_records([record], {self.home}, {self.domain}, self.root)
        finally:
            os.chown(foreign, os.getuid(), -1)


@unittest.skipUnless(sys.platform == 'darwin', 'Requires macOS executable census and lsof')
class CacheProcessTests(CacheCase):
    def test_owned_cache_cleanup_preserves_other_domains_and_symlink_targets(self):
        record = self.claim()
        path = Path(record['path'])
        outside = self.home / 'Library/Caches/personal-cache'
        outside.mkdir()
        (outside / 'keep').write_text('personal')
        (path / 'org.sparkle-project.Sparkle/Installation').mkdir(parents=True)
        (path / 'link').symlink_to(outside, target_is_directory=True)
        self.native.cleanup_cache_records([record], {self.home}, {self.domain}, self.root)
        self.assertFalse(path.exists())
        self.assertEqual((outside / 'keep').read_text(), 'personal')

    def test_live_open_cache_directory_blocks_cleanup_without_signalling_child(self):
        record = self.claim()
        path = Path(record['path'])
        child = subprocess.Popen(['/bin/sleep', '60'], cwd=path)
        try:
            with self.assertRaisesRegex(ValueError, 'open|active|process'):
                self.native.cleanup_cache_records([record], {self.home}, {self.domain}, self.root)
            self.assertTrue(path.is_dir())
            self.assertIsNone(child.poll())
        finally:
            child.terminate()
            child.wait(timeout=5)
        self.native.cleanup_cache_records([record], {self.home}, {self.domain}, self.root)
        self.assertFalse(path.exists())

    def test_live_fixture_executable_blocks_cleanup_without_cache_open_files(self):
        record = self.claim()
        helper = self.root / 'bin/helper'
        helper.parent.mkdir()
        # Preserve the interpreter's actual @executable_path/../lib dependency.
        (self.root / 'lib').symlink_to(Path(sys.base_prefix) / 'lib', target_is_directory=True)
        shutil.copyfile(sys.executable, helper)
        helper.chmod(0o700)
        child = subprocess.Popen([str(helper), '-c',
                                  'import time; print("ready", flush=True); time.sleep(60)'], cwd='/',
                                 env={'PYTHONHOME': sys.base_prefix, 'PATH': '/usr/bin:/bin'},
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.addCleanup(child.stdout.close)
        self.addCleanup(child.stderr.close)
        try:
            self.assertTrue(select.select([child.stdout], [], [], 5)[0], 'helper startup must be bounded')
            self.assertEqual(child.stdout.readline(), b'ready\n', 'helper must complete real startup')
            self.assertIsNone(child.poll(), 'helper must actually be running')
            with self.assertRaisesRegex(ValueError, 'active|process|helper'):
                self.native.cleanup_cache_records([record], {self.home}, {self.domain}, self.root)
            self.assertIsNone(child.poll())
            self.assertTrue(Path(record['path']).exists())
        finally:
            if child.poll() is None:
                child.terminate()
            child.wait(timeout=5)

    def test_preexisting_private_cache_refuses_run_before_host_launch(self):
        self.native.fixture_environment(self.root)
        (self.root / 'launches').mkdir()
        existing = self.root / 'home/Library/Caches' / self.domain
        existing.mkdir()
        (existing / 'prior').write_text('keep')
        import json
        manifest = dict(root=str(self.root), bundle_id=self.domain, payload_id=self.domain,
                        executable=str(self.root / 'unlaunched'), case='valid-zip')
        (self.root / 'manifest.json').write_text(json.dumps(manifest))
        with self.assertRaisesRegex(ValueError, 'already exists'):
            self.native.run(self.root, 1)
        self.assertFalse((self.root / 'host.log').exists())
        self.assertFalse((self.root / 'waiting.json').exists())
        self.assertEqual((existing / 'prior').read_text(), 'keep')
        self.assertEqual(self.native.read_launches(self.root), [])

    def test_manifest_write_failure_rolls_back_only_new_empty_cache_claims(self):
        self.native.fixture_environment(self.root)
        (self.root / 'manifest.json').mkdir()
        manifest = dict(bundle_id=self.domain, payload_id=self.domain)
        with self.assertRaises(IsADirectoryError):
            self.native.claim_caches(self.root, manifest)
        for home in self.native.cache_homes(self.root):
            self.assertFalse(self.native.cache_path(self.domain, home).exists())

    def test_lsof_failure_cannot_masquerade_as_empty_output(self):
        self.assertTrue(callable(getattr(self.native, 'cache_open_processes', None)))
        with self.assertRaises(ValueError):
            self.native.cache_open_processes(self.root / 'not-a-directory')
