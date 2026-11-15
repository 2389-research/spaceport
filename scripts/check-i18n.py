#!/usr/bin/env python3
"""Check how the app uses its Fluent messages (docs/i18n.md).

Whether a `crates/*/locales/<tag>/<feature>.ftl` is a valid file is the
build's to say, not this script's: `crates/sheprd-gpui/locales.rs` parses
each one with Fluent's own parser and fails the build when a file does not
parse, an ID lacks its file's feature prefix or repeats within a language,
or a translation has an ID English lacks, and the compiler rejects an `fl!`
that names no English message. This script reads only message IDs and
references from files the build accepts: a message starts a line with its
ID, and Fluent indents every line that continues one.

Every English message is used: the app's Rust code names it as a string
literal (`fl!("id")`, or a table such as the command catalog's), or another
message references it. Tests do not count, so a message only a test names is
reported.

A module whose code shows only messages carries the line `// i18n: migrated`.
That file, and every file in the directory of the same name, may then hold
no string literal that reads as English (a capitalized word, or two words
with a space between), except where it is not shown: logs, panics,
attributes such as `#[error]`, `write!` in a `Display` for logs, key
contexts and bindings. A literal that is data, such as a font or a theme
name, takes `// i18n: data` at the end of its line or on a line of its own
just above it.

The platform files name the same languages as the app: the macOS bundle's
`CFBundleLocalizations` lists every language directory, each of which
names itself in `language-name`, and its development region is `en-US`;
every language has an installer `scripts/release/installer/<tag>.isl` and a
desktop entry `Comment[tag]` (the plain `Comment` is English); and each
`assets/macos/<tag>.lproj`, which is optional, installer file, and
`Comment[tag]` is for one of them.

    python3 scripts/check-i18n.py
"""

from __future__ import annotations

import plistlib
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

FALLBACK = "en-US"
IDENTIFIER = r"[a-zA-Z][a-zA-Z0-9_-]*"
MESSAGE = re.compile(rf"^({IDENTIFIER})\s*=")
REFERENCE = re.compile(rf"\{{\s*({IDENTIFIER})(?:\.{IDENTIFIER})?\s*\}}")
LITERAL = re.compile(r"\"([a-z][a-z0-9-]*)\"")
LOCALIZED_KEY = re.compile(r"^\w+\[([^\]]+)\]=", re.MULTILINE)
# The message each language names itself by in the Settings language picker.
ENDONYM = "language-name"
INFO_PLIST = Path("assets/macos/Info.plist")
INSTALLER = Path("scripts/release/installer")
DESKTOP_ENTRY = Path("scripts/release/org.sheprd.Sheprd.desktop")
MIGRATED = "// i18n: migrated"
DATA = "// i18n: data"
# A capitalized word, or two lowercase words with a space between, outside
# format placeholders.
ENGLISH = re.compile(r"\b[A-Z][a-z]+|\b[a-z]{2,} [a-z]{2,}\b")
PLACEHOLDER = re.compile(r"\{[^{}]*\}")
# Calls whose string arguments are never shown, by the name before their
# opening bracket. `#` is an attribute.
NOT_SHOWN = {
    "#",
    "actions!",
    "anyhow!",
    "assert!",
    "assert_eq!",
    "assert_ne!",
    "bail!",
    "concat!",
    "context",
    "debug!",
    "debug_assert!",
    "debug_assert_eq!",
    "debug_assert_ne!",
    "debug_selector",
    "ensure!",
    "env!",
    "error!",
    "expect",
    "include_str!",
    "info!",
    "key_context",
    "KeyBinding::new",
    "panic!",
    "trace!",
    "todo!",
    "unreachable!",
    "warn!",
    "with_context",
    "write!",
    "writeln!",
}


@dataclass
class Ftl:
    path: Path
    language: str
    feature: str
    # ID -> line, in file order.
    messages: dict[str, int] = field(default_factory=dict)
    references: set[str] = field(default_factory=set)


