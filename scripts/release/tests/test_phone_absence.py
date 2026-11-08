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

    def test_desktop_has_no_phone_start_or_control_route(self):
        app = (ROOT / "crates/sheprd-gpui/src/app.rs").read_text()
        main = (ROOT / "crates/sheprd-gpui/src/main.rs").read_text()
        cli = (ROOT / "crates/sheprd-gpui/src/cli/control.rs").read_text()
        protocol = (ROOT / "crates/sheprd-control-protocol/src/protocol.rs").read_text()
        commands = (ROOT / "crates/sheprd-commands/src/lib.rs").read_text()
        self.assertFalse("remote::Supervisor::install" in app, "app starts phone supervisor")
        self.assertFalse("mod remote;" in main, "desktop links phone UI")
        self.assertFalse('"remote" => ControlCommand::Remote' in cli, "phone CLI remains")
        self.assertFalse("Remote(Call<remote::RemoteOp>)" in protocol, "phone IPC remains")
        self.assertFalse("RemotePair" in commands, "phone pairing command remains")

if __name__ == "__main__":
    unittest.main()
