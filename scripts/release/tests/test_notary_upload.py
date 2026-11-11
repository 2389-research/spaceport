"""ABOUTME: Bound real upload subprocesses and preserve their partial receipts.
ABOUTME: Signing fixtures check control flow only, never Apple acceptance.
"""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

import test_macos_preview

HELPER = test_macos_preview.SCRIPTS / 'notary-upload.py'


class UploadProcess(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='spaceport-upload-')
        self.addCleanup(self.temp.cleanup)
        self.receipt = Path(self.temp.name)

    def command(self, body, timeout='0.3', receipt=None):
        return [sys.executable, str(HELPER), '--timeout-seconds', timeout,
                str(receipt or self.receipt), '--', sys.executable, '-c', body]

    def test_existing_receipt_files_and_symlinks_prevent_upload_without_overwrite(self):
        for name in ('submission.json', 'submit.stderr'):
            for kind in ('file', 'symlink'):
                with self.subTest(name=name, kind=kind):
                    receipt = self.receipt / f'{name}-{kind}'
                    receipt.mkdir()
                    existing = receipt / name
                    if kind == 'symlink':
                        target = receipt / 'original'
                        target.write_bytes(b'retained receipt')
                        existing.symlink_to(target)
                    else:
                        existing.write_bytes(b'retained receipt')
                    marker = receipt / 'launched'
                    result = subprocess.run(self.command(
                        f'from pathlib import Path; Path({str(marker)!r}).touch()', receipt=receipt),
                        capture_output=True, text=True, timeout=8)
                    self.assertFalse(marker.exists(), 'Existing receipt must prevent child launch')
                    self.assertEqual(existing.read_bytes(), b'retained receipt')
                    self.assertEqual(existing.is_symlink(), kind == 'symlink')
                    self.assertEqual(result.returncode, 1, result.stderr)

    def test_timeout_retains_partial_receipts_and_reaps_only_owned_child(self):
        other = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'])
        self.addCleanup(other.wait)
        self.addCleanup(lambda: other.poll() is None and other.kill())
        body = ('import os, sys, time; print(os.getpid(), flush=True); '
                'print("partial diagnostic", file=sys.stderr, flush=True); time.sleep(30)')
        result = subprocess.run(self.command(body), capture_output=True, text=True, timeout=8)
        self.assertEqual(result.returncode, 124, result.stderr)
        pid = int((self.receipt / 'submission.json').read_text())
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)
        self.assertIsNone(other.poll())
        self.assertEqual((self.receipt / 'submit.stderr').read_text(), 'partial diagnostic\n')
        self.assertIn('outcome unknown; do not resubmit', result.stderr)

    def test_normal_and_nonzero_exit_preserve_receipts_and_status(self):
        for status in (0, 17):
            receipt = self.receipt / str(status)
            receipt.mkdir()
            result = subprocess.run(self.command(
                f'import sys; print("receipt"); print("diagnostic", file=sys.stderr); sys.exit({status})',
                receipt=receipt),
                capture_output=True, text=True, timeout=8)
            self.assertEqual(result.returncode, status, result.stderr)
            self.assertEqual((receipt / 'submission.json').read_text(), 'receipt\n')
            self.assertEqual((receipt / 'submit.stderr').read_text(), 'diagnostic\n')

    def test_timeout_kills_and_reaps_child_that_ignores_termination(self):
        body = ('import os, signal, time; signal.signal(signal.SIGTERM, signal.SIG_IGN); '
                'print(os.getpid(), flush=True); time.sleep(30)')
        result = subprocess.run(self.command(body), capture_output=True, text=True, timeout=8)
        self.assertEqual(result.returncode, 124, result.stderr)
        with self.assertRaises(ProcessLookupError):
            os.kill(int((self.receipt / 'submission.json').read_text()), 0)

    def test_cancellation_reaps_upload_child(self):
        body = 'import os, time; print(os.getpid(), flush=True); time.sleep(30)'
        wrapper = subprocess.Popen(self.command(body, '30'), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.addCleanup(wrapper.wait)
        self.addCleanup(lambda: wrapper.poll() is None and wrapper.kill())
        receipt = self.receipt / 'submission.json'
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            if receipt.exists() and receipt.read_text().strip():
                break
            if wrapper.poll() is not None:
                break
            time.sleep(0.01)
        self.assertTrue(receipt.exists() and receipt.read_text().strip())
        wrapper.send_signal(signal.SIGTERM)
        wrapper.communicate(timeout=8)
        self.assertEqual(wrapper.returncode, 143)
        with self.assertRaises(ProcessLookupError):
            os.kill(int(receipt.read_text()), 0)


class UploadSigning(unittest.TestCase):
    setUp = test_macos_preview.PreviewSigning.setUp
    write_plist = test_macos_preview.PreviewSigning.write_plist
    calls = test_macos_preview.PreviewSigning.calls
    sign = test_macos_preview.PreviewSigning.sign

    def test_uploads_use_standard_endpoint_and_report_safe_stages(self):
        result = self.sign(success=True)
        submits = [call for call in self.calls() if call[1:3] == ['notarytool', 'submit']]
        self.assertTrue(all('--no-s3-acceleration' in call for call in submits))
        for stage in ('app upload starting', 'dmg upload starting',
                      f'app submission {test_macos_preview.UUID}', 'app processing Accepted',
                      'app stapling', 'DMG creation', 'DMG signing'):
            self.assertIn(stage, result.stderr)
        self.assertNotIn('dummy', result.stderr)

    def test_uncertain_upload_never_waits_or_submits_again_even_with_uuid(self):
        # A controlled helper failure checks the signer's uncertainty boundary.
        # UploadProcess above exercises the real subprocess deadline and cleanup.
        self.sign_script.with_name('notary-upload.py').write_text(
            'import pathlib, sys\n'
            'receipt = pathlib.Path(sys.argv[1])\n'
            f'(receipt / "submission.json").write_text({json.dumps({"id": test_macos_preview.UUID})!r})\n'
            '(receipt / "submit.stderr").write_text("partial upload")\n'
            'sys.exit(124)\n')
        result = self.sign()
        self.assertIn('outcome unknown; do not resubmit', result.stderr)
        calls = self.calls()
        self.assertFalse(any(call[1:3] == ['notarytool', 'wait'] for call in calls))
        self.assertFalse(any(call[1:3] == ['stapler', 'staple'] for call in calls))
        self.assertFalse(any(call[0] == 'hdiutil' for call in calls))
        self.assertIn('receipt already exists', self.sign().stderr)


if __name__ == '__main__':
    unittest.main()
