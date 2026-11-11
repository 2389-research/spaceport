#!/usr/bin/env python3
# ABOUTME: Writes a temporary Ubuntu CI source list using independent mirrors.
# ABOUTME: Preserves the runner's suites, components, and Ubuntu signing keyring.

import argparse
import re
import sys
from pathlib import Path


MIRRORS = {
    "amd64": {
        "primary": "https://archive.ubuntu.com/ubuntu/",
        "fallback": "https://mirrors.mit.edu/ubuntu/",
    },
    "arm64": {
        "primary": "https://ports.ubuntu.com/ubuntu-ports/",
        "fallback": "https://mirrors.mit.edu/ubuntu-ports/",
    },
}
KEYRING = "/usr/share/keyrings/ubuntu-archive-keyring.gpg"


def rewrite(source: str, architecture: str, mirror: str) -> str:
    if architecture not in MIRRORS:
        raise ValueError(f"unsupported runner architecture: {architecture}")
    origins = {"mirror+file:/etc/apt/apt-mirrors.txt"}
    hosts = ("azure.archive.ubuntu.com/ubuntu", "archive.ubuntu.com/ubuntu", "security.ubuntu.com/ubuntu")
    if architecture == "arm64":
        hosts = ("ports.ubuntu.com/ubuntu-ports",)
    for host in hosts:
        for scheme in ("http", "https"):
            origins.update((f"{scheme}://{host}", f"{scheme}://{host}/"))

    allowed = {"Types", "URIs", "Suites", "Components", "Signed-By", "Architectures"}
    fields = {}
    previous = None
    count = 0
    lines = source.splitlines(keepends=True)

    def validate():
        if not fields:
            return
        required = allowed - {"Architectures"}
        if not required <= fields.keys():
            raise ValueError("missing required Ubuntu source fields")
        if fields["Signed-By"] != KEYRING:
            raise ValueError("expected the runner's Ubuntu archive signing keyring")
        for field, values in (
            ("Types", {"deb", "deb-src"}),
            ("Suites", {"noble", "noble-updates", "noble-backports", "noble-security"}),
            ("Components", {"main", "restricted", "universe", "multiverse"}),
            ("URIs", origins),
            ("Architectures", {architecture, "i386"} if architecture == "amd64" else {architecture}),
        ):
            if field in fields and (not fields[field].split() or not set(fields[field].split()) <= values):
                raise ValueError(f"unsupported Ubuntu source {field}: {fields[field]}")

    for index, line in enumerate(lines):
        if not line.strip():
            validate()
            count += bool(fields)
            fields = {}
            previous = None
            continue
        if line.startswith("#"):
            continue
        if line[0].isspace():
            if previous is None or previous == "URIs":
                raise ValueError("unsupported source continuation")
            fields[previous] += " " + line.strip()
            continue
        match = re.fullmatch(r"([A-Za-z-]+):[ \t]*(.*?)(\r?\n)?", line)
        if match is None:
            raise ValueError("malformed Deb822 source field")
        field, value, ending = match.groups()
        if field not in allowed or field in fields:
            raise ValueError(f"unsupported or repeated source field: {field}")
        fields[field] = value.strip()
        previous = field
        if field == "URIs":
            lines[index] = f"URIs: {MIRRORS[architecture][mirror]}{ending or ''}"
    validate()
    count += bool(fields)
    if count == 0:
        raise ValueError("no Ubuntu source stanzas found")
    return "".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__ or "Rewrite Ubuntu CI mirror URIs; never edit the source file.")
    parser.add_argument("--architecture", required=True, help="runner's dpkg architecture (amd64 or arm64)")
    parser.add_argument("--mirror", choices=("primary", "fallback"), required=True,
                        help="Canonical primary or independent MIT fallback")
    parser.add_argument("source", type=Path, help="runner's Ubuntu Deb822 .sources file")
    args = parser.parse_args()
    try:
        with args.source.open(newline="") as source:
            result = rewrite(source.read(), args.architecture, args.mirror)
    except (OSError, UnicodeError, ValueError) as error:
        print(f"Ubuntu CI sources: {error}", file=sys.stderr)
        return 1
    sys.stdout.write(result)
    return 0


if __name__ == "__main__":
    sys.exit(main())
