"""ABOUTME: Checks that Container has no new machine or image entry route.
ABOUTME: Keeps saved device and Core machine paths visible for later cuts.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


def source(path):
    return (ROOT / path).read_text()


class ContainerProvisioningAbsence(unittest.TestCase):
    def test_container_creation_and_adoption_are_unreachable(self):
        settings = source("crates/sheprd-gpui/src/settings_window/cloud_devices/container.rs")
        for name in ("create_container", "adopt_container", "cloud-container-new", "cloud-container-adopt"):
            with self.subTest(name=name):
                self.assertFalse(name in settings, name)
        self.assertNotIn(
            "provision: Some(crate::container::setup::provision)",
            source("crates/sheprd-remote/src/cloud/providers.rs"),
        )
        setup = source("crates/sheprd-remote/src/container/setup.rs")
        for name in ("pub fn add_device(", "pub fn adopt(", "pub fn provision("):
            with self.subTest(name=name):
                self.assertFalse(name in setup, name)
        self.assertNotIn("dispatch_sandboxes", source("crates/sheprd-gpui/src/dispatch/hosts.rs"))
        self.assertNotIn("Destination::Sandbox", source("crates/sheprd-gpui/src/menu/workspace/dispatch.rs"))

    def test_container_image_entry_points_are_unreachable(self):
        settings = source("crates/sheprd-gpui/src/settings_window/cloud_devices/container.rs")
        self.assertNotIn("render_sandbox_images", settings)
        for name in ("settings-cloud-container-image", "settings-cloud-container-cpus", "settings-cloud-container-memory"):
            with self.subTest(name=name):
                self.assertFalse(name in settings, name)
        self.assertNotIn(
            "image: Some(&crate::container::image::SUPPORT)",
            source("crates/sheprd-remote/src/cloud/providers.rs"),
        )
        self.assertNotIn(
            "build_workspace_sandbox_images(window, cx)",
            source("crates/sheprd-gpui/src/menu/workspace.rs"),
        )

    def test_saved_device_and_core_paths_remain(self):
        self.assertIn("CloudProvider::$variant => crate::$module::connect", source("crates/sheprd-remote/src/cloud/providers.rs"))
        self.assertIn("pub fn connect(", source("crates/sheprd-remote/src/container/connect.rs"))
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/menu/devices/add_device/core.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/menu/devices/core_catalog.rs").exists())

    def test_cli_control_and_startup_have_no_creation_route(self):
        for folder in (
            "crates/sheprd-gpui/src/control",
            "crates/sheprd-control-protocol/src",
        ):
            with self.subTest(folder=folder):
                self.assertFalse(
                    any("Container" in path.read_text() for path in (ROOT / folder).rglob("*.rs"))
                )
        app = source("crates/sheprd-gpui/src/app.rs")
        self.assertNotIn("start_cloud_job", app)
        self.assertNotIn("install_idle_checks", app)


if __name__ == "__main__":
    unittest.main()
