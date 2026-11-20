"""ABOUTME: Check the fork's compiler identity and exact startup ownership.
ABOUTME: Keep inherited installer state outside Spaceport's smoke contract.
"""
import importlib.util
from pathlib import Path
import sys
import types
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SCRIPTS = ROOT / "scripts/release"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


INSTALLER = load("windows_installer", SCRIPTS / "windows-installer.py")
PACKAGE = load("package_windows", SCRIPTS / "package-windows.py")


class RebrandCompatibility(unittest.TestCase):
    def test_compiler_and_smoke_read_the_runtime_identity(self):
        identity = (ROOT / "crates/sheprd-gpui/development-app-id").read_text().strip()
        arguments = INSTALLER.iscc_arguments("20261117.1", "x86_64-pc-windows-msvc", "src", "out")
        self.assertIn(f"/DAppUserModelID={identity}", arguments)
        self.assertEqual(INSTALLER.app_user_model_id(), identity)

    def test_every_packaged_executable_is_installed(self):
        text = INSTALLER.SCRIPT.read_text()
        for name in PACKAGE.EXECUTABLES:
            definition = {"spaceport.exe": "{#AppExe}", "spaceport-launcher.exe": "{#Launcher}",
                          }.get(name, name)
            self.assertIn(f'Source: "{{#SourceDir}}\\{definition}";', text)

    def test_smoke_checks_only_the_forks_disabled_startup_entry(self):
        launcher = Path("C:/Programs/Spaceport/spaceport-launcher.exe")
        values = {INSTALLER.STARTUP_KEYS[0]: {"Spaceport": (f'"{launcher}"', 1), "Sheprd": ("other", 1)},
                  INSTALLER.STARTUP_KEYS[1]: {"Spaceport": (bytes([3]) + bytes(11), 3), "Herdr UI": ("other", 1)}}

        def query(key, name):
            if name not in values[key]:
                raise FileNotFoundError(name)
            return values[key][name]

        def open_key(_, name):
            context = mock.MagicMock()
            context.__enter__.return_value = name
            return context

        registry = types.SimpleNamespace(HKEY_CURRENT_USER=0, REG_SZ=1, REG_BINARY=3,
                                         OpenKey=open_key, QueryValueEx=query)
        with mock.patch.dict(sys.modules, {"winreg": registry}):
            INSTALLER.check_startup_preserved(launcher)
            self.assertEqual(set(INSTALLER.startup_entries()), set(INSTALLER.STARTUP_KEYS))
            values[INSTALLER.STARTUP_KEYS[1]]["Spaceport"] = (bytes([2]) + bytes(11), 3)
            with self.assertRaisesRegex(ValueError, "disabled state"):
                INSTALLER.check_startup_preserved(launcher)
            del values[INSTALLER.STARTUP_KEYS[0]]["Spaceport"]
            del values[INSTALLER.STARTUP_KEYS[1]]["Spaceport"]
            self.assertEqual(INSTALLER.startup_entries(), [])
