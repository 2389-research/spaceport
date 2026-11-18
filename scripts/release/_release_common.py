"""ABOUTME: Share the pinned preview formats and the gh invocation between release scripts.
ABOUTME: Load by file path like the sibling scripts, since dashed names are not modules.
"""
import re
import subprocess

# Calendar preview version YYYYMMDD.COUNTER: eight-digit UTC date, same-day
# counter from 1, without `v`, suffixes, or leading zeros.
VERSION = r'[1-9][0-9]{7}\.[1-9][0-9]*'
# Full lowercase commit SHA, as Git and the GitHub API report it.
COMMIT = r'[0-9a-f]{40}'


def is_version(value):
    """Accept only a calendar preview version without its `v` tag prefix."""
    return re.fullmatch(VERSION, value) is not None


def is_commit(value):
    """Accept only a full lowercase 40-character commit SHA."""
    return re.fullmatch(COMMIT, value) is not None


def gh(*args, timeout=120):
    """Return gh's stdout bytes, capturing stderr and failing on a nonzero exit."""
    return subprocess.check_output(['gh', *args], stderr=subprocess.PIPE, timeout=timeout)
