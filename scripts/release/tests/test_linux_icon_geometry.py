# ABOUTME: Check platform masks and raster crop geometry independently of regeneration.
# ABOUTME: Reject SVG layouts that move, stretch, replace, or expose the approved image.
import base64
import math
from pathlib import Path
import re
import struct
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[3]
ASSETS = ROOT / "assets/icons"
APPROVED = ROOT / "design/spaceport-icon/rocket-nebula.png"
SVG = "{http://www.w3.org/2000/svg}"
NUMBER = r"[-+]?(?:\d*\.\d+|\d+\.?\d*)(?:[eE][-+]?\d+)?"


class LinuxIconGeometryTests(unittest.TestCase):
    def check_geometry(self, root, shape="circle", development=False):
        self.assertEqual(root.get("viewBox"), "0 0 1024 1024")
        self.assertEqual((root.get("width"), root.get("height")), ("1024", "1024"))
        if shape == "circle":
            tag, geometry = "circle", {"cx": "512", "cy": "512", "r": "448"}
        elif shape == "tile":
            tag, geometry = "rect", {"x": "64", "y": "64", "width": "896", "height": "896", "rx": "192"}
        else:
            tag, geometry = "rect", {"width": "1024", "height": "1024"}
        background = root.find(SVG + tag)
        self.assertIsNotNone(background)
        self.assertEqual(background.attrib, geometry | {"fill": "url(#background)"})
        silhouette = root.find(f"{SVG}defs/{SVG}clipPath[@id='silhouette']")
        self.assertIsNotNone(silhouette, "Raster artwork must stay inside the platform silhouette")
        self.assertIn(silhouette.get("clipPathUnits"), (None, "userSpaceOnUse"))
        self.assertTrue(set(silhouette.attrib) <= {"id", "clipPathUnits"}, "Mask must not move or scale")
        self.assertEqual(len(silhouette), 1)
        self.assertEqual(silhouette[0].tag, SVG + tag)
        mask_geometry = {key: value for key, value in silhouette[0].attrib.items() if key != "fill"}
        self.assertEqual(mask_geometry, geometry)
        clipped = root.find(f"{SVG}g")
        self.assertIsNotNone(clipped)
        self.assertEqual(clipped.attrib, {"clip-path": "url(#silhouette)"})
        mark = clipped.find(f"{SVG}g[@id='rocket']")
        self.assertIsNotNone(mark)
        if shape == "square":
            self.assertEqual(mark.get("transform"), "scale(10.24)")
            tx, ty, scale = 0, 0, 10.24
        else:
            pair = rf"\(\s*({NUMBER})[\s,]+({NUMBER})\s*\)"
            transform = re.fullmatch(rf"translate{pair}\s+scale\(({NUMBER})\)", mark.get("transform", ""))
            self.assertIsNotNone(transform)
            tx, ty, scale = map(float, transform.groups())
            expected = (135.68, 135.68, 7.5264) if shape == "circle" else (64, 64, 8.96)
            self.assertEqual((tx, ty, scale), expected)
        self.assertEqual(len(mark), 1)
        viewport = mark[0]
        self.assertEqual(viewport.tag, SVG + "svg")
        self.assertEqual(viewport.get("viewBox"), "114 114 1025 1031")
        self.assertEqual((viewport.get("width"), viewport.get("height")), ("100", "100"))
        self.assertEqual(viewport.get("preserveAspectRatio"), "none")
        self.assertIsNone(viewport.get("transform"))
        self.assertIn(viewport.get("x"), (None, "0"))
        self.assertIn(viewport.get("y"), (None, "0"))
        self.assertEqual(len(viewport), 1)
        image = viewport[0]
        self.assertEqual(image.tag, SVG + "image")
        self.assertEqual((image.get("width"), image.get("height")), ("1254", "1254"))
        self.assertIsNone(image.get("transform"))
        self.assertIn(image.get("x"), (None, "0"))
        self.assertIn(image.get("y"), (None, "0"))
        href = image.get("href", "")
        self.assertTrue(href.startswith("data:image/png;base64,"), "Installed SVG must be self-contained")
        png = base64.b64decode(href.split(",", 1)[1], validate=True)
        self.assertTrue(png == APPROVED.read_bytes(), "Embedded artwork differs from the selected PNG")
        self.assertEqual(png[:8], b"\x89PNG\r\n\x1a\n")
        self.assertEqual(struct.unpack_from(">II", png, 16), (1254, 1254))
        # The circular crop is inset to keep the nose and flame inside its mask.
        # Evaluate source-to-output mapping rather than trusting generator text.
        x, y, width, height = map(float, viewport.get("viewBox").split())
        mapped = [(tx + scale * 100 * (sx - x) / width,
                   ty + scale * 100 * (sy - y) / height)
                  for sx, sy in ((114, 114), (1139, 1145))]
        inset = {"circle": 135.68, "tile": 64, "square": 0}[shape]
        for actual, expected in zip(mapped, ((inset, inset), (1024 - inset, 1024 - inset)), strict=True):
            for coordinate, bound in zip(actual, expected, strict=True):
                self.assertAlmostEqual(coordinate, bound)
        ring = clipped.find(f".//*[@id='development-ring']")
        if development:
            self.assertIsNotNone(ring, "Development artwork needs its visible berry ring")
            self.assertEqual(ring.tag, SVG + tag)
            self.assertEqual({key: ring.get(key) for key in geometry}, geometry)
            self.assertEqual(ring.get("fill"), "none")
            self.assertIsNone(ring.get("transform"))
            self.assertIn(ring.get("stroke"), ("#df4b62", "#991f43", "url(#background)"))
            self.assertGreater(float(ring.get("stroke-width")), 0)
        else:
            self.assertIsNone(ring)

    def test_shipped_platform_masks_and_raster_mapping(self):
        for name, shape, development in (
            ("sheprd-linux.svg", "circle", False),
            ("sheprd-linux-worktree.svg", "circle", True),
            ("sheprd-icon.svg", "tile", False),
            ("sheprd-icon-worktree.svg", "tile", True),
            ("sheprd-icon-square.svg", "square", False),
            ("sheprd-icon-square-worktree.svg", "square", True),
        ):
            with self.subTest(icon=name):
                self.assertTrue((ASSETS / name).is_file(), f"Missing generated source {name}")
                self.check_geometry(ET.parse(ASSETS / name).getroot(), shape, development)

    def check_circle_foreground_clearance(self, root):
        mark = root.find(f"{SVG}g/{SVG}g[@id='rocket']")
        self.assertIsNotNone(mark)
        transform = re.fullmatch(
            rf"translate\(({NUMBER}) ({NUMBER})\) scale\(({NUMBER})\)", mark.get("transform", ""))
        self.assertIsNotNone(transform)
        tx, ty, scale = map(float, transform.groups())
        viewport = mark.find(f"{SVG}svg")
        self.assertIsNotNone(viewport)
        x, y, width, height = map(float, viewport.get("viewBox").split())
        # These observed master-image landmarks touch the rocket's extremities.
        # The nebula tile's corners may be clipped; the rocket must clear the rim.
        for landmark, sx, sy in (("nose", 1002, 255), ("flame tip", 213, 1025)):
            px = tx + scale * 100 * (sx - x) / width
            py = ty + scale * 100 * (sy - y) / height
            clearance = 448 - math.hypot(px - 512, py - 512)
            self.assertGreaterEqual(clearance, 24, f"Rocket {landmark} reaches the circular mask: {clearance:.2f}px clearance")

    def test_circular_icons_preserve_nose_and_flame_tip(self):
        for name in ("sheprd-linux.svg", "sheprd-linux-worktree.svg"):
            with self.subTest(icon=name):
                self.check_circle_foreground_clearance(ET.parse(ASSETS / name).getroot())

    def test_foreground_check_rejects_the_clipping_circle_layout(self):
        root = ET.parse(ASSETS / "sheprd-linux.svg").getroot()
        root.find(f"{SVG}g/{SVG}g[@id='rocket']").set("transform", "translate(64 64) scale(8.96)")
        with self.assertRaisesRegex(AssertionError, "Rocket nose reaches"):
            self.check_circle_foreground_clearance(root)

    def test_geometry_rejects_broken_mask_crop_and_embedded_source(self):
        for selector, attribute, value in (
            (f"{SVG}circle", "r", "512"),
            (f"{SVG}circle", "cx", "540"),
            (f"{SVG}defs/{SVG}clipPath/{SVG}circle", "r", "440"),
            (f"{SVG}defs/{SVG}clipPath", "clipPathUnits", "objectBoundingBox"),
            (f"{SVG}defs/{SVG}clipPath", "transform", "translate(1 1)"),
            (f"{SVG}g", "clip-path", "none"),
            (f"{SVG}g", "transform", "translate(64 64)"),
            (f"{SVG}g/{SVG}g", "transform", "translate(400 792) scale(8.96)"),
            (f"{SVG}g/{SVG}g", "transform", "translate(64 64) scale(30)"),
            (f".//{SVG}svg", "viewBox", "0 0 1254 1254"),
            (f".//{SVG}svg", "preserveAspectRatio", "xMidYMid meet"),
            (f".//{SVG}svg", "width", "99"),
            (f".//{SVG}image", "width", "1024"),
            (f".//{SVG}image", "x", "10"),
            (f".//{SVG}image", "transform", "translate(1 1)"),
            (f".//{SVG}image", "href", "../../../design/spaceport-icon/rocket-nebula.png"),
            (f".//{SVG}image", "href", "data:image/png;base64,Y29ycnVwdA=="),
        ):
            root = ET.parse(ASSETS / "sheprd-linux.svg").getroot()
            node = root.find(selector)
            self.assertIsNotNone(node, f"Missing node required for corruption check: {selector}")
            node.set(attribute, value)
            with self.subTest(attribute=attribute, value=value), self.assertRaises(AssertionError):
                self.check_geometry(root)

    def test_development_geometry_rejects_shifted_or_unpainted_ring(self):
        for attribute, value in (("r", "400"), ("cx", "540"), ("stroke-width", "0"),
                                 ("fill", "#df4b62"), ("transform", "translate(1 1)")):
            root = ET.parse(ASSETS / "sheprd-linux-worktree.svg").getroot()
            ring = root.find(f".//*[@id='development-ring']")
            self.assertIsNotNone(ring)
            ring.set(attribute, value)
            with self.subTest(attribute=attribute), self.assertRaises(AssertionError):
                self.check_geometry(root, development=True)

    def test_development_geometry_rejects_a_missing_ring(self):
        root = ET.parse(ASSETS / "sheprd-linux-worktree.svg").getroot()
        clipped = root.find(f"{SVG}g")
        ring = clipped.find(f".//*[@id='development-ring']")
        self.assertIsNotNone(ring)
        clipped.remove(ring)
        with self.assertRaises(AssertionError):
            self.check_geometry(root, development=True)
