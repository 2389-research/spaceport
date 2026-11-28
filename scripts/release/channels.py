#!/usr/bin/env python3
"""ABOUTME: Name each release channel's release, appcast text and downloadable artifacts.
ABOUTME: Keep the feed address in sparkle.json and derive every artifact URL from one host.
"""

import importlib.util
import json
from dataclasses import dataclass
from pathlib import Path
import re
import urllib.parse

ROOT = Path(__file__).resolve().parents[2]
CHANNELS = ROOT / "scripts/release/channels.json"
SPARKLE = ROOT / "scripts/release/sparkle.json"
MAX_CHANNELS = 8
MAX_DOWNLOADS = 8
MAX_FILE = 64 * 1024
MAX_TEXT = 200
# The appcast's one enclosure is the macOS disk image the updater installs.
APPCAST_PLATFORM = "macos"
APPCAST_FORMAT = "dmg"
# Both channels publish the calendar version behind one immutable v tag.
TAG = "v{version}"
REPOSITORY = re.compile(r"[A-Za-z0-9._-]+/[A-Za-z0-9._-]+")
TOKEN = re.compile(r"[a-z0-9][a-z0-9._-]*")
PROBE = "20260101.1"


def _sibling(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_common = _sibling("release_common", Path(__file__).with_name("_release_common.py"))
_framework = _sibling("sparkle_framework", Path(__file__).with_name("sparkle_framework.py"))


def is_version(version):
    """Accept only the calendar release version, without its tag prefix."""
    return _common.is_version(version)


def is_release_tag(tag):
    """Accept only a published release tag: v and a calendar version."""
    return isinstance(tag, str) and tag.startswith("v") and _common.is_version(tag[1:])


def release_tag(version):
    if not _common.is_version(version):
        raise ValueError("Version must be YYYYMMDD.COUNTER without a v prefix or leading zeros")
    return TAG.format(version=version)


def _text(value, field, template):
    if not isinstance(value, str) or not value or len(value) > MAX_TEXT:
        raise ValueError(f"{field} must be a nonempty bounded string")
    if template and "{version}" not in value:
        raise ValueError(f"{field} must carry {{version}}")
    try:
        formatted = value.format(version=PROBE)
    except (IndexError, KeyError, ValueError) as error:
        raise ValueError(f"{field} may only name {{version}}") from error
    if formatted != value.replace("{version}", PROBE):
        raise ValueError(f"{field} may only name {{version}}")
    if formatted != formatted.strip() or any(character < " " or character == "\x7f" for character in formatted):
        raise ValueError(f"{field} must be one printable line")
    return value


def _download(name, entry):
    fields = {"name", "kind", "label", "platform", "architecture", "format", "minimum_system_version"}
    if not isinstance(entry, dict) or set(entry) != fields:
        raise ValueError(f"Release download must name {', '.join(sorted(fields))}: {name}")
    for field in fields:
        if not isinstance(entry[field], str):
            raise ValueError(f"Release download {field} must be a string: {name}")
    for field in ("kind", "platform", "architecture", "format", "minimum_system_version"):
        if not TOKEN.fullmatch(entry[field]):
            raise ValueError(f"Release download {field} must be a lower-case token: {name}/{field}")
    asset = _text(entry["name"], f"download name {name}", template=True)
    label = _text(entry["label"], f"download label {name}", template=False)
    if "/" in asset or "\\" in asset or not asset.format(version=PROBE).endswith(f".{entry['format']}"):
        raise ValueError(f"Download name must be one file of its format: {name}")
    return Download(
        name=asset,
        kind=entry["kind"],
        label=label,
        platform=entry["platform"],
        architecture=entry["architecture"],
        format=entry["format"],
        minimum_system_version=entry["minimum_system_version"],
    )


@dataclass(frozen=True)
class Download:
    """One published artifact: the file a release page carries and an appcast points at."""

    name: str
    kind: str
    label: str
    platform: str
    architecture: str
    format: str
    minimum_system_version: str

    def asset(self, version):
        release_tag(version)
        return self.name.format(version=version)


@dataclass(frozen=True)
class Channel:
    """What one channel publishes: its release, its appcast text and its downloads."""

    name: str
    prerelease: bool
    title: str
    appcast_title: str
    appcast_item: str
    feed_release: str
    feed_url: str
    base_url: str
    downloads: tuple

    @property
    def appcast_download(self):
        candidates = [download for download in self.downloads
                      if download.platform == APPCAST_PLATFORM and download.format == APPCAST_FORMAT]
        if len(candidates) != 1:
            raise ValueError(f"Channel must name exactly one {APPCAST_PLATFORM} {APPCAST_FORMAT}: {self.name}")
        return candidates[0]

    @property
    def releases_url(self):
        return f"{self.base_url}/releases"

    @property
    def download_page_url(self):
        """The page a download link opens; GitHub has no latest release among prereleases."""
        return f"{self.base_url}/releases" if self.prerelease else f"{self.base_url}/releases/latest"

    def release_title(self, version):
        release_tag(version)
        return self.title.format(version=version)

    def item_title(self, version):
        release_tag(version)
        return self.appcast_item.format(version=version)

    def release_url(self, version):
        return f"{self.base_url}/releases/tag/{release_tag(version)}"

    def download_url(self, version, download=None):
        download = download or self.appcast_download
        return f"{self.base_url}/releases/download/{release_tag(version)}/{download.asset(version)}"


def _channel(name, entry, feed, base_url):
    fields = {"release", "appcast", "downloads"}
    if not isinstance(entry, dict) or set(entry) != fields:
        raise ValueError(f"Release channel must name {', '.join(sorted(fields))}: {name}")
    release = entry["release"]
    if not isinstance(release, dict) or set(release) != {"prerelease", "title"}:
        raise ValueError(f"Release channel release must name prerelease and title: {name}")
    if type(release["prerelease"]) is not bool:
        raise ValueError(f"Release channel prerelease must be a bool: {name}")
    appcast = entry["appcast"]
    if not isinstance(appcast, dict) or set(appcast) != {"title", "item"}:
        raise ValueError(f"Release channel appcast must name title and item: {name}")
    downloads = entry["downloads"]
    if not isinstance(downloads, list) or not 1 <= len(downloads) <= MAX_DOWNLOADS:
        raise ValueError(f"Release channel must name between one and {MAX_DOWNLOADS} downloads: {name}")
    table = tuple(_download(f"{name}[{index}]", download) for index, download in enumerate(downloads))
    if len({download.name for download in table}) != len(table):
        raise ValueError(f"Release channel names one download twice: {name}")
    if len({(d.platform, d.architecture, d.format) for d in table}) != len(table):
        raise ValueError(f"Release channel names one download target twice: {name}")
    if sum(1 for d in table if (d.platform, d.format) == (APPCAST_PLATFORM, APPCAST_FORMAT)) != 1:
        raise ValueError(f"Release channel must name exactly one {APPCAST_PLATFORM} {APPCAST_FORMAT}: {name}")
    channel = Channel(
        name=name,
        prerelease=release["prerelease"],
        title=_text(release["title"], f"release title {name}", template=True),
        appcast_title=_text(appcast["title"], f"appcast title {name}", template=False),
        appcast_item=_text(appcast["item"], f"appcast item {name}", template=True),
        feed_release=feed["release"],
        feed_url=feed["url"],
        base_url=base_url,
        downloads=table,
    )
    return channel


def load(path=CHANNELS, sparkle=SPARKLE):
    """Read the release channel table, keyed by the app's own channel names."""
    try:
        with Path(path).open("rb") as opened:
            data = opened.read(MAX_FILE + 1)
    except OSError as error:
        raise ValueError(f"Release channel table unreadable: {error}") from error
    if len(data) > MAX_FILE:
        raise ValueError("Release channel table exceeds size bound")
    try:
        table = json.loads(data)
    except (ValueError, UnicodeError) as error:
        raise ValueError("Release channel table is malformed JSON") from error
    if not isinstance(table, dict) or set(table) != {"repository", "url", "channels"}:
        raise ValueError("Release channel table must name repository, url and channels")
    repository, base_url = table["repository"], table["url"]
    if not isinstance(repository, str) or not REPOSITORY.fullmatch(repository):
        raise ValueError("Release channel repository must be owner/name")
    split = urllib.parse.urlsplit(base_url) if isinstance(base_url, str) else None
    if (split is None or split.scheme != "https" or not split.hostname or split.username
            or split.password or split.query or split.fragment or split.path != f"/{repository}"):
        raise ValueError("Release channel url must be https://HOST/owner/name")
    feeds = _framework.read_config(sparkle)["feeds"]
    # The artifacts are released from the repository the feeds are, on one host.
    if split.hostname not in {urllib.parse.urlsplit(feed["url"]).hostname for feed in feeds.values()}:
        raise ValueError("Release channel url must serve the channel feeds")
    for name, feed in feeds.items():
        if not urllib.parse.urlsplit(feed["url"]).path.startswith(f"/{repository}/releases/download/"):
            raise ValueError(f"Sparkle feed must be a release asset of the channel repository: {name}")
    entries = table["channels"]
    if not isinstance(entries, dict) or sorted(entries) != sorted(feeds):
        raise ValueError(f"Release channel table must name the app's channels: {', '.join(sorted(feeds))}")
    if len(entries) > MAX_CHANNELS:
        raise ValueError("Release channel table names too many channels")
    return {name: _channel(name, entries[name], feeds[name], base_url)
            for name in sorted(entries)}


def channel(name, path=CHANNELS, sparkle=SPARKLE):
    """One channel, refused by name when the table does not publish it."""
    table = load(path, sparkle)
    if name not in table:
        raise ValueError(f"Unknown release channel {name}; expected one of {', '.join(sorted(table))}")
    return table[name]
