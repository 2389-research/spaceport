"""ABOUTME: Ensures Azure cloud provisioning has no desktop route.
ABOUTME: Preserves distinct Azure OpenAI and Core features.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class AzureCloudSourceAbsence(unittest.TestCase):
    def test_provider_module_feature_and_helper_are_absent(self):
        self.assertFalse((ROOT / "crates/sheprd-remote/src/azure").exists())
        self.assertFalse((ROOT / "crates/sheprd-remote/src/cloud/relay.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-remote/src/credentials.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-control-protocol/src/protocol/cloud.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-gpui/src/settings_window/cloud_devices/azure.rs").exists())
        self.assertFalse((ROOT / "assets/icons/cloud/azure.svg").exists())
        for crate in ("sheprd-client", "sheprd-remote", "sheprd-usage", "sheprd-gpui"):
            manifest = ROOT / "crates" / crate / "Cargo.toml"
            self.assertNotIn("azure = [", manifest.read_text(), str(manifest))
        self.assertNotIn("crate::azure::sandbox::relay::run_helper", (ROOT / "crates/sheprd-gpui/src/app.rs").read_text())

    def test_cloud_host_routes_and_claims_are_absent_with_other_products_retained(self):
        for name in (
            "crates/sheprd-usage/src/model.rs",
            "crates/sheprd-gpui/src/system_load.rs",
            "crates/sheprd-gpui/src/system_load/render.rs",
            "crates/sheprd-gpui/src/listening_ports.rs",
        ):
            path = ROOT / name
            self.assertNotIn("Host::Azure", path.read_text(), name)
        for name in ("settings-cloud-azure", "error-remote-azure", "azure-sizes"):
            self.assertFalse((ROOT / f"crates/sheprd-gpui/locales/en-US/{name}.ftl").exists())
        for name in ("index.html", "assets/features.js"):
            self.assertNotIn("Azure", (ROOT / "website/site" / name).read_text(), name)
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/menu/devices/add_device/core.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-remote/src/container.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-gpui/src/i18n/errors/orchestrator_azure.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/locales/en-US/usage-azureopenai.ftl").exists())


if __name__ == "__main__":
    unittest.main()
