"""ABOUTME: Keeps the removed custom update engine out of release tooling.
ABOUTME: Protects the retained package, signing, and release refusal boundaries.
"""

import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]


class UpdaterAbsenceTests(unittest.TestCase):
    def test_custom_updater_scripts_and_workflow_are_absent(self):
        for name in ("scripts/update-manifest.py", "scripts/test-updater.py",
                     "scripts/test_update_manifest.py", "scripts/test_package_manifest.py",
                     "scripts/test_windows_manifest.py", ".github/workflows/updater.yml"):
            with self.subTest(name=name):
                self.assertFalse((ROOT / name).exists(), name)

    def test_release_routes_do_not_create_or_sign_custom_feeds(self):
        for name in (".github/workflows/release.yml", ".github/workflows/ci.yml",
                     "scripts/release/build-macos.sh", "scripts/release/sign-macos.sh",
                     "justfile", "flake.nix"):
            source = (ROOT / name).read_text()
            for token in ("scripts/update-manifest.py", "scripts/test-updater.py",
                          "HERDR_UPDATE_PUBLIC_KEY", "HERDR_UPDATE_SIGNING_KEY",
                          "HERDR_WINDOWS_SIGNER", "sheprd-updater", "update-manifest.json",
                          "package-manifest.json", "windows-manifest.json", "-update.tar.gz",
                          "macos-universal.app.tar.gz"):
                with self.subTest(name=name, token=token):
                    self.assertNotIn(token, source)

    def test_current_artifacts_are_packages_and_sbom(self):
        spec = importlib.util.spec_from_file_location(
            "artifact_manifest", ROOT / "scripts/release/artifact-manifest.py")
        manifest = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(manifest)
        names = manifest.base_names("20261109.1")
        self.assertTrue(names)
        self.assertTrue(all(name.startswith("Sheprd-") for name in names), names)
        self.assertTrue(all(name.endswith((".dmg", ".tar.gz", ".deb", ".rpm", ".pkg.tar.zst",
                                          ".zip", ".exe", ".cdx.json")) for name in names), names)

    def test_windows_installer_retains_restart_manager_without_custom_relaunch(self):
        source = (ROOT / "scripts/release/sheprd.iss").read_text()
        self.assertIn("CloseApplications=yes", source)
        self.assertIn("RestartApplications=no", source)
        self.assertNotIn("RELAUNCH", source)
        self.assertNotIn("RelaunchRequested", source)
        smoke = (ROOT / "scripts/release/windows-installer.py").read_text()
        self.assertNotIn("check_update_relaunches", smoke)
        self.assertNotIn("/RELAUNCH=1", smoke)

    def test_protected_publication_and_release_version_remain(self):
        release = (ROOT / ".github/workflows/release.yml").read_text()
        for token in ("github.repository == 'sheprd-labs/sheprd'", "github.actor == 'penso'",
                      "github.ref == 'refs/heads/main'", "environment: release",
                      "HERDR_RELEASE_VERSION:", "scripts/release/check-ci.py",
                      "scripts/release/artifact-manifest.py verify", "actions/attest-build-provenance@",
                      "scripts/release/smoke-cli.py"):
            self.assertIn(token, release)

    def test_install_references_do_not_advertise_custom_update_controls(self):
        for directory in ("site", "website/site", "website/herdrui/public"):
            for filename in ("install.sh", "install.ps1", "llms.txt"):
                with self.subTest(directory=directory, filename=filename):
                    source = (ROOT / directory / filename).read_text()
                    self.assertNotIn("Check for Updates", source)
                    self.assertNotIn("App Updates panel", source)
                    self.assertNotIn("docs/updating.md", source)
