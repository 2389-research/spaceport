#!/usr/bin/env python3
# ABOUTME: Validate preview identity records without executing a packaged program.
# ABOUTME: Keep bundle metadata consistent with the build's app ID, version and profile.
import mmap
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
PREFIX = b'\0SPACEPORT_PREVIEW_V1\n'


def check(binary, version, profile):
    app_id = (ROOT / 'crates/sheprd-gpui/macos-preview-app-id').read_text().strip()
    expected = f'id={app_id}\nversion={version}\nprofile={profile}\n'.encode()
    with open(binary, 'rb') as stream, mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_READ) as data:
        records = []
        offset = data.find(PREFIX)
        while offset != -1:
            start = offset + len(PREFIX)
            end = data.find(b'\0', start, start + 512)
            records.append(data[start:end] if end != -1 else None)
            offset = data.find(PREFIX, start)
        if not records or any(record != expected for record in records):
            raise ValueError(f'Preview binary identity/version/profile mismatch: {binary}')


if __name__ == '__main__':
    try:
        if len(sys.argv) != 4 or sys.argv[2] not in ('debug', 'release'):
            raise ValueError('Usage: preview-identity.py VERSION debug|release BINARY')
        check(sys.argv[3], sys.argv[1], sys.argv[2])
    except (OSError, ValueError) as error:
        sys.exit(str(error))
