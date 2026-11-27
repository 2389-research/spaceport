#!/usr/bin/env python3
"""ABOUTME: Snapshot and compare the filesystem surfaces Spaceport shares with the apps it inherits.
ABOUTME: Prove one real install keeps Spaceport's own state and leaves inherited state untouched.

Standard library only, so the same script runs on macOS, Windows and Linux. It
installs nothing and edits nothing: `snapshot` only reads, and `compare` only
reads its two arguments. Files are recorded by size and SHA-256; their contents
are never printed, so a snapshot is safe to hand to a reviewer.

    python3 scripts/release/install-isolation.py surfaces
    python3 scripts/release/install-isolation.py snapshot > before.json
    # install, launch once, quit, uninstall
    python3 scripts/release/install-isolation.py snapshot > after.json
    python3 scripts/release/install-isolation.py compare before.json after.json

A surface is `own` when Spaceport alone writes it, so a change there is the
install working; `inherited` when the inherited Sheprd or Herdr installation owns
it, so a change there is a failure; and `shared` when the two really do share the
path today, which is reported so a reviewer sees it rather than fails.

Exit codes: 0 nothing inherited changed, 1 something inherited changed, 2 a
usage or unreadable-snapshot error.
"""
import argparse
import fnmatch
import hashlib
import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path

FORMAT = 1
# Bigger files are recorded by size alone; a hash of one proves nothing a
# second install run would keep stable, and reading it costs more than it is worth.
DIGEST_LIMIT = 1 << 20

# While the selected daemon runs it owns these under its configuration root.
DAEMON_CONFIG = (
    "*.log",
    "*.sock",
    "*.sock.agent",
    "*.lock",
    "session.json",
    "session-snapshots/**",
)
# The daemon's own live state, and the chat sockets this app opens there. The
# saved-device catalog below `client/` has a surface of its own.
DAEMON_STATE = ("agent-detection/**", "acp/**", "client/**")


class Usage(Exception):
    """A refused invocation; reported as exit 2, never as a verification result."""


@dataclass
class Surface:
    """One path a released app could read, write or collide with."""

    name: str
    owner: str
    path: Path
    note: str = ""
    # Relative paths a running daemon rewrites by itself, so a change is not
    # the install's. Everything else on an inherited surface is watched.
    ignore: tuple[str, ...] = ()
    # Basenames to keep in a listing that holds unrelated entries, such as the
    # user's LaunchAgents directory. None keeps every entry.
    match: tuple[str, ...] = ()
    # A tree is walked; a bundle is recorded by its named members alone.
    walk: bool = True
    files: tuple[str, ...] = ()


def variable(*names: str) -> Path | None:
    for name in names:
        value = os.environ.get(name)
        if value:
            return Path(value)
    return None


def home() -> Path:
    found = variable("HOME", "USERPROFILE")
    if found is None:
        raise Usage("HOME and USERPROFILE are both unset")
    return found


def config_root() -> Path:
    found = variable("XDG_CONFIG_HOME")
    if found is not None:
        return found
    if sys.platform == "win32":
        return variable("APPDATA") or home() / "AppData" / "Roaming"
    return home() / ".config"


def state_root() -> Path:
    found = variable("XDG_STATE_HOME")
    if found is not None:
        return found
    if sys.platform == "win32":
        return variable("LOCALAPPDATA") or home() / "AppData" / "Local"
    return home() / ".local" / "state"


