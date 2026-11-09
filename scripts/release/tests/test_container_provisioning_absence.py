"""ABOUTME: Checks that Container has no new machine or image entry route.
ABOUTME: Keeps retained Core machine paths visible after the full cloud cut.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


def source(path):
    return (ROOT / path).read_text()


class ContainerProvisioningAbsence(unittest.TestCase):
    def test_container_creation_and_adoption_are_unreachable(self):
        settings = source("crates/sheprd-gpui/src/settings_window.rs")
        for name in ("create_container", "adopt_container", "cloud-container-new", "cloud-container-adopt"):
            with self.subTest(name=name):
                self.assertFalse(name in settings, name)
        self.assertNotIn(
            "provision: Some(crate::container::setup::provision)",
            source("crates/sheprd-remote/src/lib.rs"),
        )
        setup = source("crates/sheprd-remote/src/lib.rs")
        for name in ("pub fn add_device(", "pub fn adopt(", "pub fn provision("):
            with self.subTest(name=name):
                self.assertFalse(name in setup, name)
        for relative in ("dispatch/hosts.rs", "menu/workspace/dispatch.rs"):
            self.assertFalse((ROOT / "crates/sheprd-gpui/src" / relative).exists())
        for path in (ROOT / "crates/sheprd-gpui/src").rglob("*.rs"):
            self.assertNotIn("dispatch_sandboxes", path.read_text(), str(path))
            self.assertNotIn("Destination::Sandbox", path.read_text(), str(path))

    def test_container_image_entry_points_are_unreachable(self):
        settings = source("crates/sheprd-gpui/src/settings_window.rs")
        self.assertNotIn("render_sandbox_images", settings)
        for name in ("settings-cloud-container-image", "settings-cloud-container-cpus", "settings-cloud-container-memory"):
            with self.subTest(name=name):
                self.assertFalse(name in settings, name)
        self.assertNotIn(
            "image: Some(&crate::container::image::SUPPORT)",
            source("crates/sheprd-remote/src/lib.rs"),
        )
        self.assertNotIn(
            "build_workspace_sandbox_images(window, cx)",
            source("crates/sheprd-gpui/src/menu/workspace.rs"),
        )

    def test_core_paths_remain_without_saved_provider_connector(self):
        self.assertFalse((ROOT / "crates/sheprd-remote/src/cloud/providers.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-remote/src/container/connect.rs").exists())
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
