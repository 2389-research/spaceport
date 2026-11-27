"""ABOUTME: Guards the Spaceport workspace against the retired GitHub implementation.
ABOUTME: Keeps its unused GitHub App upload art out of shipped sources.
"""

from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[3]


class GitHubCrateAbsence(unittest.TestCase):
    def test_excluded_crate_has_no_workspace_route(self):
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())
        self.assertFalse((ROOT / "crates/sheprd-github").exists())
        self.assertNotIn("crates/sheprd-github", workspace["workspace"]["members"])
        self.assertNotIn("sheprd-github", workspace["workspace"]["dependencies"])
        self.assertNotIn('name = "sheprd-github"', (ROOT / "Cargo.lock").read_text())

    def test_github_app_upload_art_is_absent(self):
        for name in ("sheprd-github-app.svg", "sheprd-github-app.png", "sheprd-github-app-200.png"):
            with self.subTest(name=name):
                self.assertFalse((ROOT / "assets/icons" / name).exists())
        for relative in ("scripts/generate-linux-icons.py", "scripts/generate-icons.swift"):
            with self.subTest(path=relative):
                self.assertNotIn("sheprd-github-app", (ROOT / relative).read_text())


if __name__ == "__main__":
    unittest.main()
