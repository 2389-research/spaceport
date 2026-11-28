# C5 install and upstream-state isolation (007r)

Ticket 007r verifies three things for a fresh install of each Spaceport release
channel: the install lands a working app, it does not read, write, migrate or
collide with the inherited Sheprd/Herdr-1 state that D60 and D61 imply is
separate, and removing it is clean. This record is the verification procedure
and the path-level evidence that needs no install. **No install was performed
and no runtime was installed, started, stopped or removed.** A real install run
on each platform remains an explicit gate, listed at the end.

Isolation here has one nuance worth stating before the tables. Under D58
Spaceport is a client of a separately installed Herdr 1 or Herdr Core, so it is
*supposed* to dial that runtime's sockets and to see that runtime's sessions.
Those are shared by design. What must not be shared is the inherited
*application's* own state: its preferences, logs, control channel, caches,
keychain entries, login items and OS identity. A42 is the acceptance scenario
this record serves, and it stays `not_run`: nothing below is a side-by-side
install.

## Method

1. Trace, in source, every path the app reads or writes at startup and on its
   own timers, with the function that resolves it.
2. Record what exists now on the one machine available here (macOS, a signed
   preview installed, a Herdr 1 daemon running), read-only.
3. Record the identity each channel carries: build-time app ID, bundle or
   installer identity, and install location.
4. Turn 1–3 into a repeatable before/after check:
   `scripts/release/install-isolation.py` snapshots the owned and inherited
   surfaces before the install, after first launch and after uninstall.
5. Write down what only a real install can settle.

## What the app reads and writes

Every path below was read out of the current source. `own` is Spaceport's own
file, `shared` is a file the selected runtime also owns, and `inherited` is an
inherited application's state this app must not touch.

