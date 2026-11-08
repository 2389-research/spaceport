"""site/install.ps1: its Start menu shortcut, taskbar identity, and Explorer entry."""
from pathlib import Path
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
INSTALL = ROOT / "site/install.ps1"
ISS = ROOT / "scripts/release/sheprd.iss"


def app_id():
    identity = (ROOT / "crates/sheprd-updater/src/lib.rs").read_text(encoding="utf-8")
    return re.search(r'^pub const APP_ID: &str = "([^"]+)";$', identity, re.M).group(1)


def snippet():
    """The script's Add-Type block and the call that sets the shortcut's ID."""
    source = INSTALL.read_text(encoding="utf-8")
    start = source.index("    Add-Type -TypeDefinition")
    end = source.index("\n", source.index("[HerdrShortcut]::SetAppUserModelId(", start))
    return source[start:end]


def install_steps():
    """Everything the script does once the release's files are in place."""
    source = INSTALL.read_text(encoding="utf-8")
    start = source.index("  $hasLauncher = Test-Path")
    end = source.index("\n  }\n", source.index("Start it from the Start menu (Sheprd) or with sheprd.", start))
    return source[start:end + len("\n  }")]


class InstallScriptTests(unittest.TestCase):
    def test_shortcut_gets_the_apps_and_installers_user_model_id(self):
        install = INSTALL.read_text(encoding="utf-8")
        self.assertIn(f"[HerdrShortcut]::SetAppUserModelId($shortcut, '{app_id()}')", install)
        iss = ISS.read_text(encoding="utf-8")
        self.assertIn(f'#define AppUserModelID "{app_id()}"\n', iss)

    @unittest.skipUnless(sys.platform == "win32" and shutil.which("powershell.exe"), "needs Windows PowerShell")
    def test_snippet_writes_the_id_windows_reads_back(self):
        with tempfile.TemporaryDirectory(prefix="herdr-install-ps1-") as temp:
            script = Path(temp) / "check.ps1"
            script.write_text(
                "$ErrorActionPreference = 'Stop'\n"
                "$shortcut = Join-Path $env:HERDR_TEST_DIR 'Sheprd.lnk'\n"
                "$link = (New-Object -ComObject WScript.Shell).CreateShortcut($shortcut)\n"
                "$link.TargetPath = Join-Path $env:SystemRoot 'notepad.exe'\n"
                "$link.Save()\n"
                + snippet() + "\n"
                "(New-Object -ComObject Shell.Application).Namespace($env:HERDR_TEST_DIR)"
                ".ParseName('Sheprd.lnk').ExtendedProperty('System.AppUserModel.ID')\n",
                encoding="utf-8")
            result = subprocess.run(
                ["powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
                 "-File", str(script)],
                capture_output=True, text=True, timeout=120,
                env={**os.environ, "HERDR_TEST_DIR": temp})
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.strip(), app_id())

    def test_installer_does_not_start_or_stop_scheduler(self):
        for source in (INSTALL, ISS):
            with self.subTest(path=source):
                self.assertNotIn("Automate", source.read_text(encoding="utf-8"))

    @unittest.skipUnless(sys.platform == "win32" and shutil.which("powershell.exe"), "needs Windows PowerShell")
    def test_shortcut_and_explorer_entry_leave_other_link_schemes_alone(self):
        # Releases up to 20261016.1 ship spaceport.exe alone (#632): there
        # the shortcut opens it and an existing Explorer entry goes away.
        for has_launcher in (False, True):
            key = rf"Software\HerdrInstallPs1Test-{os.getpid()}-{has_launcher}"
            with self.subTest(has_launcher=has_launcher), \
                    tempfile.TemporaryDirectory(prefix="herdr-install-ps1-") as temp:
                dest = Path(temp) / "Sheprd"
                dest.mkdir()
                launcher = dest / "spaceport-launcher.exe"
                (dest / "spaceport.exe").write_bytes(b"")
                if has_launcher:
                    launcher.write_bytes(b"")
                script = Path(temp) / "check.ps1"
                script.write_text("\n".join([
                    "$ErrorActionPreference = 'Stop'",
                    "$version = '20000101.1'",
                    "$dest = $env:HERDR_TEST_DEST",
                    "$exe = Join-Path $dest 'spaceport.exe'",
                    "$launcher = Join-Path $dest 'spaceport-launcher.exe'",
                    rf"$verbs = 'HKCU:\{key}\Directory', 'HKCU:\{key}\Background'",
                    rf"$scheme = 'HKCU:\{key}\Scheme'",
                    rf"$schemes = @('HKCU:\{key}\Scheme', 'HKCU:\{key}\LegacyScheme')",
                    "$ownedShortcuts = @()",
                    # Other handlers must survive either install shape.
                    r'foreach ($verb in $verbs) { New-Item -Force -Path "$verb\command" | Out-Null }',
                    r'foreach ($scheme in $schemes) { New-Item -Force -Path "$scheme\shell\open\command" | Out-Null; Set-Item -LiteralPath "$scheme\shell\open\command" -Value "other-app" }',
                    install_steps().replace("[Environment]::GetFolderPath('Programs')", "$env:HERDR_TEST_DIR"),
                    "'target=' + (New-Object -ComObject WScript.Shell).CreateShortcut($shortcut).TargetPath",
                    "foreach ($verb in $verbs) {",
                    r"""  if (Test-Path -LiteralPath $verb) { 'verb=' + (Get-Item -LiteralPath "$verb\command").GetValue('') }""",
                    "}",
                    r"""foreach ($scheme in $schemes) { if (Test-Path -LiteralPath $scheme) { 'scheme=' + (Get-Item -LiteralPath "$scheme\shell\open\command").GetValue('') } }""",
                    "",
                ]), encoding="utf-8")
                try:
                    result = subprocess.run(
                        ["powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
                         "-File", str(script)],
                        capture_output=True, text=True, timeout=120,
                        env={**os.environ, "HERDR_TEST_DIR": temp, "HERDR_TEST_DEST": str(dest)})
                finally:
                    subprocess.run(["reg.exe", "delete", rf"HKCU\{key}", "/f"], capture_output=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                lines = result.stdout.splitlines()
                message = next(line for line in lines if line.startswith("Installed Sheprd"))
                target = next(line for line in lines if line.startswith("target="))[len("target="):]
                verbs = [line[len("verb="):] for line in lines if line.startswith("verb=")]
                schemes = [line[len("scheme="):] for line in lines if line.startswith("scheme=")]
                # samefile, since Windows can hand back the temp directory
                # as an 8.3 short path.
                self.assertTrue(os.path.samefile(target, launcher if has_launcher else dest / "spaceport.exe"),
                                target)
                if has_launcher:
                    self.assertEqual(len(verbs), 2, result.stdout)
                    for verb in verbs:
                        program, rest = verb[1:].split('"', 1)
                        self.assertTrue(os.path.samefile(program, launcher), verb)
                        self.assertEqual(rest, ' open "%V"')
                    self.assertIn("spaceport-launcher.exe", message)
                    self.assertIn("Open in Sheprd", message)
                else:
                    self.assertEqual(verbs, [], result.stdout)
                    self.assertNotIn("launcher", message)
                    self.assertNotIn("Open in Sheprd", message)
                self.assertEqual(schemes, ["other-app", "other-app"], result.stdout)


if __name__ == "__main__":
    unittest.main()
