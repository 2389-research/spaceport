"""The Nix wrapper exposes one app command."""
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]


class LinuxAliasTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.work = Path(temporary.name)

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
