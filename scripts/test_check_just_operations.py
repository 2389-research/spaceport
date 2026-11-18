# ABOUTME: Check the codified-operations checker's parsing and drift detection.
# ABOUTME: These fixtures are pure files; nothing is executed or networked.
import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("check_ops", SCRIPTS / "check-just-operations.py")
check = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check)


def parse(tmp: Path, justfile: str, guide: str):
    """Point the module's constants at fixture files, then parse both."""
    (tmp / "justfile").write_text(justfile)
    ops = tmp / "docs"
    ops.mkdir()
    (ops / "just-operations.md").write_text(guide)
    saved = check.JUSTFILE, check.GUIDE
    check.JUSTFILE = tmp / "justfile"
    check.GUIDE = ops / "just-operations.md"
    try:
        return check.parse_justfile(), check.parse_guide()
    finally:
        check.JUSTFILE, check.GUIDE = saved


class JustfileParsing(unittest.TestCase):
    def test_recipes_with_every_parameter_shape(self):
        with tempfile.TemporaryDirectory() as d:
            recipes, (_, _) = parse(
                Path(d),
                "\n".join(
                    [
                        "default:",
                        "\t@just --list",
                        "run *args:",
                        "\t@echo run",
                        "dmg $VERSION profile=\"debug\" architecture=\"arm64\":",
                        "\t@echo dmg",
                        "nextest *args:",
                        "\t@echo nextest",
                        "test-gpui filter=\"package = spaceport\" repetitions=\"1\":",
                        "\t@echo t",
                        "_private:",
                        "\t@echo hidden",
                    ]
                ),
                "",
            )
        self.assertEqual(
            recipes,
            {
                "default": [],
                "run": ["args"],
                "dmg": ["VERSION", 'profile="debug"', 'architecture="arm64"'],
                "nextest": ["args"],
                "test-gpui": ['filter="package = spaceport"', 'repetitions="1"'],
            },
        )

    def test_recipe_lines_inside_bodies_do_not_parse(self):
        with tempfile.TemporaryDirectory() as d:
            recipes, (_, _) = parse(
                Path(d),
                "build:\n\tcargo build\nother:\n\t@echo build:",
                "",
            )
        self.assertEqual(recipes, {"build": [], "other": []})


class DriftDetection(unittest.TestCase):
    def test_undocumented_recipe_fails(self):
        with tempfile.TemporaryDirectory() as d:
            _, (refs, _) = parse(Path(d), "ci:\n\t@echo ci\n", "nothing here\n")
        self.assertEqual(refs, set())

    def test_documented_recipe_passes_and_stale_reference_fails(self):
        with tempfile.TemporaryDirectory() as d:
            _, (refs, _) = parse(
                Path(d),
                "ci:\n\t@echo ci\nlint:\n\t@echo lint\n",
                "| `just ci` | all checks |\n| `just fmt` | gone |\n",
            )
        self.assertEqual(refs, {"ci", "fmt"})

    def test_word_boundary_does_not_confuse_run_with_run_debug(self):
        with tempfile.TemporaryDirectory() as d:
            _, (refs, _) = parse(
                Path(d),
                "run-debug *args:\n\t@echo d\n",
                "| `just run-debug args` | debug |\n",
            )
        self.assertEqual(refs, {"run-debug"})


class LiveFiles(unittest.TestCase):
    def test_the_repository_itself_passes(self):
        """The gate the CI job runs: the real justfile and guide agree."""
        problems = []
        recipes = check.parse_justfile()
        referenced, _ = check.parse_guide()
        problems += [n for n in recipes if n not in referenced]
        problems += [n for n in referenced if n not in recipes]
        self.assertEqual(problems, [])


if __name__ == "__main__":
    unittest.main()
