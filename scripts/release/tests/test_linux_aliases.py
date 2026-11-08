"""The Linux package and Nix wrapper expose one app command."""
import importlib.util
import io
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("linux_distro", ROOT / "scripts/release/package-linux-distro.py")
DISTRO = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DISTRO)


class LinuxAliasTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.work = Path(temporary.name)

    def archive(self, changes=None, duplicate=False):
        changes = changes or {}
        archive = self.work / "manual.tar.gz"
        with tarfile.open(archive, "w:gz") as output:
            for relative in sorted(DISTRO.TREE):
                entry = tarfile.TarInfo("release/" + relative)
                entry.mode = 0o755 if relative.startswith("bin/") else 0o644
                entry.type, entry.linkname = changes.get(relative, (tarfile.REGTYPE, ""))
                data = b"canonical executable" if relative == "bin/spaceport" else b"fixture"
                entry.size = len(data) if entry.isfile() else 0
                output.addfile(entry, io.BytesIO(data) if entry.isfile() else None)
                if duplicate and relative == "bin/spaceport":
                    output.addfile(entry, io.BytesIO(data))
        return archive

    def test_distro_keeps_only_one_app_executable(self):
        root = DISTRO.extract(self.archive(), "release", self.work / "tree")
        canonical = root / "bin/spaceport"
        self.assertTrue(canonical.is_file())
        self.assertFalse((root / "bin/herdr-gpui").exists())
        config = DISTRO.config("20261103.1", "x86_64-unknown-linux-gnu", root, "2026-11-03T00:00:00Z")
        entry = next(entry for entry in config["contents"] if entry["dst"] == "/usr/bin/spaceport")
        self.assertEqual(entry["type"], "file")
        self.assertEqual((entry["file_info"]["owner"], entry["file_info"]["group"]), ("root", "root"))

    def test_distro_rejects_symlinks_and_duplicate_app_entries(self):
        cases = [{"bin/spaceport": (tarfile.SYMTYPE, target)}
                 for target in ("/usr/bin/spaceport", "../bin/spaceport", "../../outside", "./sheprd")]
        cases += [{"bin/spaceport": (tarfile.LNKTYPE, "release/bin/spaceport")},
                  {"bin/spaceport": (tarfile.DIRTYPE, "")}]
        for changes in cases:
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                DISTRO.extract(self.archive(changes), "release", self.work / "tree")
            self.assertFalse((self.work / "tree").exists())
        with self.assertRaisesRegex(ValueError, "Duplicate archive entry"):
            DISTRO.extract(self.archive(duplicate=True), "release", self.work / "tree")

    def test_nix_wraps_only_the_spaceport_command(self):
        text = (ROOT / "flake.nix").read_text()
        flags = re.search(r"cargoBuildFlags = \[([^]]+)\];", text)[1]
        packages = re.findall(r'"([^"]+)"', flags)
        self.assertEqual(packages, ["-p", "sheprd-gpui", "-p", "sheprd-acp"])
        hook = re.search(r"postFixup = ''\n(.*?)\n\s*'';", text, re.S)[1]
        hook = re.sub(r"\$\{lib\.[^}]+\}", "/fixture", hook)
        binaries = self.work / "bin"
        binaries.mkdir()
        for command in ("spaceport", "sheprd-acp"):
            executable = binaries / command
            executable.write_text(f"#!/bin/sh\necho {command}\n")
            executable.chmod(0o755)
        core = binaries / "herdr-core"
        core.write_bytes(b"upstream unchanged")
        stubs = r'''
patchelf() { :; }
wrapProgram() {
    mv "$1" "$1.real"
    printf '#!/bin/sh\nprintf "wrapped:"\nexec "%s" "$@"\n' "$1.real" > "$1"
    chmod +x "$1"
}
'''
        subprocess.run(["bash", "-euc", stubs + hook], check=True,
                       env={**os.environ, "out": str(self.work)}, capture_output=True)
        self.assertEqual(subprocess.check_output([str(binaries / "spaceport")], text=True).strip(), "wrapped:spaceport")
        self.assertFalse((binaries / "sheprd").exists())
        self.assertFalse((binaries / "herdr-gpui").exists())
        self.assertEqual(core.read_bytes(), b"upstream unchanged")
