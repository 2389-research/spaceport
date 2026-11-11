#!/usr/bin/env python3
"""ABOUTME: Create and verify Spaceport's signed Sparkle preview appcast.
ABOUTME: Bind feed entries to immutable preview artifacts and the public trust key.
"""

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / "scripts/release/sparkle.json"
NS = "http://www.andymatuschak.org/xml-namespaces/sparkle"
PREFIX = "https://github.com/harperreed/spaceport/releases/download/"
OPENSSL = os.environ.get("SPACEPORT_OPENSSL", "openssl")
MAX_FEED = 1024 * 1024
ET.register_namespace("sparkle", NS)


def version_tuple(version):
    if not re.fullmatch(r"[1-9][0-9]{7}\.[1-9][0-9]*", version):
        raise ValueError("Version must be YYYYMMDD.COUNTER without leading zeros")
    return tuple(map(int, version.split(".")))


def read_feed(source):
    with source.open("rb") as opened:
        data = opened.read(MAX_FEED + 1)
    if len(data) > MAX_FEED:
        raise ValueError("Feed exceeds size bound")
    return data


def require_newer(version, previous):
    if version_tuple(version) <= version_tuple(previous):
        raise ValueError("Preview feed version must be strictly newer")


def decoded(value, length):
    try:
        data = base64.b64decode(value, validate=True)
    except (ValueError, TypeError) as error:
        raise ValueError("Invalid signature or public key encoding") from error
    if len(data) != length:
        raise ValueError("Invalid signature or public key length")
    return data


def verify_signature(content, signature, public_key):
    """Use OpenSSL's Ed25519 implementation, never a local crypto implementation."""
    with tempfile.TemporaryDirectory(prefix="spaceport-signature-") as temporary:
        root = Path(temporary)
        # RFC 8410 SubjectPublicKeyInfo prefix for a raw Ed25519 public key.
        (root / "public.der").write_bytes(bytes.fromhex("302a300506032b6570032100")
                                         + decoded(public_key, 32))
        (root / "signature").write_bytes(decoded(signature, 64))
        source = content if isinstance(content, Path) else root / "content"
        if not isinstance(content, Path):
            source.write_bytes(content)
        result = subprocess.run(
            [OPENSSL, "pkeyutl", "-verify", "-pubin", "-inkey", str(root / "public.der"),
             "-keyform", "DER", "-rawin", "-in", str(source),
             "-sigfile", str(root / "signature")],
            capture_output=True, check=False, timeout=30,
        )
        if result.returncode:
            raise ValueError("Ed25519 signature verification failed (OpenSSL 3+ required)")


def artifact_url(version):
    version_tuple(version)
    return f"{PREFIX}v{version}/Spaceport-{version}-arm64-apple-darwin.dmg"


def render(version, length, signature, notes):
    url = artifact_url(version)
    decoded(signature, 64)
    if not isinstance(length, int) or length <= 0:
        raise ValueError("Archive length must be positive")
    rss = ET.Element("rss", {"version": "2.0"})
    channel = ET.SubElement(rss, "channel")
    ET.SubElement(channel, "title").text = "Spaceport macOS previews"
    ET.SubElement(channel, "link").text = "https://github.com/harperreed/spaceport/releases"
    item = ET.SubElement(channel, "item")
    ET.SubElement(item, "title").text = f"Spaceport {version} (macOS arm64 preview)"
    ET.SubElement(item, "description", {f"{{{NS}}}format": "markdown"}).text = notes
    ET.SubElement(item, f"{{{NS}}}minimumSystemVersion").text = "14.2"
    ET.SubElement(item, "enclosure", {
        "url": url, "length": str(length), "type": "application/octet-stream",
        f"{{{NS}}}version": version, f"{{{NS}}}shortVersionString": version,
        f"{{{NS}}}edSignature": signature, f"{{{NS}}}os": "macos",
    })
    data = ET.tostring(rss, encoding="utf-8", xml_declaration=True) + b"\n"
    if len(data) > MAX_FEED:
        raise ValueError("Feed exceeds size bound")
    return data


