"""ABOUTME: Checks that native pull request actions have no shipped route.
ABOUTME: Keeps read-only pull request and account owners available for later cuts.
"""

from pathlib import Path
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

    def test_read_only_owners_remain(self):
        for path in (
            "crates/sheprd-github/src/pull_request.rs",
            "crates/sheprd-gpui/src/menu/github.rs",
            "crates/sheprd-gpui/src/menu/git.rs",
        ):
            with self.subTest(path=path):
                self.assertTrue((ROOT / path).is_file())


if __name__ == "__main__":
    unittest.main()
