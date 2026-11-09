"""ABOUTME: Keeps the retired Orchestrator backend out of the workspace.
ABOUTME: Protects retained credential, avatar, and Core presentation owners.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]


class OrchestratorCrateAbsence(unittest.TestCase):
    def test_backend_has_no_workspace_or_package_route(self):
        self.assertFalse((ROOT / "crates/sheprd-orchestrator").exists())
        for manifest in [ROOT / "Cargo.toml", *(ROOT / "crates").glob("*/Cargo.toml")]:
            with self.subTest(manifest=manifest.relative_to(ROOT)):
                self.assertNotIn("sheprd-orchestrator", manifest.read_text())
        self.assertNotIn('name = "sheprd-orchestrator"', (ROOT / "Cargo.lock").read_text())

    def test_retained_owners_stay_present(self):
        for relative in (
            "crates/sheprd-credentials/src/lib.rs",
            "crates/sheprd-gpui/src/avatars/cache.rs",
            "crates/sheprd-github/src/pull_request/cache.rs",
            "crates/sheprd-gpui/src/core_terminals.rs",
        ):
            with self.subTest(path=relative):
                self.assertTrue((ROOT / relative).is_file())


if __name__ == "__main__":
    unittest.main()
