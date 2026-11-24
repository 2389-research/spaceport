# wx48 — settings stage 1: the gpui-free policy moves into `spaceport-settings`

Branch `feat/wx48-settings-stage1`, cut from `origin/main` (`c0dd95889`). This
is stage 1 of `notes-settings-extraction.md` §3: the save queue, the drafts,
the section list, the row catalog and the control tables are now the settings
crate's, and the Settings window keeps only what draws them.

Three commits:

| Commit | What it holds |
| --- | --- |
| `99238579b` | the drafts and the save queue (`save_queue`, `sidebar_draft`, `drafts`, `theme::{Scope, filter_names}`) |
| `0eea92aeb` | the sections, the catalog and the field tables (`section`, `catalog`, `fields`) |
| this one | the record |

## What moved (moved, not copied)

| Was | Is | Lines |
| --- | --- | --- |
| `settings_window/persistence/lifecycle.rs` | `spaceport-settings/src/save_queue.rs` | 262 → 406 |
| `settings_window/persistence/operation.rs` | the same file's `Request`, `Operation`, `SharedWrite`, `Reconciled`, `reconcile` | 95 |
| `settings_window/sidebar.rs`'s draft half | `spaceport-settings/src/sidebar_draft.rs` | 180 → 184 |
| the `ThemeDraft`/`LayoutDraft`/`SidebarDraft` globals | `spaceport-settings/src/drafts.rs` | 230 → 161 |
| `settings_window/section.rs` | `spaceport-settings/src/section.rs` | 165 → 159 |
| `settings_window/catalog.rs` | `spaceport-settings/src/catalog.rs` | 214 → 210 |
| `inputs.rs`'s `NumberField`/`DropdownField`, `pages.rs`'s position and on/off tables, `Spacing`, `face_message`, `size_control`, `padding_ids` | `spaceport-settings/src/fields.rs` | ~150 → 223 |
| `themes.rs`'s `Scope` and `filter_names` | `spaceport-settings/src/theme.rs` | 37 → 31 |

`save_queue.rs` is longer than what it replaced because the queue is generic:
`Lifecycle<Q>` queues the caller's own write, and `Request<L, E>` carries the
load result and the error the caller uses. `fields.rs` holds the tables that
were spread over four app files.

The app keeps a façade per moved module — `settings_window/section.rs` (6
lines), `catalog.rs` (33), `drafts.rs` (58) — so every importer outside
`settings_window/` still compiles unchanged. `settings_window.rs` re-exports
the seven draft functions `menu/settings.rs`, `theme_picker.rs` and
`settings_panel.rs` call, at the paths they already used.

## The four inversions

The design named three; a fourth came out of the code.

1. **One drafts value.** `ThemeDraft`, `LayoutDraft` and `SidebarDraft` (three
   GPUI globals, each with its own revision) become one `Drafts` the app keeps
   in one global (`settings_window/drafts.rs`). The crate answers what a draft
   does to a load (`drafts.apply_loaded_sidebar(config, revision)`); the app
   keeps the window.
2. **Two lookups for search.** `catalog::search(query, message, english)`
   matches through the caller's two wordings, so the crate names no language.
3. **A row's words are the app's.** `Entry` keeps its `title` and `description`
   as the message ids they always were; `settings_window::catalog::{title,
   description}` word them for the pages, the search results and `ui::row`.
   `Section::label`, `Group::label` and `Section::lead` are gone: the crate
   returns ids (`lead() -> Option<&'static str>`) and the app resolves the seven
   call sites that worded them.
4. **A close is a goal, not a window.** `Lifecycle` cannot name
   `gpui::AnyWindowHandle` — this crate has no GPUI — so `closing()` answers
   whether a close is pending and `take_close()` returns whether the goal was
   taken. The window keeps `close_when_idle: Option<AnyWindowHandle>` and takes
   it in `finish_close`, where it already deferred the removal. The field is
   set in `should_close` and taken exactly once, so a save failure that drops
   the goal leaves at worst a stale handle the next close overwrites.

