"""ABOUTME: Guards Spaceport against client-owned Container stop and destroy routes.
ABOUTME: Ensures no client engine mutation or catalog route remains.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


def source(path):
    return (ROOT / path).read_text()


class ContainerStopAbsence(unittest.TestCase):
    def test_normal_startup_does_not_install_idle_stopping(self):
        self.assertFalse("install_idle_checks" in source("crates/sheprd-gpui/src/app.rs"))
        self.assertFalse((ROOT / "crates/sheprd-remote/src/container.rs").exists())

    def test_saved_device_controls_cannot_stop_or_destroy_engine_state(self):
        for name in ("crates/sheprd-gpui/src/settings_window/cloud_devices/container.rs",
                     "crates/sheprd-remote/src/cloud/providers.rs",
                     "crates/sheprd-remote/src/container/setup.rs"):
            self.assertFalse((ROOT / name).exists(), name)
        self.assertFalse(
            (ROOT / "crates/sheprd-gpui/src/menu/devices/host_menu/destroy/tests.rs").exists()
        )

    def test_saved_row_forget_and_connector_are_absent(self):
        host_menu = source("crates/sheprd-gpui/src/menu/devices/host_menu.rs")
        self.assertNotIn("crate::cloud::remove", host_menu)
        self.assertFalse("deletion.run" in host_menu)
        self.assertNotIn("crate::cloud::connect", source("crates/sheprd-gpui/src/connection.rs"))


if __name__ == "__main__":
    unittest.main()