@dataclass
class Literal:
    line: int
    text: str
    # The calls and brackets around it, innermost last.
    around: list[str]


# A name or path. `\w` is Unicode-aware, as Rust identifiers are.
WORD = re.compile(r"\w+(?:::\w+)*!?")


def literals(source: str) -> list[Literal]:
    """The string literals in Rust `source`, skipping comments and chars."""
    found: list[Literal] = []
    stack: list[str] = []
    name = ""
    i, line, n = 0, 1, len(source)
    while i < n:
        ch = source[i]
        if source.startswith("//", i):
            end = source.find("\n", i)
            i = n if end < 0 else end
            continue
        if source.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if source.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif source.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    line += source[i] == "\n"
                    i += 1
            continue
        raw = re.match(r'b?r(#*)"', source[i:])
        if raw and (i == 0 or not (source[i - 1].isalnum() or source[i - 1] == "_")):
            close = '"' + raw.group(1)
            start = i + raw.end()
            end = source.find(close, start)
            end = n if end < 0 else end
            text = source[start:end]
            found.append(Literal(line, text, list(stack)))
            line += text.count("\n")
            i, name = end + len(close), ""
            continue
        if ch == '"':
            j, start = i + 1, line
            while j < n and source[j] != '"':
                if source[j] == "\\":
                    j += 1
                line += source[j] == "\n"
                j += 1
            found.append(Literal(start, source[i + 1 : j], list(stack)))
            i, name = j + 1, ""
            continue
        if ch == "'":
            char = re.match(r"'(\\.[^']*|[^\\'])'", source[i:])
            i += char.end() if char else 1
            name = ""
            continue
        word = WORD.match(source, i) if ch.isalnum() or ch == "_" else None
        if word:
            name = word.group(0)
            i = word.end()
            continue
        if ch in "([{":
            stack.append("#" if ch == "[" and source[i - 1 : i] == "#" else name)
        elif ch in ")]}":
            if stack:
                stack.pop()
        elif ch == "!" and name and not name.endswith("!"):
            name += "!"
            i += 1
            continue
        line += ch == "\n"
        if not ch.isspace():
            name = ""
        i += 1
    return found


def shown_english(path: Path, source: str) -> list[str]:
    """Literals in a migrated file that read as English and may be shown."""
    lines = source.splitlines()
    errors: list[str] = []
    for literal in literals(source):
        if not ENGLISH.search(PLACEHOLDER.sub("", literal.text.replace("{{", "").replace("}}", ""))):
            continue
        if any(call.split("::")[-1] in NOT_SHOWN or call in NOT_SHOWN for call in literal.around):
            continue
        here = lines[literal.line - 1] if literal.line <= len(lines) else ""
        above = lines[literal.line - 2].strip() if literal.line >= 2 else ""
        if here.rstrip().endswith(DATA) or above == DATA:
            continue
        first = literal.text.splitlines()[0] if literal.text else ""
        errors.append(f"{path}:{literal.line}: \"{first}\" is not a message")
    return errors


def migrated(sources: dict[Path, str]) -> set[Path]:
    """Files a `// i18n: migrated` marker covers: its own, and those in the
    directory of the same name."""
    roots = [
        path
        for path, text in sources.items()
        if any(line.strip() == MIGRATED for line in text.splitlines())
    ]
    covered = set(roots)
    for root in roots:
        directory = root.with_suffix("")
        covered.update(path for path in sources if directory in path.parents)
    return {path for path in covered if not is_test(path)}


def is_test(path: Path) -> bool:
    """Test files, by the repository's layout: `tests.rs`, a `tests/`
    directory, or a `*_tests` module or directory."""
    return any(
        part == "tests" or part == "tests.rs" or part.removesuffix(".rs").endswith("_tests")
        for part in path.parts
    )


