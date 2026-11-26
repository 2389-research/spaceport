#!/usr/bin/env python3
# ABOUTME: Decide whether a published v* preview tag also gets an Android APK, and with which version code.
# ABOUTME: Compares the APK's sources with the newest earlier release that already carries an APK.
import importlib.util
import json
import os
import posixpath
from pathlib import Path
import re
import subprocess
import sys
import tomllib

SPEC = importlib.util.spec_from_file_location('release_common', Path(__file__).with_name('_release_common.py'))
common = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(common)

# The Kotlin app, and the Rust controller crate it embeds. apk_sources adds the shared crates
# that crate reaches, the root manifest they inherit versions from, and the pinned toolchain.
APP = 'mobile/android'
CONTROLLER = 'mobile/rust'
BUILD_INPUTS = ('Cargo.toml', 'rust-toolchain.toml')
# Dependency tables whose crates are compiled into the library; dev-dependencies are not.
SHIPPED = ('dependencies', 'build-dependencies')
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


def apk_sources(repo, sha):
    """Repository paths whose contents reach the APK at commit sha, following Cargo path dependencies."""
    def manifest(directory):
        path = posixpath.join(directory, 'Cargo.toml') if directory else 'Cargo.toml'
        text = subprocess.check_output(['git', '-C', str(repo), 'show', f'{sha}:{path}'], text=True, timeout=120)
        return tomllib.loads(text)

    shared = manifest('').get('workspace', {}).get('dependencies', {})
    crates, pending = set(), [CONTROLLER]
    while pending:
        crate = pending.pop()
        if crate in crates:
            continue
        crates.add(crate)
        data = manifest(crate)
        tables = [data] + list(data.get('target', {}).values())
        for table in tables:
            for kind in SHIPPED:
                for name, spec in table.get(kind, {}).items():
                    if not isinstance(spec, dict):
                        continue
                    if 'path' in spec:
                        pending.append(posixpath.normpath(posixpath.join(crate, spec['path'])))
                    elif spec.get('workspace') and isinstance(shared.get(name), dict) and 'path' in shared[name]:
                        pending.append(posixpath.normpath(shared[name]['path']))
    if any(crate.startswith('..') for crate in crates):
        raise ValueError('A controller dependency lies outside the repository')
    return sorted(crates | {APP, *BUILD_INPUTS})


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
        sources = apk_sources(repo, sha)
        diff = subprocess.run(['git', '-C', str(repo), 'diff', '--quiet', f'refs/tags/{baseline}', sha, '--', *sources],
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
        # The checkout of the tagged commit is passed in; this script itself always comes from main.
        outputs = plan(Path(sys.argv[1]), env['TAG'], env['SHA'], apk_releases(env['GITHUB_REPOSITORY']))
        with open(env['GITHUB_OUTPUT'], 'a') as output:
            for key, value in outputs.items():
                output.write(f'{key}={value}\n')
        print(f"Android sources {'changed' if outputs['changed'] == 'true' else 'unchanged'} "
              f"since {outputs['baseline'] or 'the first APK release'}")
    except (ValueError, KeyError, IndexError, OSError, subprocess.SubprocessError, json.JSONDecodeError,
            tomllib.TOMLDecodeError) as error:
        sys.exit(f'Android release request refused: {error}')
