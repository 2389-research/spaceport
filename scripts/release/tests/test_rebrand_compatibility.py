"""Windows packaging declares Spaceport's own installation identity."""
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


class WindowsIdentity(unittest.TestCase):
    def test_installer_declares_spaceports_own_identity(self):
        text = INSTALLER.SCRIPT.read_text()
        for line in ('#define AppName "Spaceport"',
                     '#define StartupName "Spaceport"',
                     'AppPublisher=2389 Research, Inc.',
                     r'DefaultDirName={localappdata}\Programs\Spaceport'):
            self.assertIn(line, text)
        self.assertEqual(INSTALLER.app_id(), "{62B8722B-1FA2-48E0-9609-46AA5D431983}")
        # The smoke test seeds and checks the same value name.
        self.assertEqual(INSTALLER.STARTUP_NAME, INSTALLER.define("StartupName"))
        startup = [line for line in text.splitlines() if line.startswith("Root:") and
                   ("CurrentVersion\\Run" in line or "StartupApproved\\Run" in line)]
        self.assertTrue(startup)
        for line in startup:
            self.assertIn('ValueName: "{#StartupName}"', line)
            self.assertIn("uninsdeletevalue dontcreatekey", line)
            self.assertNotIn("ValueData:", line)

    def test_installer_leaves_the_inherited_installation_alone(self):
        text = INSTALLER.SCRIPT.read_text()
        # Setup finds an installation by its AppId and replaces files in its own
        # directory, so an inherited Sheprd install keeps all of these.
        for inherited in ("{7FA70894-4FC0-4923-BDCB-B05F7475610A}",
                          r"Programs\Herdr UI", "AppPublisher=Herdr UI",
                          "Sheprd.lnk", "Herdr UI.lnk"):
            with self.subTest(inherited=inherited):
                self.assertNotIn(inherited, text)
        # Renaming or copying another installation's shortcuts would adopt it.
        self.assertNotIn("RenameLegacyShortcut", text)

    def test_every_packaged_executable_is_installed(self):
        text = INSTALLER.SCRIPT.read_text()
        for name in PACKAGE.EXECUTABLES:
            definition = {"spaceport.exe": "{#AppExe}", "spaceport-launcher.exe": "{#Launcher}",
                          }.get(name, name)
            self.assertIn(f'Source: "{{#SourceDir}}\\{definition}";', text)

    def test_native_smoke_preserves_a_disabled_startup_entry(self):
        launcher = Path("C:/Programs/Spaceport/spaceport-launcher.exe")
        values = {INSTALLER.STARTUP_KEYS[0]: {INSTALLER.STARTUP_NAME: (f'"{launcher}"', 1)},
                  INSTALLER.STARTUP_KEYS[1]: {INSTALLER.STARTUP_NAME: (bytes([3]) + bytes(11), 3)}}

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
            values[INSTALLER.STARTUP_KEYS[1]][INSTALLER.STARTUP_NAME] = (bytes([2]) + bytes(11), 3)
            with self.assertRaisesRegex(ValueError, "disabled state"):
                INSTALLER.check_startup_preserved(launcher)
            values[INSTALLER.STARTUP_KEYS[1]][INSTALLER.STARTUP_NAME] = (bytes([3]) + bytes(11), 3)
            values[INSTALLER.STARTUP_KEYS[0]][INSTALLER.INHERITED_STARTUP_NAME] = (f'"{launcher}"', 1)
            with self.assertRaisesRegex(ValueError, "second startup identity"):
                INSTALLER.check_startup_preserved(launcher)


if __name__ == "__main__":
    unittest.main()
