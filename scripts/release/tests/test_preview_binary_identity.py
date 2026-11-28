"""ABOUTME: Reads real compiled identity records through the preview package validator.
ABOUTME: Builds both signed channels with the same version, app ID and profile.
"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
RECORD = ROOT / "crates/spaceport-gpui/src/cli/build_info.rs"
CHECK = ROOT / "scripts/release/preview-identity.py"


class BinaryIdentity(unittest.TestCase):
    def test_preview_packaging_reads_the_channel_from_the_compiled_program(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "fixture.rs"
            source.write_text(f'#[path = r#"{RECORD}"#] mod record;\n'
                              'fn main() { print!("{}", record::build_info()); }\n')
            version = "20261128.1"
            for channel in ("macos-preview", "production"):
                with self.subTest(channel=channel):
                    binary = root / f"{channel}.exe"
                    env = {**os.environ, "SPACEPORT_APP_ID": "ai.2389.spaceport", "SPACEPORT_APP_VERSION": version,
                           "SPACEPORT_BUILD_PROFILE": "release", "SPACEPORT_BUILD_CHANNEL": channel,
                           "HERDR_BUILD_WORKTREE": "0", "HERDR_BUILD_BRANCH": "", "HERDR_BUILD_PR": ""}
                    subprocess.run(["rustc", "-O", source, "-o", binary], env=env, check=True)
                    result = subprocess.run([sys.executable, CHECK, version, "release", binary],
                                            capture_output=True, text=True)
                    self.assertEqual(result.returncode == 0, channel == "macos-preview", result.stderr)
                    if channel == "production":
                        self.assertIn("channel mismatch", result.stderr)
