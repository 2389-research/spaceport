#!/usr/bin/env python3
# ABOUTME: Verify shipped cross-platform artwork derives from the approved mark.
# ABOUTME: Keep native container sizes and generated platform sources covered.
from pathlib import Path
import runpy
import struct
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / "assets/icons"
SVG = "{http://www.w3.org/2000/svg}"
GENERATOR = runpy.run_path(str(ROOT / "scripts/generate-linux-icons.py"))


class ArtworkTests(unittest.TestCase):
    def test_platform_sources_derive_from_the_canonical_mark(self):
        source = (ASSETS / "sheprd-avatar.svg").read_text()
        variants = {
            "sheprd-linux.svg": ("circle", False, False),
            "sheprd-linux-worktree.svg": ("circle", True, False),
            "sheprd-icon.svg": ("tile", False, False),
            "sheprd-icon-worktree.svg": ("tile", True, False),
            "sheprd-icon-square.svg": ("square", False, False),
            "sheprd-icon-square-worktree.svg": ("square", True, False),
            "sheprd-avatar-worktree.svg": ("square", True, False),
            "sheprd-icon-badge.svg": ("tile", False, True),
        }
        for name, options in variants.items():
            with self.subTest(name=name):
                self.assertTrue((ASSETS / name).is_file(), f"Missing generated source {name}")
                self.assertTrue((ASSETS / name).read_text() == GENERATOR["artwork"](source, *options),
                                f"Regenerate {name} from the canonical source")

    def test_development_background_is_distinct(self):
        source = ET.parse(ASSETS / "sheprd-linux-worktree.svg").getroot()
        self.assertEqual([stop.get("stop-color") for stop in source.iter(f"{SVG}stop")], ["#df4b62", "#991f43"])

    def test_windows_containers_have_all_native_png_sizes(self):
        for name in ("Sheprd.ico", "Sheprd-worktree.ico"):
            data = (ASSETS / name).read_bytes()
            reserved, kind, count = struct.unpack_from("<HHH", data)
            self.assertEqual((reserved, kind), (0, 1))
            sizes = []
            for index in range(count):
                w, h, _, _, planes, depth, length, offset = struct.unpack_from("<BBBBHHII", data, 6 + index * 16)
                sizes.append(w or 256)
                self.assertEqual((w, planes, depth), (h, 1, 32))
                png = data[offset:offset + length]
                self.assertEqual(png[:8], b"\x89PNG\r\n\x1a\n")
                self.assertEqual(struct.unpack_from(">II", png, 16), (w or 256, h or 256))
            self.assertEqual(sizes, [16, 20, 24, 32, 40, 48, 64, 256])

    def test_social_cards_have_platform_dimensions_and_fit_upload_limit(self):
        for name, size in (("sheprd-social", (1200, 630)), ("sheprd-repository", (1280, 640))):
            png = (ROOT / f"assets/branding/{name}.png").read_bytes()
            self.assertEqual(struct.unpack_from(">II", png, 16), size)
            self.assertLess(len(png), 1_000_000)


if __name__ == "__main__":
    unittest.main()
