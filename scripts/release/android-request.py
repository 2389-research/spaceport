#!/usr/bin/env python3
# ABOUTME: Decide whether a published v* preview tag also gets an Android APK, and with which version code.
# ABOUTME: Compares the Android sources with the newest earlier release that already carries an APK.
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys

SPEC = importlib.util.spec_from_file_location('release_common', Path(__file__).with_name('_release_common.py'))
common = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(common)

# Everything packed into the APK: the Kotlin app and the shared Rust controller it embeds.
SOURCES = ('mobile/android', 'mobile/rust')
# Release asset names end with this; it marks a release that already shipped an APK.
APK_SUFFIX = '-android.apk'


def parse(tag):
    """Return (date, counter) for a v* preview tag, or None for any other tag."""
    match = re.fullmatch(rf'v({common.VERSION})', tag)
    if not match:
        return None
    date, counter = match.group(1).split('.')
    return int(date), int(counter)


def version_code(version):
    """Android versionCode YYYYMMDD*100+COUNTER: grows with every later preview tag."""
    date, counter = (int(part) for part in version.split('.'))
    if counter >= 100:
        raise ValueError('Android version codes allow at most 99 previews in one day; counter too large')
    return date * 100 + counter


def plan(repo, tag, sha, published):
    """Validate the tag and report whether its Android sources differ from the last APK release."""
    current = parse(tag)
    if current is None:
        raise ValueError('Expected a v* preview tag such as v20261126.1')
    if not common.is_commit(sha):
        raise ValueError('Invalid commit')

    def git(*args):
        return subprocess.check_output(['git', '-C', str(repo), *args], text=True, timeout=120).strip()

    git('fetch', '--no-tags', 'origin', '+refs/heads/main:refs/remotes/origin/main', f'+refs/tags/{tag}:refs/tags/{tag}')
    if git('rev-parse', f'refs/tags/{tag}^{{commit}}') != sha:
        raise ValueError('Tag does not point at the requested commit')
    ancestry = subprocess.run(['git', '-C', str(repo), 'merge-base', '--is-ancestor', sha, 'refs/remotes/origin/main'],
                              timeout=120, check=False)
    if ancestry.returncode:
        raise ValueError('Preview commit must belong to origin/main')

    earlier = sorted((parse(name), name) for name in published if parse(name) and parse(name) < current)
    baseline = earlier[-1][1] if earlier else ''
    changed = True
    if baseline:
        git('fetch', '--no-tags', 'origin', f'+refs/tags/{baseline}:refs/tags/{baseline}')
        diff = subprocess.run(['git', '-C', str(repo), 'diff', '--quiet', f'refs/tags/{baseline}', sha, '--', *SOURCES],
                              timeout=120, check=False)
        if diff.returncode not in (0, 1):
            raise ValueError(f'Could not compare Android sources with {baseline}')
        changed = diff.returncode == 1
    version = tag[1:]
    return dict(tag=tag, version=version, sha=sha, changed='true' if changed else 'false',
                version_code=str(version_code(version)), baseline=baseline)


def apk_releases(repository):
    """Tags of this repository's releases that already carry an Android APK asset."""
    pages = common.gh('api', '--paginate', '--slurp', f'repos/{repository}/releases?per_page=100')
    return [release['tag_name'] for page in json.loads(pages) for release in page
            if any(asset['name'].endswith(APK_SUFFIX) for asset in release.get('assets', []))]


if __name__ == '__main__':
    try:
        env = os.environ
        outputs = plan(Path.cwd(), env['TAG'], env['SHA'], apk_releases(env['GITHUB_REPOSITORY']))
        with open(env['GITHUB_OUTPUT'], 'a') as output:
            for key, value in outputs.items():
                output.write(f'{key}={value}\n')
        print(f"Android sources {'changed' if outputs['changed'] == 'true' else 'unchanged'} "
              f"since {outputs['baseline'] or 'the first APK release'}")
    except (ValueError, KeyError, OSError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        sys.exit(f'Android release request refused: {error}')
