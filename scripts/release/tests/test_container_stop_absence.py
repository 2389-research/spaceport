"""ABOUTME: Guards Spaceport against client-owned Container stop and destroy routes.
ABOUTME: Keeps exact saved-row forgetting separate from engine mutation.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


def source(path):
    return (ROOT / path).read_text()


class ContainerStopAbsence(unittest.TestCase):
    def test_normal_startup_does_not_install_idle_stopping(self):
        self.assertFalse("install_idle_checks" in source("crates/sheprd-gpui/src/app.rs"))
        self.assertFalse("mod idle" in source("crates/sheprd-remote/src/container.rs"))

    def test_saved_device_controls_cannot_stop_or_destroy_engine_state(self):
        settings = source("crates/sheprd-gpui/src/settings_window/cloud_devices/container.rs")
        provider = source("crates/sheprd-remote/src/cloud/providers.rs")
        setup = source("crates/sheprd-remote/src/container/setup.rs")
        self.assertFalse("cloud-container-stop-" in settings)
        self.assertFalse("deletion:" in provider)
        self.assertFalse("pub fn deletion" in provider)
        self.assertFalse("destroy_saved" in setup)
        self.assertFalse("stop_saved" in setup)
        self.assertFalse(
            (ROOT / "crates/sheprd-gpui/src/menu/devices/host_menu/destroy/tests.rs").exists()
        )

    def test_saved_row_forget_and_connector_are_separate_pending_routes(self):
        host_menu = source("crates/sheprd-gpui/src/menu/devices/host_menu.rs")
        self.assertIn("crate::cloud::remove(provider, &id)?", host_menu)
        self.assertFalse("deletion.run" in host_menu)
        self.assertIn("crate::cloud::connect(target, stop)", source("crates/sheprd-gpui/src/connection.rs"))


if __name__ == "__main__":
    unittest.main()
