# i18n stage 2: delete the `Labels` seams

The second stage of the owner's P1 ruling
(`docs/notes-owner-decisions-2026-11-25.md`): the translation lookup moved into
`spaceport-i18n` (#250), and each `Labels` struct a crate took from the app is
deleted in turn. Base revision: `f12350f8d8` (origin/main, after #250). Branch
`feat/wx84-i18n-2`.

It changes no behaviour. Every message keeps the id and the `.ftl` file it had;
what changes is who reads it.

## One tree, not one file per crate

Stage 1 moved the whole `locales/` tree into `crates/spaceport-i18n`, named per
*feature* (`settings.ftl`, `chat.ftl`, `state.ftl`, `sidebar.ftl`,
`chat-text.ftl`, …), and `crates/spaceport-i18n/locales.rs` refuses an id that
does not start with its file's name and a hyphen. A `settings-window.ftl`
could hold only `settings-window-*` ids, so moving a crate's keys into a file
named after the crate would rename the ids themselves. The crate that first
named a key therefore keeps naming it, from the file it is in: `fl!` is
checked at compile time against the whole tree, and `scripts/check-i18n.py`
still reports an English message nothing names.

## Verdict per seam

| Seam | Carried | Verdict |
| --- | --- | --- |
| `spaceport-settings-window`'s `ui::Labels` and `navigation::Labels` | a message lookup, the band a number field takes, the label that follows Herdr's value, and the footer's unsaved drafts | **deleted.** The kit names its own ids and words them; an id that arrives as data — a catalogued row's title or description, a section's or group's name — is read through `message`. `MESSAGE_IDS` goes with it, and a crate test over `takes` marks the ids it names as consts. |
| `spaceport-chat-core`'s `labels::Labels` | ten lookup words written into prepared bodies and their selectable text, plus the one `turn` slot that was app-side *code* | **deleted.** `prepared` and `selection` call `fl!` themselves; a pane keeps the replacement discipline by forgetting its prepared text when the shown locale changes. The `turn` slot's wording was not a lookup: the measurement stayed in the crate and `text::turn`/`text::duration` now write it, from the keys the app used to read. |
| `spaceport-live`'s `Labels` | three lookup strings, and two `fn` slots | **mostly deleted.** The three strings are the crate's now. The two slots were never lookups: `localize` words a refused client error, which stays the app's (a later stage may move each `impl Localize` beside its `Error`), and `describe` reads the sidebar's naming of a workspace, which cannot move below the app. They become `words::{Localize, Describe}` and are passed as `fn` items, so the struct of words is gone and the behaviour stays where it can live. |
| `spaceport-sidebar`'s `paint::Word` and `Painters::words` | one word: Orca's chip naming a repository's own checkout | **deleted.** It is a lookup, not the painter contract: the two seams a row painter truly cannot own are the element a label is painted as and the window's layer for a moving mark, and both stay. The layout reads `sidebar-primary` itself, and `Painters` stays `Copy`. |

The two crates the ruling leaves alone keep returning values, as
`spaceport-release-notes` and `spaceport-ago` do: their words are also printed
in English on the command line, so the app writes them.

## What the checks see

`scripts/check-i18n.py` reports an English message no source names, and it
counts `fl!` and `message` call sites the same way it counted the ids a
`MESSAGE_IDS` table held: as string literals in non-test code. Every id the
Settings kit and the live state used to list is now spelled at its call site,
so the tally is unchanged — 2,114 messages in 114 files, all valid and used —
without weakening the script.

## Verification

- `cargo check -p spaceport-gpui --all-targets` after each commit, with the
  crate that commit changed
- `cargo test -p spaceport-settings-window -p spaceport-chat-core -p spaceport-live -p spaceport-sidebar`
- `cargo test -p spaceport-gpui --bin spaceport` (3,340 passed)
- `just check-i18n`, `cargo fmt --check`, `just check-file-size`
- Not run here: the native window checks and the `integration-test` probes that
  click real controls (`just test-gui`/`just test-sidebar`), which need a
  desktop and a pinned `herdr`; CI's platform jobs cover them. The headless
  `en-XA` leak tests ran in the same `--bin spaceport` pass.
