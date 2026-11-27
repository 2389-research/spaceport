# ABOUTME: Exercises the task board guard as a JSON file consumer.
# ABOUTME: Checks output and exit status for bare records and nested envelopes.
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("board-guard.py")


class BoardGuardTest(unittest.TestCase):
    def run_guard(self, body, *args):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "body.json"
            path.write_text(body, encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(SCRIPT), str(path), *args],
                capture_output=True, text=True, timeout=2,
            )

    def test_unwrap_consumes_each_envelope(self):
        record = {"title": "Ready"}
        for value in (record, {"value": record}, {"revision": 2, "value": {"value": record}}):
            with self.subTest(value=value):
                result = self.run_guard(json.dumps(value), "--unwrap")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(json.loads(result.stdout), record)

    def test_refusal_has_no_output_record(self):
        for value in ({"revision": 1}, {"value": {"commit_revision": 2}}, {"value": "private text"},
                      [], None, 3):
            for args in ([], ["--unwrap"]):
                with self.subTest(value=value, args=args):
                    result = self.run_guard(json.dumps(value), *args)
                    self.assertEqual(result.returncode, 1)
                    self.assertEqual(result.stdout, "")
                    self.assertNotIn("private text", result.stderr)

    def test_malformed_json_stays_an_input_error(self):
        result = self.run_guard("{", "--unwrap")
        self.assertEqual(result.returncode, 2)
        self.assertEqual(result.stdout, "")


if __name__ == "__main__":
    unittest.main()
