# ABOUTME: Check that installed Linux SVGs preserve the approved shared raster art.
# ABOUTME: Exercise regeneration and reject canonical sources missing required content.
import base64
from pathlib import Path
import runpy
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[3]
ASSETS = ROOT / "assets/icons"
GENERATOR = runpy.run_path(str(ROOT / "scripts/generate-linux-icons.py"))
SVG = "{http://www.w3.org/2000/svg}"


class LinuxIconTests(unittest.TestCase):
    def test_committed_icons_are_reproducible_and_self_contained(self):
        source = (ASSETS / "sheprd-avatar.svg").read_text()
        approved = (ROOT / "design/spaceport-icon/rocket-nebula.png").read_bytes()
        for name, worktree in (("sheprd-linux.svg", False), ("sheprd-linux-worktree.svg", True)):
            with self.subTest(icon=name):
                committed = (ASSETS / name).read_text()
                self.assertTrue(committed == GENERATOR["artwork"](source, "circle", worktree),
                                f"Regenerate {name} from the canonical source")
                root = ET.fromstring(committed)
                self.assertIn("sheprd-avatar.svg", root.find(f"{SVG}metadata").text)
                self.assertIn("rocket-nebula.png", root.find(f"{SVG}metadata").text)
                images = list(root.iter(f"{SVG}image"))
                self.assertEqual(len(images), 1)
                href = images[0].get("href", "")
                self.assertTrue(href.startswith("data:image/png;base64,"))
                self.assertTrue(base64.b64decode(href.split(",", 1)[1], validate=True) == approved,
                                "Linux installation must not depend on an external image path")
                self.assertEqual([child.tag.removeprefix(SVG) for child in root],
                                 ["title", "desc", "metadata", "defs", "circle", "g"])

    def test_missing_image_mark_or_gradient_is_rejected(self):
        source = (ASSETS / "sheprd-avatar.svg").read_text()
        for tag in ("image", "linearGradient", "defs", "g"):
            root = ET.fromstring(source)
            self.assertIsNotNone(root.find(f".//{SVG}{tag}"))
            for parent in root.iter():
                for child in list(parent):
                    if child.tag == SVG + tag:
                        parent.remove(child)
            with self.subTest(missing=tag), self.assertRaises(ValueError):
                GENERATOR["artwork"](ET.tostring(root))

    def test_unknown_shape_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "shape"):
            GENERATOR["artwork"]((ASSETS / "sheprd-avatar.svg").read_text(), "triangle")
