"""ABOUTME: Checks that host readers cannot pull the removed migration engine into Spaceport.
ABOUTME: Preserves the existing host, chat, and pane routes without migration compatibility APIs.
"""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[3]
REMOTE = ROOT / "crates/sheprd-remote"
APP = ROOT / "crates/sheprd-gpui/src"


class TeleportEngineAbsence(unittest.TestCase):
    def test_migration_modules_and_compatibility_exports_are_absent(self):
        for relative in ("src/teleport.rs", "src/teleport"):
            self.assertFalse((REMOTE / relative).exists(), relative)
        self.assertFalse((APP / "teleport.rs").exists())
        for source in (*REMOTE.rglob("*.rs"), *APP.rglob("*.rs")):
            with self.subTest(source=source.relative_to(ROOT)):
                self.assertNotRegex(source.read_text(), r"\b(?:crate|sheprd_remote)::teleport\b")

    def test_host_readers_do_not_retain_migration_or_provisioning_apis(self):
        excluded = (
            "dispatch_worktree", "move_session", "handoff_prompt", "handoff_path",
            "handoff_resume_prompt", "rewrite_archive", "rewrite_paths", "remap_argv",
            "claude_project_dir", "pi_session_dir", "HostRepositories", "SourceState",
            "NewCheckout", "WorktreeCreated", "TRANSFER_OUTPUT", "TRANSFER_IDLE",
        )
        for source in (REMOTE / "src").rglob("*.rs"):
            text = source.read_text()
            for symbol in excluded:
                with self.subTest(source=source.relative_to(ROOT), symbol=symbol):
                    self.assertNotRegex(text, rf"\b{re.escape(symbol)}\b")
            for command in ("git commit-tree", "git worktree", "gh auth token", "refs/herdr-teleport"):
                self.assertNotIn(command, text, source)

    def test_retained_callers_use_the_narrow_shared_contract(self):
        for relative in ("chat/existing.rs", "pane_move.rs"):
            text = (APP / relative).read_text()
            self.assertIn("host_script::", text, relative)
        self.assertIn("agent::AgentKind", (APP / "overview/reopen.rs").read_text())
        processes = (APP / "processes.rs").read_text()
        self.assertIn("sheprd_remote::snapshot::ProcessInfo", processes)
        self.assertIn("info.foreground_pids()", processes)


if __name__ == "__main__":
    unittest.main()
