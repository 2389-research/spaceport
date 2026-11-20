#!/usr/bin/env python3
"""Rebuild the recorded `spaceport --help` text from the source it prints.

`crates/sheprd-gpui/src/control/client/tests/family_help.rs` compares
`family_help.txt` byte for byte with what `help::usage` builds out of
`cli::control::FAMILIES` (each family's usage) and `help::COMMON`. This script
rebuilds that same text from those same sources, so a change to the help text
can record the new snapshot without a full `spaceport` build.

    python3 scripts/help-snapshot.py --check   # does the file match the source?
    python3 scripts/help-snapshot.py --write   # record the current text
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CONTROL = ROOT / "crates/sheprd-gpui/src/cli/control.rs"
HELP = ROOT / "crates/sheprd-gpui/src/control/client/help.rs"
CLI_CONTROL_DIR = ROOT / "crates/sheprd-gpui/src/cli/control"
SNAPSHOT = ROOT / "crates/sheprd-gpui/src/control/client/tests/family_help.txt"

# The overview's own first line, and the two fixed lines around the list.
HEADER = "spaceport COMMAND ... [--json] [--window ID]\n\n"
LIST_END = "\nRun `spaceport COMMAND --help` for one command's usage.\n\n"
NAME_WIDTH = 14

FAMILY = re.compile(r'Family \{\s*name: "([^"]+)",\s*summary: "([^"]*)",')


def rust_string(literal: str) -> str:
    """A Rust string literal's contents, for the escapes the sources use."""
    out = []
    index = 0
    while index < len(literal):
        char = literal[index]
        if char != "\\":
            out.append(char)
            index += 1
            continue
        index += 1
        escape = literal[index]
        index += 1
        out.append({"n": "\n", "t": "\t", "\\": "\\", '"': '"', "0": "\0"}[escape])
    return "".join(out)


def families() -> list[tuple[str, str]]:
    """Each family's name and the constant naming its usage, in help order."""
    found = FAMILY.findall(CONTROL.read_text())
    assert found, "no families in cli/control.rs"
    return found


def usage_text(constant: str) -> str:
    """The usage a `cli/control/*.rs` constant holds."""
    pattern = re.compile(
        r"const " + constant + r'\s*: &str\s*=\s*"((?:[^"\\]|\\.)*)";', re.DOTALL
    )
    for path in sorted(CLI_CONTROL_DIR.glob("*.rs")):
        match = pattern.search(path.read_text())
        if match:
            return rust_string(match.group(1))
    raise SystemExit(f"no `{constant}` in {CLI_CONTROL_DIR}")


def common() -> str:
    """The shared text every usage ends with."""
    match = re.search(
        r'const COMMON: &str = "\\\n(.*?)";\n', HELP.read_text(), re.DOTALL
    )
    if not match:
        raise SystemExit(f"no COMMON in {HELP}")
    # A backslash-newline continues the literal without a newline; the line
    # breaks that follow are the text's own.
    return rust_string(match.group(1))


def record() -> str:
    families_text = families()
    shared = common()
    text = HEADER
    for name, summary in families_text:
        text += "  " + name.ljust(NAME_WIDTH) + summary + "\n"
    text += LIST_END + shared
    for name, _ in families_text:
        constant = re.search(
            r'Family \{\s*name: "'
            + re.escape(name)
            + r'",(?:.|\n)*?usage: (\w+)::(\w+),',
            CONTROL.read_text(),
        )
        if not constant:
            raise SystemExit(f"no usage constant for {name}")
        text += f"\n\n=== {name} ===\n" + usage_text(constant.group(2))
    return text + "\n"


def main(argv: list[str]) -> int:
    text = record()
    if "--write" in argv:
        SNAPSHOT.write_text(text)
        print(f"wrote {SNAPSHOT.relative_to(ROOT)}")
        return 0
    recorded = SNAPSHOT.read_text()
    if recorded != text:
        import difflib

        sys.stdout.writelines(
            difflib.unified_diff(
                recorded.splitlines(keepends=True),
                text.splitlines(keepends=True),
                "recorded",
                "source",
            )
        )
        return 1
    print(f"{SNAPSHOT.relative_to(ROOT)} matches the source")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
