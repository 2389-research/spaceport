"""ABOUTME: Guards the full retirement of the client-owned cloud connector.
ABOUTME: Keeps Core machine onboarding and manual SSH paths in the source graph.
"""

from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[3]


class CloudConnectorAbsence(unittest.TestCase):
    def test_no_provider_feature_or_saved_connector_remains(self):
        for crate in ("spaceport-client", "spaceport-remote", "spaceport-gpui", "spaceport-usage"):
            features = tomllib.loads((ROOT / "crates" / crate / "Cargo.toml").read_text())[
                "features"
            ]
            self.assertNotIn("cloud", features, crate)
            self.assertNotIn("container", features, crate)
        for name in (
            "crates/spaceport-remote/src/cloud.rs",
            "crates/spaceport-remote/src/container.rs",
            "crates/spaceport-gpui/src/cloud.rs",
            "crates/spaceport-gpui/src/settings_window/cloud_devices.rs",
            "assets/icons/cloud/container.svg",
        ):
            self.assertFalse((ROOT / name).exists(), name)

    def test_no_catalog_or_transport_can_select_old_provider_rows(self):
        for name in (
            "crates/spaceport-client/src/discovery.rs",
            "crates/spaceport-gpui/src/endpoint/catalog.rs",
            "crates/spaceport-gpui/src/connection.rs",
            "crates/spaceport-gpui/src/menu/devices/rail.rs",
            "crates/spaceport-gpui/src/menu/devices/host_menu.rs",
        ):
            source = (ROOT / name).read_text()
            with self.subTest(path=name):
                self.assertNotIn("CloudProvider", source)
                self.assertNotIn("ConnectTarget::Cloud", source)
                self.assertNotIn("crate::cloud", source)
        self.assertTrue((ROOT / "crates/spaceport-gpui/src/menu/devices/add_device/core.rs").exists())
        self.assertTrue((ROOT / "crates/spaceport-gpui/src/menu/devices/core_catalog.rs").exists())
        self.assertTrue((ROOT / "crates/spaceport-gpui/src/menu/devices/add_device.rs").exists())


if __name__ == "__main__":
    unittest.main()
