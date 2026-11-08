"""ABOUTME: Guards Spaceport's lack of client-owned AI provider probes.
ABOUTME: Keeps shared host and CLI usage routes for the Core-feed transition.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


def source(path):
    return (ROOT / path).read_text()


class ProviderProbeAbsence(unittest.TestCase):
    def test_startup_does_not_load_grants_or_poll_providers(self):
        self.assertFalse(
            "KeychainGrants::load" in source("crates/sheprd-gpui/src/app.rs")
        )
        self.assertFalse(
            "update_usage(cx)" in source("crates/sheprd-gpui/src/window.rs")
        )
        self.assertFalse(
            "self.usage.poll(" in source("crates/sheprd-gpui/src/window/monitors.rs")
        )

    def test_usage_ui_cannot_arm_client_probe_or_credential_read(self):
        for path in (
            "crates/sheprd-gpui/src/usage/section.rs",
            "crates/sheprd-gpui/src/usage/render.rs",
            "crates/sheprd-gpui/src/usage/panel.rs",
        ):
            with self.subTest(path=path):
                content = source(path)
                self.assertFalse("this.usage.refresh(" in content, path)
                self.assertFalse("this.usage.allow(" in content, path)
                self.assertFalse("KeychainGrants::grant(" in content, path)

    def test_shared_host_and_usage_contract_remain(self):
        self.assertTrue((ROOT / "crates/sheprd-usage/src/model.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-usage/src/probe/shell.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/control/server/hosts.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/usage/section.rs").exists())


if __name__ == "__main__":
    unittest.main()
