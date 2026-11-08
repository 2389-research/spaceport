"""ABOUTME: Guards against the excluded cloud setup worker and UI route.
ABOUTME: Checks that retired provider settings and connector code are absent.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


def source(path):
    return (ROOT / path).read_text()


class CloudSetupAbsence(unittest.TestCase):
    def test_window_tick_cannot_launch_cloud_setup(self):
        self.assertFalse("mod cloud_setup" in source("crates/sheprd-gpui/src/main.rs"))
        self.assertFalse("poll_cloud_setup" in source("crates/sheprd-gpui/src/window.rs"))
        self.assertFalse((ROOT / "crates/sheprd-gpui/src/cloud_setup.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-gpui/src/cloud_setup/agents.rs").exists())

    def test_menu_and_settings_do_not_offer_setup_execution(self):
        self.assertFalse("RunSetup" in source("crates/sheprd-gpui/src/menu/devices/host_menu.rs"))
        self.assertFalse("RunSetup" in source("crates/sheprd-gpui/src/menu/devices/host_menu/actions.rs"))
        self.assertFalse((ROOT / "crates/sheprd-gpui/src/settings_window/cloud_devices.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-gpui/src/config/cloud.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-gpui/src/config/container.rs").exists())

    def test_saved_connector_is_also_absent(self):
        self.assertFalse((ROOT / "crates/sheprd-remote/src/container/connect.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-remote/src/cloud/catalog.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/endpoint/catalog.rs").exists())


if __name__ == "__main__":
    unittest.main()
