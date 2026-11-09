"""ABOUTME: Checks that native pull request actions have no shipped route.
ABOUTME: Keeps GitHub's library out of the Spaceport desktop app.
"""

from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[3]


class PullRequestActionAbsence(unittest.TestCase):
    def test_native_action_worker_and_pages_are_absent(self):
        self.assertFalse((ROOT / "crates/sheprd-github/src/pr_actions.rs").exists())
        self.assertFalse((ROOT / "crates/sheprd-gpui/src/menu/pr_actions.rs").exists())
        for crate in ("sheprd-github", "sheprd-gpui"):
            for path in (ROOT / "crates" / crate).rglob("*"):
                if path.suffix not in (".rs", ".ftl"):
                    continue
                text = path.read_text()
                for marker in ("pr_actions", "PrReview", "PrComment", "PrMerge"):
                    with self.subTest(path=path, marker=marker):
                        self.assertNotIn(marker, text)
        self.assertNotIn("mutation(", (ROOT / "crates/sheprd-github/src/http.rs").read_text())

    def test_github_library_has_no_desktop_route(self):
        app = ROOT / "crates/sheprd-gpui"
        manifest = tomllib.loads((app / "Cargo.toml").read_text())
        for section in ("dependencies", "dev-dependencies"):
            with self.subTest(section=section):
                self.assertNotIn("sheprd-github", manifest[section])
        self.assertNotIn("sheprd-github/test-support", manifest["features"]["integration-test"])
        for source in (app / "src").rglob("*.rs"):
            with self.subTest(path=source.relative_to(ROOT)):
                self.assertNotIn("sheprd_github", source.read_text())
        self.assertFalse((app / "locales/en-US/error-github.ftl").exists())


if __name__ == "__main__":
    unittest.main()
