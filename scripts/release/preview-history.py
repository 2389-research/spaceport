#!/usr/bin/env python3
# ABOUTME: Refuse hosted preview versions already present in this workflow's history.
# ABOUTME: Read bounded complete GitHub history before build or signing credentials exist.
import json
import os
import re
import subprocess
import sys

VERSION = r'[1-9][0-9]{7}\.[1-9][0-9]*'
TITLE = re.compile(rf'macOS preview ({VERSION}) at ([0-9a-f]{{40}})')
MAX_RUNS = 1000
PER_PAGE = 100


def check(version, sha, run_id, attempt, read_page):
    if attempt != '1':
        raise ValueError('Workflow rerun refused; inspect receipts and select a new preview version')
    if not re.fullmatch(VERSION, version) or not re.fullmatch(r'[0-9a-f]{40}', sha) or not run_id.isdecimal():
        raise ValueError('Invalid preview version, commit or run ID')
    current = int(run_id)
    seen = set()
    total = None
    for page_number in range(1, MAX_RUNS // PER_PAGE + 1):
        page = read_page(page_number)
        count = page.get('total_count')
        runs = page.get('workflow_runs')
        if type(count) is not int or not 1 <= count <= MAX_RUNS or not isinstance(runs, list):
            raise ValueError('Workflow history coverage uncertain or exceeds the 1000-run bound')
        if total is not None and count != total:
            raise ValueError('Workflow history changed during verification')
        total = count
        if len(runs) != min(PER_PAGE, total - len(seen)):
            raise ValueError('Workflow history is truncated')
        for run in runs:
            if not isinstance(run, dict):
                raise ValueError('Workflow history has a malformed record')
            identity = run.get('id')
            match = TITLE.fullmatch(run.get('display_title', ''))
            if type(identity) is not int or identity in seen or match is None:
                raise ValueError('Workflow history has a duplicate or unrecognized record')
            seen.add(identity)
            if identity == current:
                if match.groups() != (version, sha):
                    raise ValueError('Current workflow title does not match the selected preview')
            elif match[1] == version:
                raise ValueError(f'Preview version already reserved by workflow run {identity}')
        if len(seen) == total:
            if current not in seen:
                raise ValueError('Current workflow missing from history')
            return
    raise ValueError('Workflow history coverage uncertain')


def read_page(page):
    # GitHub documents id, display_title, total_count and this paged endpoint:
    # https://docs.github.com/en/rest/actions/workflow-runs#list-workflow-runs-for-a-workflow
    endpoint = f'repos/harperreed/spaceport/actions/workflows/macos-preview.yml/runs?per_page={PER_PAGE}&page={page}'
    try:
        result = subprocess.run(['gh', 'api', endpoint], capture_output=True, timeout=30, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise ValueError('GitHub workflow history unavailable') from error
    if result.returncode or len(result.stdout) > 8 * 1024 * 1024:
        raise ValueError('GitHub workflow history unavailable or oversized')
    try:
        page = json.loads(result.stdout)
    except (ValueError, UnicodeError) as error:
        raise ValueError('GitHub returned malformed workflow history') from error
    if not isinstance(page, dict):
        raise ValueError('GitHub returned malformed workflow history')
    return page


if __name__ == '__main__':
    try:
        check(os.environ.get('VERSION', ''), os.environ.get('EXPECTED_SHA', ''),
              os.environ.get('GITHUB_RUN_ID', ''), os.environ.get('GITHUB_RUN_ATTEMPT', ''), read_page)
    except ValueError as error:
        sys.exit(str(error))
