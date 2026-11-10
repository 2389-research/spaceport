#!/usr/bin/env python3
# ABOUTME: Validate a manual or tag-triggered macOS preview against current Git refs.
# ABOUTME: Emit trusted version and commit outputs before building or exposing credentials.
import os
from pathlib import Path
import re
import subprocess
import sys

VERSION = r'[1-9][0-9]{7}\.[1-9][0-9]*'


def validate(env, repo):
    if env.get('GITHUB_RUN_ATTEMPT') != '1':
        raise ValueError('Workflow rerun refused; inspect receipts and select a new preview version')
    event, ref, sha = (env.get(key, '') for key in ('GITHUB_EVENT_NAME', 'GITHUB_REF', 'GITHUB_SHA'))
    tag = ''
    if event == 'push' and re.fullmatch(rf'refs/tags/v{VERSION}', ref):
        tag = ref.removeprefix('refs/tags/')
        version = tag[1:]
    elif event == 'workflow_dispatch' and ref == 'refs/heads/main':
        version = env.get('INPUT_VERSION', '')
        if env.get('INPUT_SHA') != sha:
            raise ValueError('Manual preview SHA must match the selected main commit')
    else:
        raise ValueError('Expected a version tag push or manual main preview')
    if not re.fullmatch(VERSION, version) or not re.fullmatch(r'[0-9a-f]{40}', sha):
        raise ValueError('Invalid preview version or commit')

    def git(*args):
        return subprocess.check_output(['git', '-C', str(repo), *args], text=True, timeout=60).strip()

    if git('rev-parse', 'HEAD') != sha:
        raise ValueError('Checkout differs from the requested commit')
    refs = ['+refs/heads/main:refs/remotes/origin/main']
    if tag:
        refs.append(f'+refs/tags/{tag}:refs/tags/{tag}')
    git('fetch', '--no-tags', 'origin', *refs)
    if tag and git('rev-parse', f'refs/tags/{tag}^{{commit}}') != sha:
        raise ValueError('Current tag differs from the requested commit')
    ancestry = subprocess.run(['git', '-C', str(repo), 'merge-base', '--is-ancestor', sha,
                               'refs/remotes/origin/main'], timeout=60, check=False)
    if ancestry.returncode:
        raise ValueError('Preview commit must belong to origin/main')
    return dict(version=version, sha=sha, tag=tag)


if __name__ == '__main__':
    try:
        outputs = validate(os.environ, Path.cwd())
        with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
            for key, value in outputs.items():
                output.write(f'{key}={value}\n')
    except (ValueError, KeyError, OSError, subprocess.SubprocessError) as error:
        sys.exit(f'Preview request refused: {error}')