def surfaces() -> list[Surface]:
    """Every surface, in the order the record reads them."""
    config, state, user = config_root(), state_root(), home()
    rows = [
        Surface(
            "own-state",
            "own",
            state / "spaceport",
            "preferences, logs, crash reports, the control socket and saved window state",
        ),
        Surface(
            "own-config",
            "own",
            config / "spaceport",
            "the managed config.toml, config.local.toml, the settings lock and view.toml",
        ),
        Surface(
            "herdr-config",
            "inherited",
            config / "herdr",
            "the selected daemon's configuration root and sockets",
            DAEMON_CONFIG,
        ),
        Surface(
            "herdr-dev-config",
            "inherited",
            config / "herdr-dev",
            "the development daemon's configuration root and sockets",
            DAEMON_CONFIG,
        ),
        Surface(
            "herdr-state",
            "inherited",
            state / "herdr",
            "the daemon's state and session directories",
            DAEMON_STATE,
        ),
        Surface(
            "shared-catalog",
            "shared",
            state / "herdr" / "client",
            "the saved-device catalog this app and an inherited install both read and write",
        ),
        Surface(
            "herdr-dev-state",
            "inherited",
            state / "herdr-dev",
            "the development daemon's state",
            DAEMON_STATE,
        ),
        Surface(
            "shared-dev-catalog",
            "shared",
            state / "herdr-dev" / "client",
            "the development build's saved-device catalog",
        ),
        Surface(
            "herdr-core-state",
            "inherited",
            state / "herdr-core",
            "a separately installed Core's sessions",
        ),
        Surface(
            "herdr-core-config",
            "inherited",
            config / "herdr-core",
            "a separately installed Core's config.toml",
        ),
        Surface(
            "legacy-app-state",
            "inherited",
            state / "herdr" / "gpui",
            "the app state root before Spaceport's own suffix; absent is the expected state",
        ),
        Surface(
            "claude-skill",
            "own",
            user / ".claude" / "skills" / "spaceport",
            "the managed skill, written only when the user accepts the offer",
        ),
        Surface(
            "agents-skill",
            "own",
            user / ".agents" / "skills" / "spaceport",
            "the managed skill, written only when the user accepts the offer",
        ),
    ]
    if sys.platform == "darwin":
        rows += [
            Surface(
                "own-preview-app",
                "own",
                Path("/Applications/Spaceport.app"),
                "the signed preview bundle",
                walk=False,
                files=("Contents/Info.plist",),
            ),
            Surface(
                "own-preview-prefs",
                "own",
                user / "Library" / "Preferences" / "ai.2389.spaceport.plist",
                "the preview's preference domain",
            ),
            Surface(
                "own-preview-caches",
                "own",
                user / "Library" / "Caches" / "ai.2389.spaceport",
                "what the updater downloads and unpacks",
            ),
            Surface(
                "own-dev-prefs",
                "own",
                user / "Library" / "Preferences" / "spaceport.development.plist",
                "the development bundle's preference domain",
            ),
            Surface(
                "inherited-app",
                "inherited",
                Path("/Applications/Sheprd.app"),
                "an inherited app bundle installed beside Spaceport",
                walk=False,
                files=("Contents/Info.plist",),
            ),
            Surface(
                "inherited-prefs",
                "inherited",
                user / "Library" / "Preferences" / "so.pen.herdr-gpui.plist",
                "the inherited app's preference domain",
            ),
            Surface(
                "launch-agents",
                "inherited",
                user / "Library" / "LaunchAgents",
                "login items either app would register",
                match=("*herdr*", "*sheprd*", "*spaceport*"),
            ),
        ]
    elif sys.platform == "win32":
        local = variable("LOCALAPPDATA") or home() / "AppData" / "Local"
        rows += [
            Surface(
                "windows-install-dir",
                "inherited",
                local / "Programs" / "Herdr UI",
                "the inherited installer's directory, which Spaceport's package still installs into; it is expected to fail until C4",
            ),
            Surface(
                "start-menu",
                "inherited",
                variable("APPDATA") or home() / "AppData" / "Roaming"
                / "Microsoft" / "Windows" / "Start Menu" / "Programs",
                "the Start menu shortcuts either app creates",
                match=("*Spaceport*", "*Sheprd*", "*Herdr*"),
            ),
        ]
    else:
        rows += [
            Surface(
                "own-desktop-entry",
                "own",
                user / ".local" / "share" / "applications" / "spaceport.development.desktop",
                "the development window's desktop entry, when a package or a person puts it here; "
                "the Nix flake's copy stays in the store",
            ),
            Surface(
                "own-system-desktop-entry",
                "own",
                Path("/usr/share/applications/spaceport.development.desktop"),
            ),
            Surface(
                "inherited-desktop-entry",
                "inherited",
                user / ".local" / "share" / "applications" / "org.sheprd.Sheprd.desktop",
                "an inherited package's desktop entry",
            ),
            Surface(
                "inherited-system-desktop-entry",
                "inherited",
                Path("/usr/share/applications/org.sheprd.Sheprd.desktop"),
            ),
        ]
    return rows


def ignored(relative: str, patterns: tuple[str, ...]) -> bool:
    """Whether a relative path falls under one of `patterns`."""
    name = relative.rsplit("/", 1)[-1]
    for pattern in patterns:
        if pattern.endswith("/**"):
            prefix = pattern[:-3]
            if relative == prefix or relative.startswith(prefix + "/"):
                return True
        elif "/" in pattern:
            if fnmatch.fnmatchcase(relative, pattern):
                return True
        elif fnmatch.fnmatchcase(name, pattern):
            return True
    return False


def selected(surface: Surface, name: str) -> bool:
    return not surface.match or any(fnmatch.fnmatchcase(name, pattern) for pattern in surface.match)


