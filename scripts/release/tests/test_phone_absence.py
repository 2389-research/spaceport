"""ABOUTME: Checks that phone service routes stay out of the desktop build.
ABOUTME: Keeps local device tags and desktop control separate from phone sync.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class PhoneAbsenceTests(unittest.TestCase):
    def test_local_tags_have_no_phone_exchange(self):
        tags = (ROOT / "crates/sheprd-gpui/src/device_tags.rs").read_text()
        sidebar = (ROOT / "crates/sheprd-gpui/src/sidebar/tag_groups.rs").read_text()
        self.assertFalse("sheprd_remote_control" in tags, "local tags import the phone crate")
        self.assertFalse("mod sync;" in tags, "local tags still load daemon sync")
        self.assertFalse("DeviceTags::reach" in sidebar, "sidebar still starts tag sync")

if __name__ == "__main__":
    unittest.main()