| Surface | Path (release build; `herdr-dev` when the build is a development one) | Access | Resolved by | Verdict |
| --- | --- | --- | --- | --- |
| Preferences, window state, saved device tags and notes, mutes, palette history, browser tabs, editor groups, control socket, logs, crash reports | `<state>/spaceport/` | read, write | `spaceport-client/src/local_path.rs::state_dir` | own |
| Managed settings, local overrides, settings lock, view settings | `<config>/spaceport/{config.toml,config.local.toml,config.lock,view.toml}` | read, write | `spaceport-settings/src/config.rs::app_config_path`, `spaceport-client/src/settings_files.rs` | own |
| Agent skill, only after the user accepts the offer, only marker-owned files | `~/.claude/skills/spaceport/SKILL.md`, `~/.agents/skills/spaceport/SKILL.md` | write with consent | `spaceport-gpui/src/agent_skill.rs` | own |
| Daemon client socket | `<config>/herdr[-dev]/herdr-client.sock` and `sessions/<name>/herdr-client.sock` | connect | `spaceport-client/src/discovery.rs::session_socket` | shared by design |
| Daemon session names | `<config>/herdr[-dev]/sessions/` | read | `spaceport-client/src/sessions.rs` | shared by design |
| Saved-device catalog, selected device, SSH metadata | `<state>/herdr[-dev]/client/{endpoints.json,endpoint-selection.json,ssh-metadata/}` | read at startup, write on a device edit; `ssh-metadata/` is neither read nor written by this build | `spaceport-client/src/catalog.rs::catalog_path` | shared with the inherited tree |
| Chat attach sockets | `<state>/herdr/acp/<id>.sock`, or `\\.\pipe\herdr-acp-<id>` | create, connect | `spaceport-chat-protocol/src/endpoint.rs` | shared directory, random id per chat |
| Herdr Core session and config | `HC_SOCKET`, `<state>/herdr-core/<session>`, `<config>/herdr-core/config.toml` | connect, read | `spaceport-core-client/src/endpoint.rs` | shared by design |
| Windows control pipe | `\\.\pipe\spaceport-control-<sid>[-hash]` | create | `spaceport-control-protocol/src/socket.rs` | own |
| macOS preference domain (the updater's settings) | `~/Library/Preferences/ai.2389.spaceport.plist` | read, write | Sparkle, through the preview bundle's ID | own |
| Windows notification identity | `HKCU\Software\Classes\AppUserModelId\<app ID>` | create on first toast | `spaceport-gpui/src/window/system_notifications/toast.rs` | own |
| Windows Launch at login | `HKCU\...\CurrentVersion\Run\Spaceport` and `...\Explorer\StartupApproved\Run\Spaceport` | write when enabled | `spaceport-gpui/src/launch_at_login.rs` | own |
| Linux desktop entry | `$out/share/applications/spaceport.development.desktop` in the Nix store | write at package time | `flake.nix` | own |

The app stores nothing of its own in `NSUserDefaults`: the preview's preference
domain belongs to the updater, which the app only asks to clear a stored feed
override before startup. It registers no LaunchAgent or login item, reads no
keychain item, and writes nothing under `~/Library/Application Support`.
Checked directly:

```
$ grep -rln "LaunchAgents\|SecItem\|SecKeychain\|Application Support" --include=*.rs crates/*/src
(none)
$ grep -rn "so\.pen\.herdr-gpui" --include=*.rs crates/*/src
crates/spaceport-gpui/src/constants/tests.rs:57:    assert_ne!(APP_ID, "so.pen.herdr-gpui");
$ grep -rn 'herdr/gpui' --include=*.rs crates/*/src
crates/spaceport-gpui/src/preferences/tests.rs:274:    let root = Path::new("/state/herdr/gpui");
```

The one hit for the inherited identity is a regression test that requires it to
differ, and the `herdr/gpui` hits are test fixtures for the state root this fork
left behind. No production path names either.

What is left outside `<state>/spaceport` and `<config>/spaceport` is the
selected runtime's own state — its sockets, its session names, its Core session
— which D58 makes the point of the app, plus the shared saved-device catalog.

## Channel identities

| Channel | App ID the OS sees | Installs to | Verified here |
| --- | --- | --- | --- |
| macOS signed preview | `ai.2389.spaceport`, Team `HD9NM9NSMK` (`package-macos.sh` inserts `crates/spaceport-gpui/macos-preview-app-id`) | `/Applications/Spaceport.app` by drag from the DMG (`sign-macos.sh` links `/Applications`) | installed bundle on this machine verifies; see below |
| macOS development build | `spaceport.development` (`just bundle`) | `target/{release,debug}/Spaceport.app`, never installed | source and recipe only |
| Spaceport Windows installer | AppUserModelID `spaceport.development`, AppId `{62B8722B-1FA2-48E0-9609-46AA5D431983}`, publisher `2389 Research, Inc.` | `%LOCALAPPDATA%\Programs\Spaceport` | C4 installer smoke on a clean runner |
| Linux Nix flake | development ID; store path, immutable | the Nix profile | recipe and flake only |
| `site/install.sh` | — | — | installs the upstream product (see below) |

`build.rs` selects between the two IDs in one place
(`crates/spaceport-gpui/preview_identity.rs::select`): no version means
development, a `YYYYMMDD.N` version on macOS means the preview, and anything
else refuses the build. A `HERDR_RELEASE_VERSION` build still fails to compile
with `Spaceport release identity is not configured`.

The installed preview on this machine, read-only:

```
$ plutil -p /Applications/Spaceport.app/Contents/Info.plist | grep -E "CFBundleIdentifier|ShortVersion"
  "CFBundleIdentifier" => "ai.2389.spaceport"
  "CFBundleShortVersionString" => "20261126.1"
$ codesign -dv /Applications/Spaceport.app
Identifier=ai.2389.spaceport  TeamIdentifier=HD9NM9NSMK
$ codesign --verify --strict /Applications/Spaceport.app
valid on disk; satisfies its Designated Requirement
$ spctl --assess --type execute -vv /Applications/Spaceport.app
accepted  source=Notarized Developer ID  2389 Research, Inc (HD9NM9NSMK)
```

Nothing in that bundle claims `so.pen.herdr-gpui`, so LaunchServices, the
preference store and Gatekeeper all key on a distinct identity. It carries
`CFBundleIconFile=Sheprd.icns`; that is a cosmetic file name, not a
registration, and is recorded so it is not re-filed as an isolation failure.

## Executed checks

`scripts/release/install-isolation.py` is the repeatable half of this record. It
uses standard-library Python and installs nothing. Its snapshots contain paths,
link targets, sizes and digests. Check filenames for private details before
sharing a snapshot. The following census and test counts describe the original
snapshot format. Capture fresh snapshots with the same checker for each run.

| Check | Command | Result |
| --- | --- | --- |
| Surface list | `python3 scripts/release/install-isolation.py surfaces` | 20 surfaces on macOS: 8 own, 10 inherited, 2 shared |
| Machine census | `python3 scripts/release/install-isolation.py snapshot` | `own-state` 7 entries, `own-config` 6, `herdr-config` 3 (8 left out), `herdr-state` 2 (2 left out), `shared-catalog` 4, `own-preview-app` 2, `own-preview-prefs` 1, `own-preview-caches` 108, `launch-agents` present with 14 unrelated entries left out, and every inherited surface this fork replaced absent |
| Live comparison | `… snapshot > before.json`; app running throughout; 92 s later `… snapshot > after.json`; `… compare before.json after.json` | exit 0. The only change was `own-state/logs/spaceport.jsonl`, the running preview's own log. Every inherited surface — the daemon's config, the daemon's state, `so.pen.herdr-gpui` preferences, `/Applications/Sheprd.app`, the login items — was unchanged, and so was the shared device catalog over that window. |
| Ghost root | snapshot of `legacy-app-state` (`<state>/herdr/gpui`, this fork's pre-isolation state root) | absent, and watched, so a build that reintroduced it would fail the comparison |
| Unrelated login items | snapshot of `launch-agents` | 14 unrelated agent plists left out and none kept, so a third-party updater's comings and goings never count; a Herdr or Spaceport one would be kept |
| Suite | `python3 -m unittest scripts/release/tests/test_install_isolation.py` | 15 tests pass |

`snapshot` records presence as well as content, so the census is also the
baseline for a later install run: on this machine the inherited application is
absent entirely (no `Sheprd.app`, no `so.pen.herdr-gpui` preference domain, no
`herdr/gpui` state root), which is why a side-by-side run needs a machine that
does have one.

The state this machine does have, for orientation:

| Present | Note |
| --- | --- |
| `~/.local/state/spaceport/` — `control.sock`, `editor-groups.json`, `local-<hash>.json`, `window-state.json`, `logs/spaceport.jsonl` plus one rotation | the app's own state, and nothing else |
| `~/.config/spaceport/` — `config.toml`, `config.local.toml`, `config.lock`, `view.toml`, `.view.toml.gpui-lock` | the app's own settings; `view.toml` is 36 bytes holding one toast-delivery setting |
| `~/.config/herdr/` — `config.toml` and one `config.toml.bak-*`, sockets, logs, `session.json`, `session-snapshots/` | the daemon's, untouched by the app |
| `~/.local/state/herdr/client/` — `endpoints.json`, `endpoint-selection.json`, `ssh-metadata/` | shared device selection |
| `~/Library/Preferences/ai.2389.spaceport.plist`, `~/Library/Caches/ai.2389.spaceport` | the preview's own updater state |
| the preview process itself, `20261126.1`, running from `/Applications/Spaceport.app` | an install that was already there; this record did not create it |

The census also found `~/Library/Caches/ai.2389.spaceport.fixture.*` (50
directories) left by the recorded Sparkle native fixtures. They are unrelated to
an install, are not matched by any surface, and deleting them is the fixture
owner's call, not this ticket's.

## The verification procedure

Run each channel on a machine that already has the inherited product on it — an
installed `Sheprd.app`, a `so.pen.herdr-gpui` preference domain, and a Herdr 1
daemon with saved sessions. Stop agents first: a live daemon rewrites its own
logs and `session.json`, which the snapshot rules leave out, but nothing else
should be moving during the run.

Use the same absolute package path throughout. For macOS, use the app bundle.
For Windows, use the Spaceport install directory. For Linux, use the active Nix
profile, which can contain other packages. Keep all three snapshot files outside
the checked paths. For example, on a disposable macOS preview test machine:

```sh
package=/Applications/Spaceport.app
python3 scripts/release/install-isolation.py snapshot --package "$package" > before.json
# Install, start the app, open a workspace, then quit.
python3 scripts/release/install-isolation.py snapshot --package "$package" > installed.json
# Uninstall the selected package.
python3 scripts/release/install-isolation.py snapshot --package "$package" > removed.json
python3 scripts/release/install-isolation.py verify before.json installed.json removed.json
```

The package must start absent. All expected files must appear after installation
and disappear after removal. The macOS bundle or Windows install directory must
also disappear. The Nix profile may retain unrelated packages. The checker
compares inherited state across both intervals. `compare` alone checks changes
to inherited surfaces and cannot qualify an install.

Expected outcomes for each channel:

| Channel | Install | First launch | Uninstall | Failure looks like |
| --- | --- | --- | --- | --- |
| macOS signed preview | verify the DMG, drag `Spaceport.app` to `/Applications`; `plutil -p` shows `ai.2389.spaceport`; the previous `Sheprd.app` still present and still verifies | one window, one workspace list from the running daemon; `~/Library/Preferences/ai.2389.spaceport.plist` appears or updates; no `so.pen.herdr-gpui` change | quit, move the bundle to the trash; LaunchServices forgets it | `inherited-app` or `inherited-prefs` changed; `own-state` did not appear |
| macOS development | `just bundle release`, open `target/release/Spaceport.app` | same, with sockets from `herdr-dev` if that daemon is running | delete the bundle | a development build touching the release daemon's catalog |
| Windows installer | install beside inherited Sheprd | one window; `Run\Spaceport` only when Launch at login is turned on | run the uninstaller and inspect the registry | the inherited install's files or Start menu shortcut gone; a leftover `Run\Spaceport` |
| Linux (Nix) | `nix profile install .#spaceport`, in a test profile | one window, development identity | `nix profile remove spaceport` | any write under `$XDG_CONFIG_HOME/herdr`'s sockets or sessions |

Registry checks for the Windows run — the snapshot tool covers the filesystem,
and these are read by hand before and after each step:

```powershell
reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "Spaceport"
reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "Herdr UI"
reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"
reg query "HKCU\Software\Classes\AppUserModelId\so.pen.herdr-gpui"
reg query "HKCU\Software\Classes\AppUserModelId\spaceport.development"
reg query "HKCU\Software\Classes\spaceport"
reg query "HKCU\Software\Classes\sheprd"
reg query "HKCU\Software\Classes\herdr-gpui"
```

Expected: `Run\Spaceport` exists only while Launch at login is on and is gone
after uninstall; `Run\Herdr UI` is whatever the inherited install left and is
never edited; the three `Classes` schemes are untouched (`windows-installer.py`
already refuses to register or delete any of them); the installed app's
AppUserModelId key appears on first toast and is removed by uninstall.

## Findings

### Windows package and startup identity

C4 gave Spaceport its own install directory, shortcut, AppId and startup
value. The checker treats the Spaceport Start menu shortcut as owned. It
monitors the inherited Sheprd shortcuts and install directory for changes.
The registry commands above remain manual checks. The Windows smoke test
covers a clean installation and removal. The side-by-side run remains open.

### The saved-device catalog lives in the inherited state tree

The catalog and the selected device resolve to
`<state>/herdr/client/endpoints.json` and `endpoint-selection.json`
(`spaceport-client/src/catalog.rs`, which records that it follows upstream's
paths), not to `<state>/spaceport/`. A release build and an inherited release
build therefore read and write the same file: an inherited install's saved hosts
appear in Spaceport's device list, and a device added or selected in Spaceport is
written where that install reads it. On this machine the file holds one SSH
device. It is not a migration and it is not a copy — one file, two readers.
Either the app separates it, or the sharing is accepted as the same choice D58
makes for the sockets; that decision is not this ticket's.

### The inherited install scripts

`site/install.sh` downloads from `sheprd-labs/sheprd` and installs the upstream product; its
header says so. Reading the landing page as a Spaceport channel would install
the inherited app. `site/install.ps1` likewise installs the upstream Windows product.
No Spaceport Linux distribution package exists yet: the Nix flake is the
only Linux route, and it installs from the store without writing user
configuration.

## Verification not run

- No install, uninstall, reinstall or first-run launch of any channel. The one
  install on this machine predates this record and was left alone.
- No Windows or Linux machine, and no Windows registry was read or written. The
  `windows-package` CI job does install, reinstall and uninstall the Setup file
  on a clean runner, which is the only recorded native run of that package; it
  proves the installer's own ownership rules on an empty machine, not
  side-by-side behaviour.
- No Herdr 1 or Core runtime was installed, started, stopped or removed, and no
  existing Herdr or Sheprd state was written, moved or deleted.
- The app was running throughout the census, as the owner's own installed
  preview. Its log line is the only change the two snapshots recorded.

## Remaining gates

Each of these needs a real install on a machine with inherited state present:

1. macOS preview beside an inherited `Sheprd.app`: drag-install, first launch,
   quit, trash, with the three snapshots. Needs an inherited build installed;
   none is available here, and installing one is outside this ticket.
2. macOS development bundle beside the same install, to prove
   `spaceport.development` and `herdr-dev` stay separate.
3. Windows: install, first launch, Launch at login on then off, reinstall, and
   uninstall on a machine with an inherited installation. Record its registry
   entries before and after each step.
4. Linux: `nix profile install` then `nix profile remove` with a daemon running,
   checking the daemon's config, state and sockets are untouched.
5. Keychain and login-item negatives on a machine that has an inherited install:
   confirm no new keychain item and no new LaunchAgent appears. Both are absent
   from the source, but absence in source is not absence after a launch.
6. A Sparkle update of the installed preview, which changes the app in place:
   re-run the three snapshots around it to prove an update is as isolated as an
   install. The update path itself is T12's, and its native fixtures are
   recorded in `t12-sparkle-preview.md`.

A42 remains `not_run`. Each platform needs a side-by-side install and the
saved-device catalog still needs an ownership decision.
