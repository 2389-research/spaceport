# ABOUTME: Guard the desktop control command's user-facing product name.
# ABOUTME: Leave internal Rust identifiers and source attribution untouched.

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[3]
CONTROL = ROOT / "crates/sheprd-gpui/src/control"
OLD_LABEL = re.compile(r'"[^"\n]*Sheprd\b')


class ControlProductCopy(unittest.TestCase):
    def test_runtime_control_text_uses_spaceport_name(self):
        stale = []
        for path in sorted(CONTROL.rglob("*.rs")):
            if "tests" in path.parts:
                continue
            for number, line in enumerate(path.read_text().splitlines(), 1):
                if line.lstrip().startswith("//"):
                    continue
                if OLD_LABEL.search(line):
                    stale.append(f"{path.relative_to(ROOT)}:{number}")
        self.assertEqual(stale, [])
