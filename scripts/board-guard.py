#!/usr/bin/env python3
"""ABOUTME: Guard against double-wrapped task-row writes on an artifact-host data collection.
ABOUTME: Every PUT to data/tasks must carry the bare record, never an API envelope.

Usage: guard-task-put.py put-body.json   # exits 1 if the body is an envelope
       guard-task-put.py list-response.json --unwrap  # prints the bare record to stdout

An envelope is any object whose top-level keys include 'commit_revision', 'revision'
or 'value'. Those are GET-response fields; a record's own fields are title/body/status/
owner/done_note and friends. The 2026-11-25 incident (spaceport-crew, 36 rows) came
from piping a GET response into a PUT; this guard makes that mistake impossible to
ship silently.
"""
import json
import sys

ENVELOPE_KEYS = {"commit_revision", "revision", "value"}


def is_envelope(record: dict) -> bool:
    return ENVELOPE_KEYS & set(record) != set()


def unwrap(value):
    while isinstance(value, dict) and is_envelope(value):
        value = value.get("value", value)
    return value


def main() -> int:
    args = sys.argv[1:]
    if not args:
        print(__doc__, file=sys.stderr)
        return 2
    try:
        data = json.load(open(args[0]))
    except (OSError, ValueError) as error:
        print(f"guard: cannot read {args[0]}: {error}", file=sys.stderr)
        return 2
    if "--unwrap" in args:
        print(json.dumps(unwrap(data), indent=2, ensure_ascii=False))
        return 0
    if not isinstance(data, dict):
        print("guard: PUT body must be a JSON object (the bare record)", file=sys.stderr)
        return 1
    if is_envelope(data):
        found = sorted(ENVELOPE_KEYS & set(data))
        print(f"guard: REFUSED - PUT body is an envelope (contains {', '.join(found)}). "
              "PUT response['value'] (the bare record), not the response. "
              "See artifacts.folk-fujita.ts.net/p/spaceport-crew-incident/", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
