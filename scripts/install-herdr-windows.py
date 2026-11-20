#!/usr/bin/env python3
# ABOUTME: Stages checksum-pinned Windows runtimes for isolated native tests.
# ABOUTME: Preserves required ConPTY files without changing user installation state.
"""Stage a pinned Windows runtime for native CI tests.

Usage: python install-herdr-windows.py [--core] DESTINATION_DIR

The default stages Herdr 0.9.5 for HERDR_TEST_BINARY. --core stages Core 0.1.21
for HERDR_CORE_TEST_BINARY, including its ConPTY files and license notices.
Prints the exact executable path. Does not run an installer or change PATH.

These Windows releases are x64-only. The Core fixture targets Windows x64.
The default Herdr archive also carries an ARM64 OpenConsole; existing ARM
fixtures retain their live-input exclusion. Bump versions and digests together.
"""
import argparse
import hashlib
from pathlib import Path, PureWindowsPath
import sys
import tempfile
import urllib.request
import zipfile

RELEASES = {
    # keep-sorted start
    "core": (
        "https://releases.herdr.dev/core/0.1.21/herdr-core-windows-x86_64.zip",
        "a8984634c35b58ba82d0aec078d5a5e6004da01d5a25b3daf822ab5e31c7312b",
        ("herdr-core.exe", "conpty/conpty.dll", "conpty/x64/OpenConsole.exe", "LICENSE",
         "THIRD-PARTY-NOTICES/Microsoft.Windows.Console.ConPTY-LICENSE.txt"),
    ),
    "herdr": (
        "https://github.com/herdrdev/herdr/releases/download/v0.9.5/herdr-windows-x86_64.zip",
        "d2c6c9c971c43a0bfdde8c692a9a1a7a45c3e4807cd01ae45662b4734fac1cbf",
        ("herdr.exe",),
    ),
    # keep-sorted end
}
LIMIT = 64 * 1024 * 1024


def check_destination(destination):
    destination = Path(destination)
    if destination.exists() and any(destination.iterdir()):
        raise ValueError(f"{destination} is not empty")
    return destination


def unpack(archive, destination, expected_digest, required):
    """Validate the complete Windows package before extracting its first file."""
    destination = check_destination(destination)
    digest = hashlib.sha256()
    with Path(archive).open("rb") as source:
        total = 0
        while chunk := source.read(1024 * 1024):
            total += len(chunk)
            if total > LIMIT:
                raise ValueError("Runtime archive exceeds the size limit")
            digest.update(chunk)
    if digest.hexdigest() != expected_digest:
        raise ValueError(f"Runtime archive digest mismatch: {digest.hexdigest()}")
    with zipfile.ZipFile(archive) as zipped:
        for member in zipped.infolist():
            path = PureWindowsPath(member.filename)
            if not member.filename or path.root or path.drive or ".." in path.parts or ":" in member.filename:
                raise ValueError(f"unsafe archive member: {member.filename}")
        files = {member.filename for member in zipped.infolist() if not member.is_dir()}
        for member in required:
            if member not in files:
                raise ValueError(f"Runtime archive is missing required file: {member}")
        destination.mkdir(parents=True, exist_ok=True)
        zipped.extractall(destination)
    return (destination / required[0]).resolve()


def install(destination, runtime="herdr"):
    destination = check_destination(destination)
    url, digest, required = RELEASES[runtime]
    with tempfile.TemporaryDirectory(prefix="herdr-download-") as temp:
        archive = Path(temp) / "herdr.zip"
        request = urllib.request.Request(url, headers={"User-Agent": "Spaceport-native-fixture/1"})
        with urllib.request.urlopen(request, timeout=120) as response, archive.open("xb") as output:
            total = 0
            while chunk := response.read(1024 * 1024):
                total += len(chunk)
                if total > LIMIT:
                    raise ValueError("Runtime archive exceeds the size limit")
                output.write(chunk)
        return unpack(archive, destination, digest, required)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--core", action="store_true", help="stage the pinned Core fixture")
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    try:
        print(install(args.destination, "core" if args.core else "herdr"))
    except (OSError, ValueError, zipfile.BadZipFile) as error:
        sys.exit(f"Runtime fixture staging failed: {error}")


if __name__ == "__main__":
    main()