def describe(path: Path) -> dict[str, object]:
    if path.is_symlink():
        return {"kind": "link", "target": os.readlink(path)}
    if path.is_dir():
        return {"kind": "dir"}
    if path.is_file():
        size = path.stat().st_size
        entry: dict[str, object] = {"kind": "file", "size": size}
        if size <= DIGEST_LIMIT:
            entry["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        return entry
    return {"kind": "other"}


def collect(surface: Surface) -> tuple[dict[str, dict[str, object]], int]:
    """The entries this surface holds now, and how many the rules left out."""
    entries: dict[str, dict[str, object]] = {}
    root = surface.path
    if not (root.exists() or root.is_symlink()):
        return entries, 0
    if not (root.is_dir() and not root.is_symlink()):
        entries["."] = describe(root)
        return entries, 0
    entries["."] = describe(root)
    if not surface.walk:
        for member in surface.files:
            path = root / member
            if path.exists() or path.is_symlink():
                entries[member] = describe(path)
        return entries, 0
    skipped = 0
    for directory, children, names in os.walk(root, followlinks=False):
        base = Path(directory)
        kept = []
        for name in children:
            relative = (base / name).relative_to(root).as_posix()
            if ignored(relative, surface.ignore):
                skipped += 1
            else:
                kept.append(name)
        children[:] = kept
        for name in names:
            relative = (base / name).relative_to(root).as_posix()
            if ignored(relative, surface.ignore):
                skipped += 1
                continue
            if surface.match and not selected(surface, name):
                skipped += 1
                continue
            entries[relative] = describe(base / name)
    return entries, skipped


def snapshot() -> dict[str, object]:
    recorded = {}
    for surface in surfaces():
        if surface.match and not (surface.path.is_dir() and surface.path.exists()):
            entries, skipped = {}, 0
        else:
            entries, skipped = collect(surface)
        recorded[surface.name] = {
            "owner": surface.owner,
            "path": str(surface.path),
            "note": surface.note,
            "present": bool(entries),
            "ignored": skipped,
            "entries": entries,
        }
    return {
        "format": FORMAT,
        "platform": sys.platform,
        "surfaces": recorded,
    }


def differences(before: dict[str, object], after: dict[str, object]) -> list[str]:
    lines = []
    for relative in sorted(set(before) | set(after)):
        if relative not in after:
            lines.append(f"removed {relative}")
        elif relative not in before:
            lines.append(f"added {relative}")
        elif before[relative] != after[relative]:
            lines.append(f"changed {relative}")
    return lines


def load(path: Path) -> dict[str, object]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise Usage(f"cannot read the snapshot {path}: {error}") from error
    if not isinstance(document, dict) or document.get("format") != FORMAT:
        raise Usage(f"{path} is not a version {FORMAT} install-isolation snapshot")
    if not isinstance(document.get("surfaces"), dict):
        raise Usage(f"{path} has no surfaces")
    return document


def report(out, before: dict[str, object], after: dict[str, object]) -> int:
    failed = 0
    shared = 0
    first, second = before["surfaces"], after["surfaces"]
    for name in sorted(set(first) | set(second)):
        left = first.get(name, {})
        right = second.get(name, {})
        surface = right or left
        owner = surface.get("owner", "inherited")
        changes = differences(left.get("entries", {}), right.get("entries", {}))
        ignored_paths, ignored_then = right.get("ignored", 0), left.get("ignored", 0)
        ignored_note = f" ({ignored_then} then {ignored_paths} entries left out)" if (
            ignored_paths or ignored_then
        ) else ""
        if not changes:
            print(f"{name:<30} unchanged{ignored_note}", file=out)
            continue
        expected = owner != "inherited"
        label = "changed" if owner == "own" else ("shared" if expected else "CHANGED")
        print(f"{name:<30} {label} ({owner}): {len(changes)} entries{ignored_note}", file=out)
        for line in changes[:10]:
            print(f"{'':<30}   {line}", file=out)
        if len(changes) > 10:
            print(f"{'':<30}   and {len(changes) - 10} more", file=out)
        if not expected:
            failed += 1
            print(f"{'':<30}   {surface.get('path', '')}: {surface.get('note', '')}", file=out)
        elif owner == "shared":
            shared += 1
            print(f"{'':<30}   {surface.get('path', '')}: {surface.get('note', '')}", file=out)
    if failed:
        print(
            f"\n{failed} surface(s) Spaceport does not own changed. That is the failure this "
            "check exists to find; see docs/spaceport/evidence/c5-install-isolation.md.",
            file=out,
        )
        return 1
    if shared:
        print(
            f"\nNo inherited surface changed. {shared} shared surface(s) did, which the "
            "record names under its findings.",
            file=out,
        )
        return 0
    print("\nNo inherited surface changed.", file=out)
    return 0


def listing(out) -> None:
    for surface in surfaces():
        owner = surface.owner
        note = f"  {surface.note}" if surface.note else ""
        print(f"{surface.name:<30} {owner:<10} {surface.path}{note}", file=out)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[1])
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("surfaces", help="Print the surfaces this check records")
    commands.add_parser("snapshot", help="Print this user's snapshot as JSON")
    compare = commands.add_parser("compare", help="Compare two snapshots")
    compare.add_argument("before", type=Path)
    compare.add_argument("after", type=Path)
    arguments = parser.parse_args(argv)
    try:
        if arguments.command == "surfaces":
            listing(sys.stdout)
            return 0
        if arguments.command == "snapshot":
            json.dump(snapshot(), sys.stdout, indent=1, sort_keys=True)
            sys.stdout.write("\n")
            return 0
        return report(sys.stdout, load(arguments.before), load(arguments.after))
    except Usage as error:
        print(f"install-isolation: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
