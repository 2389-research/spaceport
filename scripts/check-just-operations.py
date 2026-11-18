#!/usr/bin/env python3
"""Fail when a justfile recipe or a documented-ops table drifts out of sync.

Two drift classes, both of which silently lose documentation today:

1. A recipe added to the justfile but never documented in docs/just-operations.md
   (or documented under a stale signature), so agents never discover it and the
   operation is re-derived — possibly subtly wrong — by the next person.
2. A recipe deleted or renamed while docs still name it, so the guide points at
   operations that no longer exist.

The check parses the justfile directly (regex over recipe headers), so no `just`
invocation or subprocess is needed and the gate stays fast. Recipes beginning
with `_` are private by convention and exempt.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
JUSTFILE = ROOT / "justfile"
GUIDE = ROOT / "docs" / "just-operations.md"

# A recipe header: optional parameters, then ':' at line end.
RECIPE = re.compile(r"^@?([a-zA-Z][a-zA-Z0-9_-]*)((?:\s+[$*]?[a-zA-Z_][a-zA-Z0-9_]*(?:=\"[^\"]*\")?)*):(?:\s|$)")

# Recipe names referenced inside the guide, e.g. `just test-gpui filter reps`
# or `just ci`. The match is anchored at a word boundary so `run-debug` does
# not match a reference to `run`.
REFERENCE = re.compile(r"`just\s+([a-zA-Z][a-zA-Z0-9_-]+)(?![-_a-zA-Z0-9])")


def _split_params(text: str) -> list[str]:
    """Parameter words, keeping quoted default values whole."""
    words: list[str] = []
    current = ""
    in_quotes = False
    for char in text:
        if char == '"':
            in_quotes = not in_quotes
            current += char
        elif char == " " and not in_quotes:
            if current:
                words.append(current)
                current = ""
        else:
            current += char
    if current:
        words.append(current)
    return words


def parse_justfile() -> dict[str, list[str]]:
    """Recipe name -> parameter names, from the justfile's own syntax."""
    recipes: dict[str, list[str]] = {}
    current: str | None = None
    for raw in JUSTFILE.read_text(encoding="utf-8").splitlines():
        line = raw.rstrip()
        if not line or line.lstrip().startswith("#"):
            continue
        if line.startswith((" ", "\t")):
            continue  # recipe body
        match = RECIPE.match(line)
        if match:
            current = match.group(1)
            if not current.startswith("_"):
                params = _split_params(match.group(2))
                recipes[current] = [p.lstrip("$*") for p in params]
        else:
            current = None
    return recipes


def parse_guide() -> tuple[set[str], set[str]]:
    """(names referenced in the guide, names given a table row)."""
    text = GUIDE.read_text(encoding="utf-8")
    referenced = set(REFERENCE.findall(text))
    # A recipe is "documented" when a guide line mentions `just <name>` anywhere.
    return referenced, referenced


def main() -> int:
    if not JUSTFILE.is_file():
        print(f"error: {JUSTFILE} not found", file=sys.stderr)
        return 2
    if not GUIDE.is_file():
        print(
            f"error: {GUIDE} not found — every justfile recipe must be documented "
            "there (see AGENTS.md, Codified operations)",
            file=sys.stderr,
        )
        return 2

    recipes = parse_justfile()
    referenced, _ = parse_guide()

    problems: list[str] = []

    undocumented = sorted(name for name in recipes if name not in referenced)
    for name in undocumented:
        params = recipes[name]
        signature = f"{name} {' '.join(params)}".rstrip()
        problems.append(
            f"{name}: recipe is not documented in docs/just-operations.md — "
            f"add a row naming `just {signature}` so agents can discover it"
        )

    stale = sorted(name for name in referenced if name not in recipes)
    for name in stale:
        problems.append(
            f"{name}: docs/just-operations.md references a recipe the justfile "
            "does not define — update or remove the guide row"
        )

    if problems:
        print("Codified-operations documentation is out of date:", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        print(
            "\nFix: add or correct the rows in docs/just-operations.md "
            "(or run `just --list` to see the recipes).",
            file=sys.stderr,
        )
        return 1

    print(f"{len(recipes)} recipes all documented in docs/just-operations.md.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
