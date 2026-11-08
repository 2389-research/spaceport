"""Excluded scheduler binaries must not enter desktop build or package routes."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
MANIFESTS = (
    "Cargo.toml",
    "flake.nix",
    "site/install.ps1",
    "website/site/install.ps1",
    "website/herdrui/public/install.ps1",
    "Justfile",
    ".github/workflows/ci.yml",
    ".github/workflows/release.yml",
    "scripts/release/build-macos.sh",
    "scripts/release/generate-sbom.py",
    "scripts/release/package-linux.sh",
    "scripts/release/package-linux-distro.py",
    "scripts/release/package-macos.sh",
    "scripts/release/package-windows.py",
    "scripts/release/sheprd.iss",
    "scripts/release/sign-macos.sh",
    "scripts/release/smoke-linux-packages.sh",
    "scripts/release/windows-installer.py",
    "scripts/release/windows-signing.py",
)


class AutomationAbsence(unittest.TestCase):
    def test_scheduler_is_not_built_or_packaged(self):
        for relative in MANIFESTS:
            with self.subTest(path=relative):
                self.assertFalse(
                    "sheprd-automate" in (ROOT / relative).read_text(),
                    f"{relative} still builds or packages the scheduler",
                )


if __name__ == "__main__":
    unittest.main()