## What stayed app-side, and why

- **The keymap in `Loaded`.** `Request<L, E>` is generic over the load result
  and the error, so the crate never names `crate::keymap::Keymap` nor the app's
  `Error`, and `Loaded` keeps the pair a published appearance needs. The
  design's §2.3 Option A splits `Loaded` only when the crate's *loader* moves;
  stage 1 does not move the loader, so splitting it here would have added a
  type with no user.
- **`Spacing`'s preset values.** `spacing_value` and `overridden` read
  `SidebarMetrics`, which lives in `spaceport-sidebar` and depends on this
  crate. The enum, its ids, band and range moved (`fields::Spacing`); the two
  config-reading functions stay in `pages/sidebar.rs` as free functions, since
  an inherent impl cannot be added outside the defining crate.
- **`NumberPreview`, `HexEdit`, `ThemeIntent`, `ConfigWatch`, `Sample`** — the
  state a window holds while editing.
- **Every page, control and the theme browser** — stages 2 to 6.

## The app's alias list shrank

`crates/spaceport-gpui/src/config.rs` is the temporary re-export of the
settings crate's names. Three names left it because this stage removed their
last readers in the app: `SidebarStyle` and `SidebarOverrides` (the draft half
and `Spacing` now live in the crate, and the tests import them from
`spaceport_settings`) and `MAX_PADDING` (the field tables). `Config` stays.

## Tests moved with their code

New in the crate (23 tests): `tests/save_queue.rs` (bounds, close, quit, the
dropped sidebar draft), `tests/settings_drafts.rs` (each draft's revision fence
and a sidebar draft's touched keys), `tests/sidebar_draft.rs` (what a commit
writes, what a quit writes, a host added then dropped, a rebase),
`tests/settings_catalog.rs` (search through both lookups, unique ids, one row
per feature flag), `tests/settings_sections.rs` (the group order and the nav
index), `tests/theme_names.rs` (`filter_names`, moved whole).

Left app-side on purpose:

- `pages_and_catalog_declare_the_same_rows` and `pages_draw_the_text_search_indexes`
  draw a window; they are the check that the page and the catalog agree.
- `every_catalog_row_names_english_messages_without_arguments` reads the app's
  `i18n::takes`, so it is the app that proves the crate's ids are messages.
- `pending_writes_are_bounded_and_saturation_reports_the_refusal` drives the
  window and reads its footer error; the queue's own bound is the crate's test.
- every `theme_drafts`, `layout_drafts` and `sidebar_drafts` case that needs a
  rendered window.

## Validation

Run on this branch, macOS, no desktop available for native screenshots (the
stage changes no pixels; the row ids and message ids are unchanged, and the
whole `settings_window` test suite draws it headlessly):

- `cargo check -p spaceport-settings` and `cargo check -p spaceport-gpui
  --all-targets --features integration-test` — clean, no warnings.
- `cargo clippy -p spaceport-settings --all-targets --features test-support --
  -D warnings` and `cargo clippy -p spaceport-gpui --all-targets --features
  integration-test -- -D warnings` — clean.
- `cargo test -p spaceport-settings --lib` — 326 passed (303 before).
- `cargo test -p spaceport-gpui --features integration-test --bins` — 3532
  passed, 10 ignored, 0 failed.
- `cargo fmt --all --check`, `just check-file-size`, `just check-sorted` and
  `just check-i18n` — clean.

Not run: `just ci`, the Windows and Linux lint legs, and any native window
check.

## What stage 2 has to know

- `ui::row` still takes `&catalog::Entry` and words it through the app's
  `catalog::title`/`description`; stage 2 passes the two lookups into the row
  kit the same way search takes them.
- `fields::Spacing` and `fields::NumberField` are the kit's inputs now; their
  ids are the same strings, so a moved test that looks one up by selector keeps
  working.
- `Lifecycle<Q>` is the queue: a second window with saves of its own queues its
  own write type, and `Request<L, E>` already carries any load result.
