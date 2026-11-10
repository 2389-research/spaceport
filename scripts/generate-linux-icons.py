#!/usr/bin/env python3
"""Package the approved Spaceport PNG in self-contained platform SVGs.

Linux uses a circular mask; desktop tiles keep platform-specific margins.
This runs first in `just icons`, before the native raster/package generators.
"""
from copy import deepcopy
import base64
from pathlib import Path
import struct
import xml.etree.ElementTree as ET

ASSETS = Path(__file__).resolve().parents[1] / "assets/icons"
SOURCE = ASSETS.parents[1] / "design/spaceport-icon/rocket-nebula.png"
SVG = "{http://www.w3.org/2000/svg}"
ET.register_namespace("", SVG[1:-1])


def canonical(png):
    if len(png) < 33 or png[:8] != b"\x89PNG\r\n\x1a\n" or png[12:16] != b"IHDR":
        raise ValueError("Canonical mark must be a PNG with an IHDR header")
    width, height = struct.unpack_from(">II", png, 16)
    if (width, height) != (1254, 1254):
        raise ValueError("Approved nebula PNG must be 1254 by 1254 pixels")
    root = ET.Element(f"{SVG}svg", width="1024", height="1024", viewBox="0 0 100 100")
    ET.SubElement(root, f"{SVG}title", id="title").text = "Spaceport"
    ET.SubElement(root, f"{SVG}desc", id="desc").text = "🚀 sculpted rocket on a violet and cyan nebula."
    ET.SubElement(root, f"{SVG}metadata").text = (
        "Approved rocket-nebula.png; generated with OpenAI imagegen from the Twemoji rocket concept. "
        "Twemoji: Twitter, Inc. and contributors, CC BY 4.0; see NOTICE and LICENSE-twemoji."
    )
    defs = ET.SubElement(root, f"{SVG}defs")
    gradient = ET.SubElement(defs, f"{SVG}linearGradient", id="background", x1="0", y1="0", x2="0.35", y2="1")
    for offset, color in (("0", "#15233b"), ("1", "#0b1220")):
        ET.SubElement(gradient, f"{SVG}stop", offset=offset, **{"stop-color": color})
    ET.SubElement(root, f"{SVG}rect", width="100", height="100", fill="url(#background)")
    mark = ET.SubElement(root, f"{SVG}g", id="rocket")
    # These are the approved image's measured alpha bounds. Remove its outer
    # padding before applying each platform's own margins; retain all artwork.
    viewport = ET.SubElement(mark, f"{SVG}svg", width="100", height="100",
                             viewBox="114 114 1025 1031", preserveAspectRatio="none")
    ET.SubElement(viewport, f"{SVG}image", width=str(width), height=str(height),
                  href="data:image/png;base64," + base64.b64encode(png).decode("ascii"))
    ET.indent(root, space="  ")
    return ET.tostring(root, encoding="unicode") + "\n"


def artwork(source, shape="circle", worktree=False, badge=False):
    avatar = ET.fromstring(source)
    defs = deepcopy(avatar.find(f"{SVG}defs"))
    mark = deepcopy(avatar.find(f"{SVG}g"))
    images = [] if mark is None else list(mark.iter(f"{SVG}image"))
    gradient = None if defs is None else defs.find(f"{SVG}linearGradient[@id='background']")
    if gradient is None or len(list(gradient.iter(f"{SVG}stop"))) != 2 or mark is None or len(images) != 1:
        raise ValueError("Canonical avatar must contain its gradient and mark")
    uri = images[0].get("href", "")
    if not uri.startswith("data:image/png;base64,"):
        raise ValueError("Canonical mark must embed its PNG, not an external file")
    canonical_png = base64.b64decode(uri.removeprefix("data:image/png;base64,"), validate=True)
    canonical(canonical_png)
    if worktree:
        # The development rim changes; the approved image stays byte-identical.
        for stop, color in zip(defs.iter(f"{SVG}stop"), ("#df4b62", "#991f43"), strict=True):
            stop.set("stop-color", color)
    icon = ET.Element(f"{SVG}svg", {
        "width": "1024", "height": "1024", "viewBox": "0 0 1024 1024",
        "role": "img", "aria-labelledby": "title desc",
    })
    ET.SubElement(icon, f"{SVG}title", id="title").text = avatar.find(f"{SVG}title").text
    ET.SubElement(icon, f"{SVG}desc", id="desc").text = (
        "🚀 rocket on a nebula" + (" with a berry development rim." if worktree else ".")
    )
    ET.SubElement(icon, f"{SVG}metadata").text = "Derived from sheprd-avatar.svg; " + avatar.find(f"{SVG}metadata").text
    icon.append(defs)
    if shape == "circle":
        outline = ET.SubElement(icon, f"{SVG}circle", cx="512", cy="512", r="448", fill="url(#background)")
    elif shape == "tile":
        outline = ET.SubElement(icon, f"{SVG}rect", x="64", y="64", width="896", height="896", rx="192", fill="url(#background)")
    elif shape == "square":
        outline = ET.SubElement(icon, f"{SVG}rect", width="1024", height="1024", fill="url(#background)")
    else:
        raise ValueError(f"Unknown icon shape: {shape}")
    clip = ET.SubElement(defs, f"{SVG}clipPath", id="silhouette")
    clip.append(deepcopy(outline))
    transforms = {
        "circle": "translate(135.68 135.68) scale(7.5264)",
        "tile": "translate(64 64) scale(8.96)",
        "square": "scale(10.24)",
    }
    # Circular exports need room for the nose and plume, beyond the tile crop.
    mark.set("transform", transforms[shape])
    clipped = ET.SubElement(icon, f"{SVG}g", **{"clip-path": "url(#silhouette)"})
    clipped.append(mark)
    if worktree:
        rim = deepcopy(outline)
        rim.attrib.update(id="development-ring", fill="none", stroke="#df4b62", **{"stroke-width": "24"})
        clipped.append(rim)
    if badge:
        ET.SubElement(icon, f"{SVG}circle", cx="839", cy="185", r="135", fill="#c93647", stroke="#fff8f0", **{"stroke-width": "20"})
        # Outlined numeral, independent of installed fonts.
        ET.SubElement(icon, f"{SVG}path", d="M811 152 L842 130 L842 236 M812 236 H874", fill="none", stroke="#fff8f0", **{"stroke-width": "22", "stroke-linecap": "round", "stroke-linejoin": "round"})
    ET.indent(icon, space="  ")
    return ET.tostring(icon, encoding="unicode") + "\n"


def main():
    source = canonical(SOURCE.read_bytes())
    (ASSETS / "sheprd-avatar.svg").write_text(source)
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
        (ASSETS / name).write_text(artwork(source, *options))
        print(f"Generated assets/icons/{name}")


if __name__ == "__main__":
    main()
