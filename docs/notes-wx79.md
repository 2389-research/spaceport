# wx79 part 1: the README and the reachability snapshot

Branch `feat/wx79-docs-1`, cut from `e99d575b6c` (main after `chore: Add a shape
guard for artifact-host task writes`). This part is documentation only: no crate,
no binary, no test and no message changed. `origin/main` moved to `d1195274fe`
(`refactor(keymap): Move the keymap's resolution into spaceport-keymap`, #243)
while the work was in progress. That merge touches `crates/spaceport-gpui/src/{actions,
config,error,keymap,terminal}.rs` and `crates/spaceport-keymap/**`, none of which
this cut edits or cites, so the diff applies on either base. The snapshot below
describes `e99d575b` and says so in its own first line.

## 1. What changed

| File | Change |
| --- | --- |
| `README.md:7` | The preview link was five previews old (`v20261117.1`). It now points at `v20261125.1`, the newest prerelease: published 2026-11-25T05:09:19Z, checked with `gh release list --repo 2389-research/spaceport --limit 5`. |
| `README.md:12` | The reachability link named `removal-reachability-50b9063be.md`, a snapshot of a tree that no longer exists. It now names the new one below. |
| `README.md:19-23` | "This branch still contains some of those paths" became what actually remains: the tab renderer in `crates/spaceport-browser`, and an Automations page that says this Core has no job feed. The sentence now also says a tab shows a device list or a Core terminal and never a web page, so nothing in the UI opens a browser tab, and a URL opens in the system browser. |
| `README.md:47-57` | The mobile section said the two apps "share the Rust client". It now says they link one Rust controller from the separate Cargo workspace in `mobile/rust`, which consumes `spaceport-client`, `spaceport-panes`, `spaceport-backend` and `spaceport-truncate` by path, and names the rules that live once because of it. |
| `README.md:68-72` | Added the two entry pointers a new contributor was missing: `just` with no arguments plus [just-operations](just-operations.md), and [gpui-split-plan](gpui-split-plan.md) for what has left the app. |
| `scripts/release/README.md:34-39` | The preview walkthrough named `20261117.1` as if it were current. It now records that `20261117.1` first completed the route with Sparkle enabled, names `20261125.1` as the newest, and keeps the qualification record attached to the build it actually qualifies. |
| `docs/spaceport/evidence/removal-reachability-e99d575b.md` | New: all 19 removal-plan entries, re-derived at this commit. |

## 2. The snapshot, and why a link swap would not have done

Five snapshots already existed before this one. The newest of them,
`removal-reachability-7f15ad1a.md`, still records RM-GIT, RM-GITHUB, RM-TELEPORT
and RM-DISPATCH as **Present** and names `crates/sheprd-gpui/src/...`,
`crates/sheprd-vcs` and `sheprd-remote`. None of those paths exist. So the README
could not be fixed by pointing at a newer file: every label had to be re-derived,
and the old files stay where they are as history.

The new file follows `removal-reachability-7f15ad1a.md`'s shape (the definitions
paragraph, the 19-row table, proof limits) and adds the grep framing from
`removal-reachability-50b9063be.md`, because that is what makes a label checkable.
Every row's claim was grepped in this tree first, with test sources excluded, and
every `file:line` was read before it was written down. The result:

| | Plan items |
| --- | --- |
| **Absent** | RM-IDE, RM-GIT, RM-GITHUB, RM-DISPATCH, RM-BROWSER, RM-CLOUD, RM-TELEPORT, RM-AUTOMATION, RM-MODS, RM-PHONE, RM-PROVIDERS, RM-ANNOTATION |
| **Present** | RM-LEGACY, KEEP-CORE, KEEP-TERM, KEEP-CONTROL, KEEP-SHELL |
| **Partial** | KEEP-SESSION, KEEP-CHAT |

Two labels are qualified rather than flat. RM-BROWSER is **Absent, engine and page
tab**: the tab renderer stays, a tab shows only `Devices` or `CoreTerminal`, and
`open_workspace_page` hands a `WebUrl` to the system handler. RM-PROVIDERS and
RM-ANNOTATION are **Absent** for their route only, because their display metadata
and ordinary selection stay.

Three residues are recorded in the snapshot instead of fixed, since a read-only
pass should not edit code: the example config still lists `git_menu` and
`toggle_annotate` as unbound commands the catalog no longer has, tells a reader
that **Settings → Cloud devices → Container** edits the container engine key while
no settings page names a cloud device, and `main.rs:135` keeps a dangling
`// Git status, without GPUI.` comment above an unrelated import.

## 3. Verification

```sh
git diff --check
uv run --with jsonschema==4.26.0 python docs/spaceport/tools/spec.py check
cd scripts/release/tests && python3 -m unittest <the 24 absence test modules>
```

`spec.py check` passed schema validation and all 381 local links (it was 328
before this file). Every relative link in the three edited files resolves. The 24
absence test modules that the snapshot cites as boundaries are 65 text checks, and
all 65 passed, so each **Absent** label rests on a guard that currently holds,
not only on a grep. Nothing here builds or launches the app; the snapshot's own
proof limits say so, and all eight gaps and the acceptance legs it reports were
read from `plan/gaps.json` and `plan/acceptance.json` rather than copied.
