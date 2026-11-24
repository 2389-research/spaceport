# wx52: the crate tables catch up with the extraction program

Branch `feat/wx52-crate-tables`, cut from `origin/main` (`79cbc1476`, after
#216–#220). Docs only: no code, no build, no new dependency.

Two places describe the crates and both lagged the extractions. `AGENTS.md`'s
table was missing `spaceport-release-notes` (#206) and had not been reread
since the sidebar's stages S6–S8a and the chat's stages 3b–3d; the
`spaceport-widgets` row #220 added was written from that commit's own survey
rather than from the crate. `docs/gpui-split-plan.md` still named the app by
its pre-rename path in the `ago`, `words` and `lenient` sections.

## 1. The survey behind each row

Read from `crates/<crate>/src` on `79cbc1476`, with each crate's own `lib.rs`
doc comment as the cross-check.

**`spaceport-sidebar`** — 23 modules, not the six the old row named: `agents`,
`animated` (+`grid`, `orca`), `cell`, `device_drag`, `device_scope`,
`device_tags` (+`error`, `rules`), `direction`, `hover`, `layout`, `layouts`
(+`herdr`, `minimal`, `orca`, `parts`, `superset`), `metrics`, `order`,
`origin`, `paint`, `reorder`, `row` (+`badge`, `configured`, `paint`), `slide`,
`sticky`, `tag_groups`, `tokens`, `wash`, `workspaces`, `worktree_notes`. The
rewritten row adds what S6–S8a moved in: the layout and density policy with the
four row layouts and the mode that picks one, the daemon's configured tokens,
which devices are listed and how each is reached, the host wash, the status
sets drawn rather than loaded, the `paint::Painters` seam, the right-to-left
mirror, and the interaction math (the two drags, the resting pointer, the
slide).

**`spaceport-chat-core`** — 14 modules. The row now names what stages 3b–3d and
their follow-ups added: the rich block model with the parse that reports
its pictures and diagrams as wants, the visual model's identity, bounds and
diagram theme, the prepared entry model, the `Labels` the app words and hands
down, the widgets catalog agent JSON is read into, the compaction-summary
recognizer, and the selection model (`Region`, `Caret`, `Document`, `Segment`)
with the 4 MiB copy bound it reads from `spaceport_client::text::MAX_COPY_BYTES`
(`selection.rs`).

**`spaceport-widgets`** — `direction`, `toggles`, `accessibility` and no
`error.rs`: nothing in the three modules fails. Corrected against #220's row:
the mirror *rules* are still `spaceport-sidebar::direction`'s, keyed on the
`bool` a painter can read, and this crate delegates to them; the direction is
published by `i18n::current::replace` (per thread under the `test-support`
feature, so one test's pseudo-locale cannot mirror another's layout); every
module keeps the path the chrome already names it by. The chrome's sites reach
it through those facades — 49 files and 59 call sites for `direction` alone,
per `notes-wx49.md` §1 — so the row describes the seam rather than quoting a
count that drifts with every commit.

**`spaceport-release-notes`** — new row, from #206: `Line`, `Kind`, `Mark`,
`Link`, `Limits`, `parse`, `parse_within`, `note`, `bare_url`, `MAX_URL`. It is
placed before `spaceport-chat-core` because the chat's rich text and its
compaction summary read the same line model.

## 2. The split plan

Four cells in `docs/gpui-split-plan.md` said `crates/sheprd-gpui/src/…` for a
path that has been `crates/spaceport-gpui` since the rename: the `ago` and
`words` before/after tables and `lenient`'s. The `osc52`, `sound` and `truncate`
sections and the "What moved" summary table still say `sheprd-gpui`; they were
left alone, since this task named the three sections and a wider sweep of a
historical table is its own decision.

## 3. Checks run

| Check | Result |
| --- | --- |
| `just check-sorted` | all keep-sorted regions sorted (10 tests, OK) |
| `git diff --check` | clean |
| markdown links in both files, resolved by a one-off walk of every `](…)` target | 4 in `AGENTS.md`, 3 in the plan; none unresolved |

No code check ran: nothing outside `*.md` changed.

## 4. Found while surveying, not fixed

`crates/spaceport-sidebar/src/lib.rs`'s doc comment has two overlapping tails
from the #213/#214 merges: lines 22–31 repeat the first paragraph, and lines
33–35 still say the app "decides which side a row's leading edge is on, paints
every row, and words every status", which the paragraph above it contradicts
(the crate paints, and the direction is handed in). It is the crate's own
description, so it wants its own commit.
