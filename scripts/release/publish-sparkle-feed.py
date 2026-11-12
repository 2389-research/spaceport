#!/usr/bin/env python3
"""ABOUTME: Publish a signed preview feed only after its immutable release is verified.
ABOUTME: Keep the marker tag fixed and reject missing, unsigned or regressing feeds.
"""

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import tempfile

SPEC = importlib.util.spec_from_file_location("sparkle_feed", Path(__file__).with_name("sparkle-feed.py"))
feed = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(feed)
REPO = "2389-research/spaceport"
MARKER = "macos-preview-feed"


def gh(*args):
    return subprocess.check_output(["gh", *args], stderr=subprocess.PIPE, timeout=120)


def publish(source, expected_sha):
    if not re.fullmatch(r"[0-9a-f]{40}", expected_sha):
        raise ValueError("Full expected source SHA required")
    config = json.loads(feed.CONFIG.read_text())
    data = feed.read_feed(source)
    item = feed.parse(data, config)
    if source.name != "appcast.xml":
        raise ValueError("Feed must be named appcast.xml")
    history = gh("api", "--paginate", "--slurp", f"repos/{REPO}/releases?per_page=100")
    if len(history) > 8 * 1024 * 1024:
        raise ValueError("Published release history exceeds size bound")
    pages = json.loads(history)
    if not isinstance(pages, list) or any(not isinstance(page, list) for page in pages):
        raise ValueError("Malformed release history")
    releases = [release for page in pages for release in page if not release["draft"]]
    marker = next((release for release in releases if release["tag_name"] == MARKER), None)
    version_release = next((release for release in releases
                            if release["tag_name"] == "v" + item["version"]), None)
    if version_release is None or not version_release["prerelease"]:
        raise ValueError("Referenced preview release is not published")
    # A moved version tag cannot redirect a pre-signed feed to different source.
    tag_sha = gh("api", f"repos/{REPO}/commits/v{item['version']}", "--jq", ".sha").decode().strip()
    if tag_sha != expected_sha:
        raise ValueError("Published version tag does not match expected source")
    with tempfile.TemporaryDirectory(prefix="spaceport-publish-feed-") as temporary:
        root = Path(temporary)
        previous = None
        if marker is not None:
            feeds = [asset for asset in marker["assets"] if asset["name"] == "appcast.xml"]
            if len(feeds) != 1 or not 0 < feeds[0]["size"] <= feed.MAX_FEED:
                raise ValueError("Missing or oversized existing preview feed asset")
            # An existing marker with a lost asset is an incident, not bootstrap.
            gh("release", "download", MARKER, "--repo", REPO, "--pattern", "appcast.xml",
               "--dir", str(root))
            previous = feed.read_feed(root / "appcast.xml")
        feed.publication_version(data, previous, config,
                                 [release["tag_name"] for release in releases])
        archive = root / item["url"].rsplit("/", 1)[-1]
        assets = [asset for asset in version_release["assets"] if asset["name"] == archive.name]
        if (len(assets) != 1 or assets[0]["browser_download_url"] != item["url"]
                or assets[0]["size"] != item["length"]):
            raise ValueError("Published archive asset mismatch")
        gh("release", "download", "v" + item["version"], "--repo", REPO,
           "--pattern", archive.name, "--dir", str(root))
        feed.verify_archive(item, archive, config["public_key"])
        with archive.open("rb") as opened:
            digest = hashlib.file_digest(opened, "sha256").hexdigest()
        if assets[0].get("digest") != "sha256:" + digest:
            raise ValueError("Published asset digest mismatch")
        if marker is None:
            notes = root / "feed-notes.txt"
            notes.write_text("Signed update metadata for Spaceport macOS previews. "
                             "Install the current version from its dated release.\n")
            # The fixed tag is created once. It is never moved or overwritten.
            gh("release", "create", MARKER, str(source.resolve()), "--repo", REPO,
               "--target", expected_sha, "--prerelease", "--title", "macOS preview update feed",
               "--notes-file", str(notes))
        else:
            # GitHub replaces assets using delete/upload, so a brief unavailable
            # feed is possible. Signature verification makes that fail closed.
            gh("release", "upload", MARKER, str(source.resolve()), "--repo", REPO, "--clobber")
        verified = root / "published"
        verified.mkdir()
        gh("release", "download", MARKER, "--repo", REPO, "--pattern", "appcast.xml",
           "--dir", str(verified))
        if feed.read_feed(verified / "appcast.xml") != data:
            raise ValueError("Published feed bytes do not match the signed artifact")
    print(f"Published signed macOS preview feed for {item['version']}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("feed", type=Path)
    parser.add_argument("expected_sha")
    args = parser.parse_args()
    try:
        publish(args.feed, args.expected_sha)
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        parser.exit(1, f"Cannot publish Sparkle feed: {error}\n")


if __name__ == "__main__":
    main()
