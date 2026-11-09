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

    def test_provider_fetch_graph_is_absent(self):
        for path in ("providers", "providers.rs", "probe", "probe.rs", "cookies.rs", "service.rs"):
            with self.subTest(path=path):
                self.assertFalse((ROOT / "crates/sheprd-usage/src" / path).exists())
        usage = source("crates/sheprd-usage/src/lib.rs")
        for symbol in ("fn poll(", "fn refresh(", "fn allow(", "struct Worker", "struct Job"):
            self.assertNotIn(symbol, usage)

    def test_sign_in_config_is_absent(self):
        example = source("crates/sheprd-gpui/config-gpui.example.toml")
        for symbol in ("usage.providers", "browser_cookies", "OPENAI_ADMIN_KEY"):
            self.assertNotIn(symbol, example)
        manifest = source("crates/sheprd-usage/Cargo.toml")
        for dependency in ("aes", "cbc", "pbkdf2", "rusqlite", "secrecy", "sha1", "sha2", "ureq", "zeroize"):
            self.assertNotRegex(manifest, rf"(?m)^{dependency}\.workspace")

    def test_shared_host_and_usage_contract_remain(self):
        self.assertTrue((ROOT / "crates/sheprd-usage/src/model.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-usage/src/shell.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/control/server/hosts.rs").exists())
        self.assertTrue((ROOT / "crates/sheprd-gpui/src/usage/section.rs").exists())


if __name__ == "__main__":
    unittest.main()