def parse(data, config):
    if len(data) > MAX_FEED:
        raise ValueError("Feed exceeds size bound")
    match = re.fullmatch(
        rb"(.+)<!-- sparkle-signatures:\nedSignature: ([A-Za-z0-9+/=]+)\n"
        rb"length: ([0-9]+)\n-->\n?", data, re.DOTALL)
    if not match:
        raise ValueError("Missing or invalid feed signature block")
    content, signature, length = match.groups()
    if int(length) != len(content):
        raise ValueError("Signed feed length mismatch")
    verify_signature(content, signature, config["public_key"])
    if b"<!DOCTYPE" in content or b"<!ENTITY" in content:
        raise ValueError("Feed must not contain XML declarations of entities")
    try:
        rss = ET.fromstring(content)
    except ET.ParseError as error:
        raise ValueError("Invalid appcast XML") from error
    items = rss.findall("./channel/item")
    if rss.tag != "rss" or len(items) != 1:
        raise ValueError("Preview feed must have exactly one update")
    enclosures = items[0].findall("enclosure")
    if len(enclosures) != 1:
        raise ValueError("Preview feed must have exactly one enclosure")
    enclosure = enclosures[0]
    version = enclosure.get(f"{{{NS}}}version", "")
    url = enclosure.get("url")
    if url != artifact_url(version):
        raise ValueError("Unexpected preview download URL")
    if (enclosure.get(f"{{{NS}}}shortVersionString") != version
            or enclosure.get(f"{{{NS}}}os") != "macos"
            or items[0].findtext(f"{{{NS}}}minimumSystemVersion") != "14.2"):
        raise ValueError("Unexpected platform or version metadata")
    try:
        archive_length = int(enclosure.get("length", ""))
    except ValueError as error:
        raise ValueError("Invalid archive length") from error
    if not 0 < archive_length <= 1024 * 1024 * 1024:
        raise ValueError("Archive length outside preview bounds")
    archive_signature = enclosure.get(f"{{{NS}}}edSignature")
    decoded(archive_signature, 64)
    return {"version": version, "url": url, "length": archive_length,
            "signature": archive_signature, "notes": items[0].findtext("description", "")}


def verify_archive(item, archive, public_key):
    if archive.stat().st_size != item["length"]:
        raise ValueError("Archive length mismatch")
    verify_signature(archive, item["signature"], public_key)


def publication_version(data, previous, config, published_tags):
    version = parse(data, config)["version"]
    versions = [version_tuple(tag[1:]) for tag in published_tags
                if re.fullmatch(r"v[1-9][0-9]{7}\.[1-9][0-9]*", tag)]
    if not versions or max(versions) != version_tuple(version):
        raise ValueError("Feed must reference the newest published preview")
    if previous is not None:
        require_newer(version, parse(previous, config)["version"])
    return version


def sign(tool, archive, key):
    env = dict(os.environ)
    env.pop("SPARKLE_PRIVATE_KEY", None)
    return subprocess.check_output(
        [str(tool), "--ed-key-file", "-", "-p", str(archive)],
        input=key, env=env, stderr=subprocess.PIPE, timeout=120).decode().strip()


def create(args, config):
    version_tuple(args.version)
    if args.output.exists() or args.output.is_symlink():
        raise ValueError("Feed output already exists")
    if args.archive.name != artifact_url(args.version).rsplit("/", 1)[-1]:
        raise ValueError("Archive name does not match preview version")
    metadata = json.loads(args.archive.with_suffix(".dmg.json").read_text())
    expected = {"app_id": "ai.2389.spaceport", "team_id": "HD9NM9NSMK",
                "version": args.version, "architecture": "arm64",
                "profile": "release", "kind": "signed-macos-preview"}
    if any(metadata.get(key) != value for key, value in expected.items()):
        raise ValueError("Signed artifact metadata mismatch")
    with args.archive.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    if digest != metadata["sha256"]:
        raise ValueError("Signed artifact checksum mismatch")
    key = os.environ.pop("SPARKLE_PRIVATE_KEY", "").encode()
    if not key:
        raise ValueError("SPARKLE_PRIVATE_KEY is required for signing")
    signature = sign(args.tool, args.archive, key)
    args.output.write_bytes(render(args.version, args.archive.stat().st_size,
                                   signature, args.notes.read_text()))
    sign(args.tool, args.output, key)
    item = parse(read_feed(args.output), config)
    verify_archive(item, args.archive, config["public_key"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, default=CONFIG)
    actions = parser.add_subparsers(dest="action", required=True)
    make = actions.add_parser("create", help="sign verified final DMG and an appcast with SDK tool")
    make.add_argument("version")
    make.add_argument("archive", type=Path)
    make.add_argument("notes", type=Path)
    make.add_argument("output", type=Path)
    make.add_argument("--tool", type=Path, required=True)
    verify = actions.add_parser("verify", help="verify signed feed and optional local archive")
    verify.add_argument("feed", type=Path)
    verify.add_argument("--archive", type=Path)
    args = parser.parse_args()
    try:
        config = json.loads(args.config.read_text())
        if args.action == "create":
            create(args, config)
        else:
            item = parse(read_feed(args.feed), config)
            if args.archive:
                verify_archive(item, args.archive, config["public_key"])
            print(json.dumps(item))
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        parser.exit(1, f"sparkle-feed: {error}\n")


if __name__ == "__main__":
    main()
