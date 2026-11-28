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
from unittest.mock import patch

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
            "HOME", "APPDATA", "LOCALAPPDATA", "XDG_CONFIG_HOME", "XDG_STATE_HOME")}
        os.environ["APPDATA"] = str(self.home / "Roaming")
        os.environ["LOCALAPPDATA"] = str(self.home / "Local")
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

    def test_windows_shortcuts_and_install_directories_have_distinct_owners(self):
        sandbox = Sandbox(self)
        with patch.object(ISOLATION.sys, "platform", "win32"):
            rows = {row.name: row for row in ISOLATION.surfaces()}
        programs = sandbox.home / "Roaming/Microsoft/Windows/Start Menu/Programs"
        shortcut = programs / "Spaceport.lnk"
        programs.mkdir(parents=True)
        shortcut.write_bytes(b"owned")
        (programs / "Sheprd.lnk").write_bytes(b"inherited")
        self.assertEqual(rows["own-start-menu"].path, shortcut)
        self.assertEqual(rows["own-start-menu"].owner, "own")
        self.assertEqual(rows["start-menu"].path, programs)
        inherited, _ = ISOLATION.collect(rows["start-menu"])
        self.assertIn("Sheprd.lnk", inherited)
        self.assertNotIn("Spaceport.lnk", inherited)
        self.assertEqual(rows["own-windows-install"].path, sandbox.home / "Local/Programs/Spaceport")
        self.assertEqual(rows["windows-install-dir"].path, sandbox.home / "Local/Programs/Herdr UI")

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

    def test_equal_size_replacement_of_a_large_inherited_file_fails(self):
        sandbox = Sandbox(self)
        exe = sandbox.config / "herdr" / "inherited.exe"
        exe.parent.mkdir()
        exe.write_bytes(b"a" * (2**20 + 1))
        before = ISOLATION.snapshot()
        with exe.open("r+b") as target:
            target.write(b"b")
        self.assertEqual(ISOLATION.report(io.StringIO(), before, ISOLATION.snapshot()), 1)

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

    def test_fresh_install_and_uninstall_require_the_packaged_files_to_transition(self):
        sandbox = Sandbox(self)
        package = sandbox.home / "Spaceport"
        programs = sandbox.home / "Roaming/Microsoft/Windows/Start Menu/Programs"
        shortcut = programs / "Spaceport.lnk"

        def capture(name):
            result = io.StringIO()
            with contextlib.redirect_stdout(result):
                self.assertEqual(ISOLATION.main(["snapshot", "--package", str(package)]), 0)
            path = sandbox.home / f"{name}.json"
            path.write_text(result.getvalue())
            return str(path)

        def verify(before, installed, removed):
            with contextlib.redirect_stdout(io.StringIO()):
                return ISOLATION.main(["verify", before, installed, removed])

        with patch.object(ISOLATION.sys, "platform", "win32"):
            before = capture("before")
            self.assertEqual(verify(before, before, before), 1)
            package.mkdir()
            partial = capture("partial")
            self.assertEqual(verify(before, partial, before), 1)
            for name in ("spaceport.exe", "spaceport-launcher.exe", "sheprd-acp.exe"):
                (package / name).write_bytes(b"fixture")
            programs.mkdir(parents=True)
            shortcut.write_bytes(b"owned")
            installed = capture("installed")
            for item in package.iterdir():
                item.unlink()
            package.rmdir()
            leftover = capture("leftover")
            self.assertEqual(verify(before, installed, leftover), 1)
            shortcut.unlink()
            self.assertEqual(verify(before, installed, capture("removed")), 0)

    def test_bundle_and_nix_profile_use_their_own_installed_artifacts(self):
        sandbox = Sandbox(self)
        for platform, files in {
            "darwin": ["Contents/Info.plist", "Contents/MacOS/spaceport", "Contents/MacOS/sheprd-acp"],
            "linux": ["bin/spaceport", "bin/sheprd-acp", "share/applications/spaceport.development.desktop",
                      "share/icons/hicolor/scalable/apps/spaceport.svg"],
        }.items():
            with self.subTest(platform=platform), patch.object(ISOLATION.sys, "platform", platform):
                root = sandbox.home / platform
                before = ISOLATION.package_files(root)
                self.assertEqual(set(before.values()), {"absent"})
                for rel in files:
                    path = root / rel
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(b"fixture")
                self.assertEqual(set(ISOLATION.package_files(root).values()), {"installed"})
                for rel in files:
                    (root / rel).unlink()
                removed = ISOLATION.package_files(root)
                if platform == "darwin":
                    self.assertEqual(removed[str(root)], "installed")
                else:
                    self.assertEqual(removed, before)

    def test_qualification_requires_the_same_package_and_host_roots(self):
        sandbox = Sandbox(self)
        with patch.object(ISOLATION.sys, "platform", "win32"):
            before = ISOLATION.snapshot(sandbox.home / "A")
            elsewhere = ISOLATION.snapshot(sandbox.home / "B")
        with self.assertRaises(ISOLATION.Usage):
            ISOLATION.verify(io.StringIO(), before, elsewhere, before)
        with self.assertRaises(ISOLATION.Usage):
            ISOLATION.verify(io.StringIO(), ISOLATION.snapshot(), before, before)

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
