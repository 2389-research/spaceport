# Removal scenarios A29 to A32 on the 20261117.1 preview

A partial run of the removal scenarios on 2026-11-17. It covers the legs
that need no running app. A29, A30, A31 and A32 stay `not_run`, because each
also needs a running-app leg that did not run.

## Subject

- Public image `Spaceport-20261117.1-arm64-apple-darwin.dmg`, SHA-256
  `9d8dcff2ff9942c7430503fd7e77546e99d07f1803f153ddab02fa38d28d963c`,
  built from `723be121f3d6850968865d1735569fb7b0514892`. Its signature and
  notarization checks are in `t12-macos-preview-20261117.1-signed.md`.
- Dependency graph: `Cargo.lock` at `origin/main` `98147d511`, cargo 1.99.0.
- Host: macOS 27.2.1, arm64. The image was mounted read-only at a private
  mount point and detached afterwards.

## Retired commands (A29, A30, A31)

Each command ran against the mounted `Contents/MacOS/spaceport` with a
cleared environment and a fresh private `HOME` and XDG directories. A
throwaway script started the exact child, waited at most ten seconds and
read its exit code.

All 20 retired forms exited 2 with empty stdout and an `Unknown option` or
`Unknown chat command` message:

`automation`, `browser list`, `chat convert`, `checkpoint create before`,
`cloud add`, `code edit src/main.rs:42`, `mod`, `orchestrator`,
`remote status`, `open .`, `review open`, `search file src/main.rs`,
`teleport --to other-host`, `pr checks`, `system`, `px0 open`,
`scripts list`, `--mockup`, `--herdr-apply-update`, `--session work`.

`--help` exited 0 and lists none of these families.

Mistake in the probe: it also ran `--socket /private/test.sock`. That option
is retained (D58, attach-only), so the app started. The script killed that
exact child after ten seconds. The app wrote only to the private `HOME`
(`config/spaceport/` and `state/spaceport/`). No personal state was read or
written, and no process was left running. This is not evidence of any
scenario.

## Shipped bundle inventory (A31, A32)

- 100 files. Mach-O executables: `spaceport`, `sheprd-acp`, and Sparkle's
  `Autoupdate`, `Sparkle`, `Updater`, `Downloader` and `Installer`.
- No LaunchAgent, LaunchDaemon, login item or app extension. The only XPC
  services are Sparkle's `Downloader.xpc` and `Installer.xpc` (D65).
- `Info.plist`: identifier `ai.2389.spaceport`, no URL scheme, no background
  mode, `NSBonjourServices` is `_ssh._tcp` only.
- `spaceport` links system frameworks and Sparkle only.
- No Herdr or Herdr Core binary is in the bundle.

## Strings in the shipped executable (A29, A30, A31)

`strings -a` on `spaceport` gave 143,123 lines. Zero lines match, ignoring
case: `daytona`, `railway.app`, `sprites.dev`, `opensandbox`, `coder.com`,
`api.github.com`, `github.com/login`, `octocrab`, `teleport`, `apns`,
`push.apple`, `mdns`, `pairing`, `uniffi`, `checkpoint`, `libgit2`, `luau`.
Hits for `boxd`, `gix` and `orchestrat` are unrelated text (math symbol
names, binary noise, and serde messages).

A string scan shows absence of these names only. It does not prove absence
of behavior.

## Dependency graph (A32)

`cargo tree --locked --offline --workspace --all-features --target all
-e normal,build` resolves 704 packages. None of these is present: `gix*`,
`git2`, `octocrab`, `uniffi`, `tokio`, `axum`, `hyper`, `reqwest`, any
`mdns` or `apns` crate, `keyring`, `mlua` or `luau`, `rusqlite`,
`tree-sitter`, any `lsp` crate, `wry`, `tao`, `ssh2`, `russh`.

`ureq` and `rustls` are present. `ureq` has one dependent, `sheprd-gpui`,
and one call site outside tests: `chat/visual/remote.rs`, which fetches
remote images for rich chat.

This list of names was chosen for this run. It is not the exhaustive
removal inventory that `plan/removals.json` asks for.

## Observations to review

- The binary carries usage and status page links for many AI providers (for
  example `platform.openai.com/usage` and `status.openai.com`). They belong
  to the retained passive usage display metadata. No provider API endpoint
  was found. G-OBS remains open.
- An `icons/azure-devops.svg` asset is still embedded.
- `--help` still lists `worktree` and `notes` ("Notes kept on worktrees").
  Whether they are retained was not checked.
- The app's entitlements include camera, microphone, location, calendars,
  contacts, photos and Apple events. Terminal apps commonly hold these for
  the programs they run. Whether the fork wants all of them was not
  reviewed.

## Not run

- A29: removed configuration and UI routes in a running app; plain git,
  diff and source text rendered in a terminal and in chat.
- A30: stale settings loaded by a running app; fake provider endpoints
  monitored; installed services inspected after a run.
- A31: process and network inventory of a running app; a retained CLI
  command answered by the running app over its private socket.
- A32: Windows and Linux release artifacts; the shared renderer and input
  contract suites; a migration and runtime-installation check at run time.
- No test suite was compiled or run for this note.
