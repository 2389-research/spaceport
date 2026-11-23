# Dependency audit 2: `spaceport-gpui` after the rename and the extraction waves

Round two of the "hold fast, you're tethered" (hfyt) dependency work. Round one
(#146) dropped seven direct edges from the app crate. This round re-reads the
graph after the crate rename (#169), the icon-family rename (#187), the
`render_sidebar` split (#188), and the work-family removal (#152/#189), and
removes what those waves left behind.

The chat-core extraction (#191) is **not** covered: there is no merge for it on
`origin/main` at `1f6a8be64` and no `crates/spaceport-chat-core` in the tree.
When it lands, its re-exports are the likeliest place for a fresh stale edge in
the app crate, and this audit's scan is the check to repeat.

## Where round one left its record

There is no `docs/spaceport/evidence/` file for #146. Its record is the commit
message of `9b645515d` ("chore(gpui): Drop the app's unused direct dependencies")
and of its guard commit `9ffaaa777`:

- Seven direct edges dropped — `flate2`, `getrandom`, `sheprd-lenient`,
  `serde_ignored`, `sha2`, `skrifa`, `tracing-subscriber` — with `flate2` and
  `sha2` moved to dev-dependencies rather than deleted.
- 66 direct edges to 59, and the normal dependency graph "keeps the same 463
  resolved packages"; no version added, removed, or upgraded.
- `ureq` retained for its one call site outside tests,
  `chat/visual/remote.rs`: the rich-chat web picture a user explicitly loads.

Those counts were taken on `sheprd-gpui`, before the rename, before the app
crate was split into the finer-grained `spaceport-*` crates, and before any of
the extraction waves above. They cannot be compared with a number measured
today. Every pair below is measured with one command before and after *this*
change, so the delta is attributable to this change alone.

## Method

A dependency edge counts as live only when a compiled path names it: a
`use name…` item, a `name::` path, or `extern crate name`. Comments and string
literals do not count, so a dep named only in prose or in a `"Cargo.toml"`
fixture string is not live. The scan covers all 1 827 `*.rs` files under
`crates/spaceport-gpui` **plus** the files the crate `#[path]`-includes from
outside its directory:

- `crates/test-support/{executable,pe,process,sandbox}.rs` and
  `crates/test-support/process/tests{,/linux}.rs`
- `crates/spaceport-gpui/{build_identity,preview_identity,release_version}.rs`

That last set is not optional: `crates/spaceport-gpui/src/main.rs` includes
`../../test-support/process.rs` under `#[cfg(all(test, unix))]`, and that file is
the *only* thing that names `libc`. A crate-local grep calls `libc` dead and is
wrong. Both the `libc` dev-dependency and the `nix`/`rustix` edges survive this
audit for exactly that reason.

Counting conventions, stated so the numbers can be reproduced:

| Command | What it counts |
| --- | --- |
| `cargo tree -p spaceport-gpui -e normal --depth 1 --target T --prefix none` | direct normal edges Cargo resolves for target `T` |
| `cargo tree -p spaceport-gpui -e normal --target T --prefix none` | resolved packages on the normal graph, `sort -u` of `name vX` |
| the same with `-e normal,build,dev` | resolved packages including build and dev graphs |

Names in `[dependencies]` are counted per manifest key, and Cargo renames keep
the key as the extern name: `config_loader` is the `config` package, `gpui` is
`gpui-pre`, `gpui_platform` is `gpui-pre-platform`, and `font-kit` is
`zed-font-kit`. `cargo tree` prints package names, so those four appear in its
output under the package name, not the manifest key.

## Before and after

Declared names in `crates/spaceport-gpui/Cargo.toml`:

| Section | Before | After |
| --- | --- | --- |
| `[dependencies]` | 53 | 53 |
| `[target.'cfg(unix)'.dependencies]` | 1 | 1 |
| `[target.'cfg(windows)'.dependencies]` | 6 | 6 |
| `[target.'cfg(target_os = "linux")'.dependencies]` | 4 | 3 |
| `[target.'cfg(target_os = "macos")'.dependencies]` | 15 | 15 |
| unique non-dev names (union of the above) | 79 | 78 |
| `[build-dependencies]` | 2 | 2 |
| dev-dependencies (`[dev-dependencies]` + two target sections) | 19 | 19 |

Direct normal edges and resolved packages, measured per target:

| Target | Direct edges before → after | Resolved (normal) | Resolved (normal+build+dev) |
| --- | --- | --- | --- |
| `aarch64-apple-darwin` | 64 → 64 | 482 → 482 | 525 → 525 |
| `aarch64-unknown-linux-gnu` | **58 → 57** | 590 → 590 | 625 → 625 |
| `x86_64-pc-windows-msvc` | 59 → 59 | 464 → 464 | 498 → 498 |

No resolved package was added, removed, or upgraded on any target. Of the 64
macOS-host direct edges, 21 are workspace path crates (`spaceport-*`) and 43 are
external packages; the split is reported because #146's 59 covered a crate that
had not yet been split into those 21.

## Removal: `fontdb` (normal, `cfg(target_os = "linux")`)

The one edge with no call path.

**Proof it has no call path.** `grep -rn "fontdb" crates/spaceport-gpui/`
matches nothing in any file of the crate — not Rust, not the manifest, not
`windows.manifest`, not the FTL catalogs. There is no `#[path]` include that
could reach a `fontdb` use from outside the directory either: the crate's four
external includes are the `test-support` and identity files listed under Method,
and none names `fontdb`.

**Where it went.** `fontdb` was never the app's. `bitmap_fonts::is_undrawable`
reads the system font database, and that file moved to `spaceport-settings` in
`5aed49a04` ("fix(settings): Carry the moved file's Linux fontdb dependency"),
which added the dependency to `crates/spaceport-settings/Cargo.toml` with the
same Linux target gate. The app crate kept its own edge. In the current tree the
only `fontdb` code in the repository is
`crates/spaceport-settings/src/bitmap_fonts.rs`.

**The edit.** One line from
`[target.'cfg(target_os = "linux")'.dependencies]`, and the one lock edge it
implied:

```
- "fontdb 0.24.0",        # in [[package]] name = "spaceport-gpui"
```

`fontdb 0.24.0` stays in `Cargo.lock` under `[[package]] name =
"spaceport-settings"`, and the workspace entry `fontdb = "0.24"` in the root
`Cargo.toml` stays because that member still needs it. `cargo metadata` rewrote
nothing else; `git diff Cargo.lock` is that single deletion.

**Graph check.** On `aarch64-unknown-linux-gnu` the direct edge list loses
`fontdb v0.24.0` and nothing else; the resolved normal and normal+build+dev sets
are byte-identical before and after. On macOS and Windows the direct lists are
byte-identical, which is expected — the edge was Linux-gated.

**What this proves and what it does not.** The Linux direct-edge delta and the
unchanged resolved set are proven by `cargo tree --target
aarch64-unknown-linux-gnu`, which needs no toolchain for that target. The
removal cannot be *compile*-proven on this macOS host: `fontdb` is behind
`cfg(target_os = "linux")`, and a `cfg` gate is only proven on the platform it
names. The compile-level proof is deferred to CI's Linux legs, backed here by
the source search above.

## Retained deliberately

- **`ureq`** — `src/chat/visual.rs:56` (`Fetch(#[source] Box<ureq::Error>)`) and
  `src/chat/visual/remote.rs:19` (the `ureq::Agent` that loads a rich-chat web
  picture), plus `src/i18n/errors/tests/chat.rs:108`. Unchanged from #146's
  reasoning.
- **`libc`** (dev, unix) — live only through the `#[path]` include of
  `crates/test-support/process.rs`; see Method.
- **`font-kit`** (normal, macOS) — this edge is live, but only from test code:
  its single call site is `src/config/tests/font_aliases.rs:122`. It is declared
  in `[target.'cfg(target_os = "macos")'.dependencies]`, so it is compiled into
  the shipped binary's manifest while only `cargo test` uses it. Moving it to a
  macOS dev-dependency would shrink the normal manifest without changing the
  resolved graph; that restructure was outside this task's "remove only deps
  with no call path" instruction and is left as a follow-up.

## Orphaned packages in `Cargo.lock`

None. Every one of the 889 `[[package]]` entries in `Cargo.lock` is reachable
from a workspace member through the lock's own dependency edges: build the graph
from the entries with no `source` line (the 28 workspace members) and walk it —
889 reachable, 0 orphans. The extraction waves did not strand a package in the
lock.

## Credential-dependency guard

`scripts/release/tests/test_provider_probe_absence.py` still carries the check
#146 added, `test_app_manifest_names_no_credential_store_dependency`, which
refuses `aes`, `cbc`, `pbkdf2`, `rusqlite`, `secrecy`, `sha1`, and `zeroize` in
the app crate's manifest. `ureq` is deliberately absent from that list for the
rich-chat reason above. The modified manifest still passes it, and the whole
release-scripts suite passes (401 tests, 18 skipped).

## Checks run

| Check | Result |
| --- | --- |
| `cargo check --locked -p spaceport-gpui --all-targets` | passed |
| `cargo clippy --locked -p spaceport-gpui --all-targets -- -D warnings` | passed |
| `cargo test --locked -p spaceport-gpui --bin spaceport` | passed — 3686 passed, 0 failed, 10 ignored |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/release/tests` | passed — 401 tests, 18 skipped |
| `python3 -m unittest -v test_provider_probe_absence` (from `scripts/release/tests`) | passed — 6 tests |
| `cargo fmt --all -- --check` | passed |
| `just check-sorted` | passed |
| `just check-file-size` | passed |
| `cargo tree` per-target snapshots, before and after | direct edges and resolved sets as tabled above |

Not run: the Linux compile legs (`just lint-linux` needs Docker and the Linux CI
runner is the gate anyway), the Windows target lint, and the native GUI suites
(`just test-gui`, `just test-sidebar`, `just test-perf`) — none of them can
observe a Linux-gated manifest edge, and none was asked for.

## Follow-ups this audit found but did not take

1. **Three workspace dependencies no member names.** `[workspace.dependencies]`
   in the root `Cargo.toml` declares 106 entries; 103 are named by a member and
   three are not: `if-addrs`, `mdns-sd`, and `zbus`. `if-addrs` and `mdns-sd`
   resolve into neither `Cargo.lock` nor any build — they are the last remnant
   of the automatic device suggestions that `75ad831f7` ("feat: remove automatic
   device suggestions") deleted, including that commit's own two gpui edges,
   while leaving the root entries behind. `zbus`'s package is still in the lock
   through `oo7`, so only its root entry is dead. Deleting those three lines is
   mechanical and changes no resolved graph, but it edits the workspace manifest
   that every member shares, which is outside this task's scope.
2. **`font-kit` is a normal dependency used only by a test**, as described under
   Retained deliberately.
