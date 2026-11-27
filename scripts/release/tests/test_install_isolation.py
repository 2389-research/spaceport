"""ABOUTME: Checks the install-isolation snapshot and compare tool.
ABOUTME: Keeps the owned and inherited surface rules honest without an install.
"""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import os
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "install_isolation", ROOT / "scripts/release/install-isolation.py")
ISOLATION = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ISOLATION)


class Sandbox:
    """A private home, configuration root and state root for one test."""

    def __init__(self, test):
        self.temp = tempfile.TemporaryDirectory(prefix="install-isolation-")
        test.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name) / "home"
        self.config = Path(self.temp.name) / "config"
        self.state = Path(self.temp.name) / "state"
        for path in (self.home, self.config, self.state):
            path.mkdir()
        self.saved = {name: os.environ.get(name) for name in (
            "HOME", "XDG_CONFIG_HOME", "XDG_STATE_HOME")}
        os.environ["HOME"] = str(self.home)
        os.environ["XDG_CONFIG_HOME"] = str(self.config)
        os.environ["XDG_STATE_HOME"] = str(self.state)
        test.addCleanup(self.restore)

    def restore(self):
        for name, value in self.saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value

    def surface(self, name):
        return next(surface for surface in ISOLATION.surfaces() if surface.name == name)


class SurfaceTable(unittest.TestCase):
    def test_spaceport_state_is_its_own_directory(self):
        Sandbox(self)
        own = self.entry("own-state").path
        inherited = self.entry("herdr-state").path
        self.assertEqual(own.name, "spaceport")
        self.assertNotEqual(own, inherited)
        self.assertFalse(str(own).startswith(str(inherited)))

    def test_the_old_state_root_is_watched_and_absent(self):
        Sandbox(self)
        legacy = self.entry("legacy-app-state")
        self.assertEqual(legacy.owner, "inherited")
        self.assertFalse(legacy.path.exists())

    def entry(self, name):
        return next(surface for surface in ISOLATION.surfaces() if surface.name == name)

    def test_roots_follow_the_xdg_variables(self):
        sandbox = Sandbox(self)
        self.assertEqual(self.entry("own-state").path.parent, sandbox.state)
        self.assertEqual(self.entry("own-config").path.parent, sandbox.config)


class Snapshot(unittest.TestCase):
    def test_own_entries_are_hashed_and_daemon_files_are_left_out(self):
        sandbox = Sandbox(self)
        own = sandbox.state / "spaceport"
        own.mkdir()
        (own / "window-state.json").write_text('{"x":0}')
        inherited = sandbox.config / "herdr"
        inherited.mkdir()
        (inherited / "config.toml").write_text("onboarding = false\n")
        (inherited / "herdr-server.log").write_text("noise\n")
        (inherited / "session.json").write_text("{}\n")
        document = ISOLATION.snapshot()
        surfaces = document["surfaces"]
        self.assertIn("window-state.json", surfaces["own-state"]["entries"])
        self.assertEqual(surfaces["own-state"]["entries"]["window-state.json"]["kind"], "file")
        self.assertIn("config.toml", surfaces["herdr-config"]["entries"])
        self.assertNotIn("herdr-server.log", surfaces["herdr-config"]["entries"])
        self.assertNotIn("session.json", surfaces["herdr-config"]["entries"])
        self.assertEqual(surfaces["herdr-config"]["ignored"], 2)

    def test_a_snapshot_records_no_file_contents(self):
        sandbox = Sandbox(self)
        secret = sandbox.state / "spaceport"
        secret.mkdir()
        (secret / "config.toml").write_text("token = 'hunter2'\n")
        text = json.dumps(ISOLATION.snapshot())
        self.assertNotIn("hunter2", text)

    def test_service_name_filter_keeps_unrelated_login_items_out(self):
        sandbox = Sandbox(self)
        agents = sandbox.home / "Library" / "LaunchAgents"
        agents.mkdir(parents=True)
        (agents / "com.example.updater.plist").write_text("x")
        (agents / "com.example.herdr-helper.plist").write_text("y")
        surface = next(
            (surface for surface in ISOLATION.surfaces() if surface.name == "launch-agents"),
            None,
        )
        if surface is None:
            self.skipTest("this platform registers no login items")
        entries, skipped = ISOLATION.collect(surface)
        self.assertIn("com.example.herdr-helper.plist", entries)
        self.assertNotIn("com.example.updater.plist", entries)
        self.assertEqual(skipped, 1)


