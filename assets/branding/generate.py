#!/usr/bin/env python3
# ABOUTME: Renders social cards from Spaceport's shared app artwork.
# ABOUTME: Keeps every card within its platform dimensions and upload size limit.
"""Render Spaceport's social cards from the canonical artwork, using librsvg."""
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

ASSETS = Path(__file__).resolve().parents[1]
SVG = "{http://www.w3.org/2000/svg}"
ET.register_namespace("", SVG[1:-1])


def card(width, height, dark=False):
    background, foreground, secondary = (
        ("#211b18", "#fff8f0", "#c9b9aa") if dark else ("#fff8f0", "#34241c", "#795b49")
    )
    root = ET.Element(f"{SVG}svg", width=str(width), height=str(height), viewBox=f"0 0 {width} {height}")
    ET.SubElement(root, f"{SVG}title").text = "Spaceport — Your agents. One place."
    ET.SubElement(root, f"{SVG}rect", width=str(width), height=str(height), fill=background)
    ET.SubElement(root, f"{SVG}rect", x="64", y="68", width="56", height="6", rx="3", fill="#d65f2a")
    for text, y, size, weight, color in (
        ("Spaceport", 255, 100, "700", foreground),
        ("Your agents. One place.", 338, 40, "500", foreground),
        ("Native workspaces for AI-assisted development.", 395, 23, "400", secondary),
        ("macOS  /  Linux  /  Windows", height - 68, 21, "400", secondary),
    ):
        ET.SubElement(root, f"{SVG}text", x="64", y=str(y), fill=color, **{
            "font-family": "Arial, sans-serif", "font-size": str(size), "font-weight": weight,
        }).text = text
    mark = ET.parse(ASSETS / "icons/sheprd-icon.svg").getroot()
    mark.set("x", str(width - 430))
    mark.set("y", str((height - 340) // 2))
    mark.set("width", "340")
    mark.set("height", "340")
    # Keep the full rounded tile rather than applying a second circular crop.
    root.append(mark)
    ET.indent(root, space="  ")
    return ET.tostring(root, encoding="unicode") + "\n"


def main():
    outputs = {
        "branding/sheprd-social": (1200, 630, False),
        "branding/sheprd-repository": (1280, 640, False),
        "social-preview-light": (1280, 640, False),
        "social-preview-light-ui": (1280, 640, False),
        "social-preview-dark": (1280, 640, True),
    }
    for name, options in outputs.items():
        svg, png = ASSETS / f"{name}.svg", ASSETS / f"{name}.png"
        svg.write_text(card(*options))
        subprocess.run(["rsvg-convert", str(svg), "-o", str(png)], check=True)
        if png.stat().st_size >= 1_000_000:
            raise ValueError(f"Social card exceeds GitHub's 1 MB limit: {png}")
        print(f"Generated {png.relative_to(ASSETS)}")


if __name__ == "__main__":
    main()
