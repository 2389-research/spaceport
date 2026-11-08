"""ABOUTME: Guards against client-owned Git snapshots on window ticks.
ABOUTME: Checks the runtime, setting, and VCS worker routes together.
"""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
GUI = ROOT / "crates/sheprd-gpui"
VCS = ROOT / "crates/sheprd-vcs"


class CheckpointWorkerAbsence(unittest.TestCase):
    def test_window_tick_cannot_capture_a_checkout(self):
        routes = (
            ("src/window.rs", "update_checkpoints("),
            ("src/config.rs", "agent_checkpoints"),
            ("src/config/preferences.rs", "AgentCheckpoints"),
            ("src/settings_window/pages/general.rs", "general-checkpoints"),
        )
        for path, route in routes:
            with self.subTest(path=path):
                self.assertFalse(route in (GUI / path).read_text(), f"{path} still has {route}")
        self.assertFalse((VCS / "src/checkpoint.rs").exists(), "Git checkpoint worker remains")


if __name__ == "__main__":
    unittest.main()
