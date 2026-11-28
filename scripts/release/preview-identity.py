#!/usr/bin/env python3
# ABOUTME: Validate preview identity records without executing a packaged program.
# ABOUTME: Keep bundle metadata consistent with the build's app ID, version, profile and channel.
import mmap
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
PREFIX = b'\0SPACEPORT_PREVIEW_V1\n'


# Universal Mach-O headers, with 32-bit or 64-bit offsets in either byte order.
FAT_HEADERS = {bytes.fromhex(value) for value in ('cafebabe', 'bebafeca', 'cafebabf', 'bfbafeca')}


def check(binary, version, profile):
    with open(binary, 'rb') as stream:
        universal = stream.read(4) in FAT_HEADERS
    if universal:
        archs = subprocess.check_output(['lipo', '-archs', binary], text=True).split()
        if not archs:
            raise ValueError(f'Universal preview executable has no architecture: {binary}')
        with tempfile.TemporaryDirectory() as directory:
            thin = Path(directory) / 'slice'
            for arch in archs:
                subprocess.run(['lipo', binary, '-thin', arch, '-output', thin], check=True)
                check_slice(thin, version, profile)
    else:
        check_slice(binary, version, profile)


def check_slice(binary, version, profile):
    app_id = (ROOT / 'crates/spaceport-gpui/macos-preview-app-id').read_text().strip()
    expected = f'id={app_id}\nversion={version}\nprofile={profile}\nchannel=macos-preview\n'.encode()
    with open(binary, 'rb') as stream, mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_READ) as data:
        records = []
        offset = data.find(PREFIX)
        while offset != -1:
            start = offset + len(PREFIX)
            end = data.find(b'\0', start, start + 512)
            records.append(data[start:end] if end != -1 else None)
            offset = data.find(PREFIX, start)
        if not records or any(record != expected for record in records):
            raise ValueError(f'Preview binary identity, version, profile or channel mismatch: {binary}')


if __name__ == '__main__':
    try:
        if len(sys.argv) != 4 or sys.argv[2] not in ('debug', 'release'):
            raise ValueError('Usage: preview-identity.py VERSION debug|release BINARY')
        check(sys.argv[3], sys.argv[1], sys.argv[2])
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