class Compare(unittest.TestCase):
    def document(self, **surfaces):
        recorded = {
            name: {
                "owner": owner,
                "path": f"/{name}",
                "note": "",
                "present": True,
                "ignored": 0,
                "entries": {".": {"kind": "dir"}, **entries},
            }
            for name, (owner, entries) in surfaces.items()
        }
        return {"format": ISOLATION.FORMAT, "platform": "test", "surfaces": recorded}

    def run_compare(self, before, after):
        out = io.StringIO()
        return ISOLATION.report(out, before, after), out.getvalue()

    def test_spaceports_own_change_is_expected(self):
        before = self.document(**{"own-state": ("own", {})})
        after = self.document(**{"own-state": ("own", {"logs/spaceport.jsonl": {"kind": "file"}})})
        code, output = self.run_compare(before, after)
        self.assertEqual(code, 0)
        self.assertIn("changed (own)", output)

    def test_an_inherited_change_fails_and_names_the_path(self):
        before = self.document(**{"herdr-state": ("inherited", {})})
        after = self.document(**{
            "herdr-state": ("inherited", {"client/endpoints.json": {"kind": "file"}})})
        code, output = self.run_compare(before, after)
        self.assertEqual(code, 1)
        self.assertIn("CHANGED (inherited)", output)
        self.assertIn("client/endpoints.json", output)

    def test_a_surface_that_appears_on_an_inherited_root_fails(self):
        before = self.document(**{"herdr-state": ("inherited", {})})
        after = self.document(**{"herdr-state": ("inherited", {})})
        after["surfaces"]["legacy-app-state"] = {
            "owner": "inherited",
            "path": "/legacy",
            "note": "",
            "present": True,
            "ignored": 0,
            "entries": {".": {"kind": "dir"}},
        }
        code, output = self.run_compare(before, after)
        self.assertEqual(code, 1)
        self.assertIn("legacy-app-state", output)

    def test_a_file_whose_digest_changed_fails(self):
        before = self.document(**{
            "herdr-config": ("inherited", {"config.toml": {"kind": "file", "sha256": "a"}})})
        after = self.document(**{
            "herdr-config": ("inherited", {"config.toml": {"kind": "file", "sha256": "b"}})})
        code, output = self.run_compare(before, after)
        self.assertEqual(code, 1)
        self.assertIn("changed config.toml", output)

    def test_a_shared_change_is_reported_without_failing(self):
        before = self.document(**{"shared-catalog": ("shared", {})})
        after = self.document(**{
            "shared-catalog": ("shared", {"endpoint-selection.json": {"kind": "file"}})})
        code, output = self.run_compare(before, after)
        self.assertEqual(code, 0)
        self.assertIn("shared (shared)", output)
        self.assertIn("1 shared surface(s)", output)

    def test_unchanged_snapshots_pass(self):
        document = self.document(**{"herdr-config": ("inherited", {"config.toml": {"kind": "file"}})})
        code, output = self.run_compare(document, document)
        self.assertEqual(code, 0)
        self.assertIn("unchanged", output)


class CommandLine(unittest.TestCase):
    def refused(self, path):
        """The exit code of comparing `path` with itself, without its message."""
        errors = io.StringIO()
        with contextlib.redirect_stderr(errors):
            code = ISOLATION.main(["compare", str(path), str(path)])
        self.assertIn("install-isolation:", errors.getvalue())
        return code

    def test_an_unreadable_snapshot_is_a_usage_error(self):
        with tempfile.TemporaryDirectory(prefix="install-isolation-") as temp:
            self.assertEqual(self.refused(Path(temp) / "missing.json"), 2)

    def test_a_foreign_json_file_is_refused(self):
        with tempfile.TemporaryDirectory(prefix="install-isolation-") as temp:
            foreign = Path(temp) / "other.json"
            foreign.write_text('{"format": 99}')
            self.assertEqual(self.refused(foreign), 2)

    def test_snapshot_writes_json(self):
        Sandbox(self)
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            self.assertEqual(ISOLATION.main(["snapshot"]), 0)
        document = json.loads(buffer.getvalue())
        self.assertEqual(document["format"], ISOLATION.FORMAT)
        self.assertIn("own-state", document["surfaces"])


if __name__ == "__main__":
    unittest.main()
