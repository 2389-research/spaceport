# ABOUTME: Check that Spaceport uses the approved nebula rocket image unchanged.
# ABOUTME: Keep canonical source validation and platform crop behavior covered.
import base64
import hashlib
from pathlib import Path
import runpy
import struct
import unittest
import xml.etree.ElementTree as ET

ASSETS = Path(__file__).resolve().parent
ROOT = ASSETS.parents[1]
APPROVED = ROOT / "design/spaceport-icon/rocket-nebula.png"
SVG = "{http://www.w3.org/2000/svg}"
GENERATOR = runpy.run_path(str(ROOT / "scripts/generate-linux-icons.py"))


class RocketTests(unittest.TestCase):
    def canonical(self, png):
        self.assertTrue(callable(GENERATOR.get("canonical")), "The canonical SVG must derive from the approved PNG")
        return GENERATOR["canonical"](png)

    def test_approved_source_has_not_been_replaced(self):
        self.assertEqual(hashlib.sha256(APPROVED.read_bytes()).hexdigest(),
                         "caa551b4dc59b1318f736cadc6e09c03f3ba5c8644ca4f9ed45f71d1ff4e4b00")

    def test_canonical_mark_embeds_the_approved_nebula_rocket(self):
        source = ET.parse(ASSETS / "sheprd-avatar.svg").getroot()
        self.assertEqual(source.get("viewBox"), "0 0 100 100")
        self.assertEqual(source.find(f"{SVG}title").text, "Spaceport")
        self.assertIn("rocket-nebula.png", source.find(f"{SVG}metadata").text)
        mark = source.find(f"{SVG}g[@id='rocket']")
        self.assertIsNotNone(mark)
        viewport = mark.find(f"{SVG}svg")
        self.assertIsNotNone(viewport, "The approved image needs an explicit crop viewport")
        self.assertEqual(viewport.get("viewBox"), "114 114 1025 1031")
        self.assertEqual(viewport.get("preserveAspectRatio"), "none")
        image = viewport.find(f"{SVG}image")
        self.assertIsNotNone(image)
        href = image.get("href", "")
        self.assertTrue(href.startswith("data:image/png;base64,"))
        self.assertTrue(base64.b64decode(href.split(",", 1)[1], validate=True) == APPROVED.read_bytes(),
                        "Canonical artwork must preserve the exact approved PNG bytes")

    def test_canonical_svg_is_reproducible_from_the_approved_png(self):
        self.assertTrue(self.canonical(APPROVED.read_bytes()) == (ASSETS / "sheprd-avatar.svg").read_text(),
                        "Regenerate the canonical SVG from the approved PNG")

    def test_canonical_rejects_non_png_truncated_and_wrong_size_sources(self):
        png = APPROVED.read_bytes()
        wrong_header = bytearray(png)
        wrong_header[12:16] = b"IDAT"
        rectangular = bytearray(png)
        rectangular[16:24] = struct.pack(">II", 1254, 1253)
        wrong_size = bytearray(png)
        wrong_size[16:24] = struct.pack(">II", 1024, 1024)
        for label, data in (("not PNG", b"not an image"), ("truncated", png[:20]),
                            ("missing IHDR", wrong_header), ("rectangular", rectangular),
                            ("wrong size", wrong_size)):
            with self.subTest(source=label), self.assertRaises(ValueError):
                self.canonical(bytes(data))

    def test_every_shape_preserves_the_approved_image_and_crop(self):
        source = (ASSETS / "sheprd-avatar.svg").read_text()
        canonical = ET.fromstring(source).find(f"{SVG}g[@id='rocket']")
        self.assertIsNotNone(canonical)
        expected = [(node.tag, node.attrib) for child in canonical for node in child.iter()]
        for shape in ("circle", "tile", "square"):
            for worktree in (False, True):
                with self.subTest(shape=shape, worktree=worktree):
                    icon = ET.fromstring(GENERATOR["artwork"](source, shape, worktree))
                    self.assertEqual(icon.find(f"{SVG}title").text, "Spaceport")
                    mark = icon.find(f".//{SVG}g[@id='rocket']")
                    self.assertIsNotNone(mark)
                    actual = [(node.tag, node.attrib) for child in mark for node in child.iter()]
                    self.assertTrue(actual == expected, "Platform masking must not alter PNG colors or crop")

    def test_missing_mark_is_a_clear_error(self):
        with self.assertRaisesRegex(ValueError, "mark"):
            GENERATOR["artwork"]('<svg xmlns="http://www.w3.org/2000/svg"><defs/></svg>')


if __name__ == "__main__":
    unittest.main()