def messages(path: Path, text: str) -> Ftl:
    """The IDs of the file's messages, and the messages its values name.
    Not a parser: see the module's first paragraphs."""
    ftl = Ftl(path, path.parent.name, path.stem)
    for number, line in enumerate(text.splitlines(), start=1):
        if line.startswith("#"):
            continue
        message = MESSAGE.match(line)
        if message:
            ftl.messages.setdefault(message.group(1), number)
        ftl.references.update(REFERENCE.findall(line))
    return ftl


def check(files: list[Ftl], sources: dict[Path, str]) -> list[str]:
    errors: list[str] = []
    used = {name for ftl in files for name in ftl.references}
    for path, text in sources.items():
        if is_test(path):
            continue
        # Comments name messages by way of example.
        text = "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("//"))
        used.update(LITERAL.findall(text))
    for path in sorted(migrated(sources)):
        errors.extend(shown_english(path, sources[path]))
    for ftl in files:
        if ftl.language != FALLBACK:
            continue
        for name, line in ftl.messages.items():
            if name not in used:
                errors.append(f"{ftl.path}:{line}: {name} is not used")
    return errors


def platform(files: list[Ftl], plist: dict, translations: list[Path], desktop: str) -> list[str]:
    """Where one of the app's languages does not name itself, or the macOS
    bundle, the Windows installer, or the Linux desktop entry names other
    languages than the app has."""
    errors: list[str] = []
    languages = {ftl.language for ftl in files}
    named = {ftl.language for ftl in files if ENDONYM in ftl.messages}
    for language in sorted(languages - named):
        errors.append(f"{language} does not name itself: add {ENDONYM} to its language.ftl")
    if plist.get("CFBundleDevelopmentRegion") != FALLBACK:
        errors.append(f"{INFO_PLIST}: CFBundleDevelopmentRegion must be {FALLBACK}")
    listed = plist.get("CFBundleLocalizations", [])
    if sorted(listed) != sorted(languages):
        errors.append(
            f"{INFO_PLIST}: CFBundleLocalizations lists {sorted(listed)}, the app has {sorted(languages)}"
        )
    for path in translations:
        if path.stem not in languages:
            errors.append(f"{path}: {path.stem} is not one of the app's languages")
    installer = {path.stem for path in translations if path.suffix == ".isl"}
    for language in sorted(languages - installer):
        errors.append(f"{INSTALLER}/{language}.isl is missing: the installer has no {language} text")
    # The plain Comment is the English one.
    commented = {FALLBACK} if re.search(r"^Comment=", desktop, re.MULTILINE) else set()
    for tag in LOCALIZED_KEY.findall(desktop):
        language = tag.replace("_", "-")
        if language not in languages:
            errors.append(f"{DESKTOP_ENTRY}: [{tag}] is not one of the app's languages")
        commented.add(language)
    for language in sorted(languages - commented):
        tag = language.replace("-", "_")
        errors.append(f"{DESKTOP_ENTRY}: no Comment[{tag}] for {language}")
    return errors


def tracked(*patterns: str) -> list[Path]:
    output = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "--", *patterns],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return [Path(line) for line in output.splitlines() if Path(line).is_file()]


def main() -> int:
    errors: list[str] = []
    files: list[Ftl] = []
    for path in tracked("crates/*/locales/*/*.ftl"):
        files.append(messages(path, path.read_text(encoding="utf-8")))
    sources = {path: path.read_text(encoding="utf-8") for path in tracked("crates/*.rs")}
    errors.extend(check(files, sources))
    translations = sorted(
        {path.parent for path in tracked("assets/macos/*.lproj/*")}
        | set(tracked(f"{INSTALLER}/*.isl"))
    )
    errors.extend(
        platform(
            [ftl for ftl in files if ftl.path.parts[:2] == ("crates", "sheprd-gpui")],
            plistlib.loads(INFO_PLIST.read_bytes()),
            translations,
            DESKTOP_ENTRY.read_text(encoding="utf-8"),
        )
    )
    for error in errors:
        print(f"FAIL: {error}")
    if errors:
        return 1
    count = sum(len(ftl.messages) for ftl in files)
    print(f"{count} messages in {len(files)} files are valid and used.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
