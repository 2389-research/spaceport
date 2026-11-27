"""ABOUTME: Ensures Azure cloud provisioning has no desktop route.
ABOUTME: Preserves distinct Azure OpenAI display identity and Core features.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class AzureCloudSourceAbsence(unittest.TestCase):
    def test_provider_module_feature_and_helper_are_absent(self):
        self.assertFalse((ROOT / "crates/spaceport-remote/src/azure").exists())
        self.assertFalse((ROOT / "crates/spaceport-remote/src/cloud/relay.rs").exists())
        self.assertFalse((ROOT / "crates/spaceport-remote/src/credentials.rs").exists())
        self.assertFalse((ROOT / "crates/spaceport-control-protocol/src/protocol/cloud.rs").exists())
        self.assertFalse((ROOT / "crates/spaceport-gpui/src/settings_window/cloud_devices/azure.rs").exists())
        self.assertFalse((ROOT / "assets/icons/cloud/azure.svg").exists())
        for crate in ("spaceport-client", "spaceport-remote", "spaceport-usage", "spaceport-gpui"):
            manifest = ROOT / "crates" / crate / "Cargo.toml"
            self.assertNotIn("azure = [", manifest.read_text(), str(manifest))
        self.assertNotIn("crate::azure::sandbox::relay::run_helper", (ROOT / "crates/spaceport-gpui/src/app.rs").read_text())

    def test_cloud_host_routes_and_claims_are_absent_with_other_products_retained(self):
        for name in (
            "crates/spaceport-usage/src/model.rs",
            "crates/spaceport-gpui/src/listening_ports.rs",
        ):
            path = ROOT / name
            self.assertNotIn("Host::Azure", path.read_text(), name)
        for name in ("settings-cloud-azure", "error-remote-azure", "azure-sizes"):
            self.assertFalse((ROOT / f"crates/spaceport-i18n/locales/en-US/{name}.ftl").exists())
        for name in ("index.html", "assets/features.js"):
            self.assertNotIn("Azure", (ROOT / "website/site" / name).read_text(), name)
        self.assertTrue((ROOT / "crates/spaceport-gpui/src/menu/devices/add_device/core.rs").exists())
        self.assertFalse((ROOT / "crates/spaceport-remote/src/container.rs").exists())
        self.assertFalse((ROOT / "crates/spaceport-gpui/src/i18n/errors/orchestrator_azure.rs").exists())
        registry = (ROOT / "crates/spaceport-usage/src/registry.rs").read_text()
        self.assertIn('id: "azureopenai"', registry)
        self.assertIn('name: "Azure OpenAI"', registry)


if __name__ == "__main__":
    unittest.main()
