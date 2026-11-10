#!/usr/bin/env python3
# ABOUTME: Select a published ancestor as the macOS preview changelog boundary.
# ABOUTME: Use the recorded fork baseline before the first published release.
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

TAG = re.compile(r'v[1-9][0-9]{7}\.[1-9][0-9]*')


def select_base(repo, baseline, tags):
    if not re.fullmatch(r'[0-9a-f]{40}', baseline):
        raise ValueError('Invalid fork baseline SHA')

    def git(*args):
        return subprocess.check_output(['git', '-C', str(repo), *args],
                                       stderr=subprocess.PIPE, text=True, timeout=30).strip()

    def ancestor(older, newer):
        result = subprocess.run(['git', '-C', str(repo), 'merge-base', '--is-ancestor', older, newer],
                                capture_output=True, timeout=30)
        if result.returncode not in (0, 1):
            result.check_returncode()
        return result.returncode == 0

    if not ancestor(baseline, 'HEAD'):
        raise ValueError('Fork baseline does not belong to this release history')
    versions = sorted({tag for tag in tags if isinstance(tag, str) and TAG.fullmatch(tag)},
                      key=lambda tag: tuple(map(int, tag[1:].split('.'))), reverse=True)
    for tag in versions:
        commit = git('rev-parse', '--verify', f'refs/tags/{tag}^{{commit}}')
        if ancestor(baseline, commit) and ancestor(commit, 'HEAD'):
            return commit
    return baseline


def main():
    argparse.ArgumentParser(description='Select the published ancestor for Spaceport preview notes.').parse_args()
    repo = Path(__file__).resolve().parents[2]
    baseline = json.loads((repo / 'docs/spaceport/evidence/t00-dependencies.json').read_text())['source_revision']
    raw = subprocess.check_output(['gh', 'api', '--paginate', '--slurp',
                                   'repos/harperreed/spaceport/releases?per_page=100'], timeout=60)
    if len(raw) > 8 * 1024 * 1024:
        raise ValueError('Published release history is oversized')
    pages = json.loads(raw)
    if not isinstance(pages, list) or any(not isinstance(page, list) for page in pages):
        raise ValueError('Malformed published release history')
    tags = []
    for page in pages:
        for release in page:
            if (not isinstance(release, dict) or type(release.get('draft')) is not bool
                    or not isinstance(release.get('tag_name'), str)):
                raise ValueError('Malformed published release record')
            if not release['draft']:
                tags.append(release['tag_name'])
    print(select_base(repo, baseline, tags))


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        sys.exit(f'Cannot select preview notes boundary: {error}')
