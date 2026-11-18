# The last accepted P3 findings of the eight-reviewer audit

The small items queued behind the fix wave (#111–#114). One commit each, no
behavior change beyond the two behavior fixes below. None of the files those
PRs own was touched.

| Finding | Outcome | Commit |
| --- | --- | --- |
| 1. A refused chat prompt entered recall history | Fixed | `cdcad8663` |
| 2. `Glide::reset` left the pending-scroll stash set | Fixed | `2d33b9294` |
| 3. The crate-wide test fixture lives inside `sidebar` | Deferred to its own PR, by decision | — |
| 4. Duplicate `Mutes::any` lookup per host in the rail | Fixed | `76307d5e0` |
| 5. Needless clone in `presets()`, stale `Chrome::device_order` doc | Fixed | `8f59effcf` |
| 6. A snooze preset past midnight showed a bare clock | Fixed | `a520cc8cf` |

## 1. A refused chat prompt entered recall history

`submit_chat_prompt` pushed the prompt into `pane.history` before
`pane.send`, so a send the link refused (a full pending queue, a read-only
transcript) still recorded a prompt that never reached the agent. Up then
recalled text identical to what the composer still held.

The push now happens in the `Ok` arm, and `Command::Prompt` takes
`text.clone()` because the history push needs the text after the command
owns it. History stays what `ChatPane::history` documents: prompts this
window sent.

Tests, `src/chat/tests/recall.rs`:

- `a_prompt_the_window_could_not_send_stays_out_of_history` fills the
  pending queue to `MAX_PENDING`, types a prompt, presses Enter, and checks
  the pane reports an error, the composer keeps the draft, and Up recalls
  nothing. Reverted against the pre-fix `window.rs`, this test fails with
  `left: "keep me", right: ""`, so it is a real regression test.
- `a_prompt_the_window_sent_is_recalled_before_the_agent_echoes_it` sends a
  prompt, reads it off the peer, and checks Up recalls it before any echo
  arrives in the transcript.

## 2. `Glide::reset` left the pending-scroll stash set

`Glide` keeps three `Rc<Cell<_>>`: `following`, `scrolled`, and `before`,
the value `wheel_scrolled` stashes as the list reports a wheel. `reset`
cleared the first two and left `before` holding whatever an unfinished
reversal had put there. Nothing reaches a misbehavior today, because only a
wheel the list also bubbles to `on_scroll` reads it, but one stale `Rc` is
the kind of thing the next reader trips over.

`reset` now clears `before` alongside `scrolled`, with a comment saying
that the wheel which stashed it never got that layout. Test
`resetting_following_leaves_no_scroll_for_the_next_wheel_to_read` in
`src/chat/glide/tests.rs` stashes a pending scroll, resets, and asserts all
three cells are cleared.

## 3. The crate-wide test fixture lives inside `sidebar` — deferred

The reviewer is right that `crate::sidebar::layout_tests` is the crate's
shared fixture rather than a sidebar detail. It is gated
`#[cfg(any(test, feature = "integration-test"))]` and holds
`fixture_window`, `fixture_core_window`, `full_draw`, `snapshot`,
`run_frames`, `SidebarFixture`, `PaintedProbes`, `within_device_pixel`, and
`REPO_KEY`.

Measured before deciding anything:

- 691 references in 319 files outside `crates/sheprd-gpui/src/sidebar/`
  (708 references in total, 336 files).
- There is no existing crate-level shared-fixture module to move it into.
  The repo's other crate-wide fixture, `crate::window::MockPeer`, is defined
  in `window/transfers.rs`'s `tests` module and re-exported at
  `crate::window`, so the nearest conventional home is a test-only
  `window/fixture.rs` re-exported at `crate::window` — still 691 edits. A new
  `crate::fixtures` module would invent a location the repo does not have.
- The module mixes the shared window fixture with sidebar-only paint probes,
  so the split boundary is itself a decision.

Options weighed:

| Option | Cost | Notes |
| --- | --- | --- |
| A. Move now | 1 commit, ~319 files, no behavior change | Maximal merge-conflict surface while four PRs are in CI |
| B. Defer to its own PR | A note | Chosen |
| C. Narrower variant (shared names only, or re-export without rewriting sites) | Less churn | Rewriting is what the finding asks for; a bare re-export changes nothing about `sidebar`'s import count |

Decided: B. A 319-file rename of a test-only module is a structural
migration, not a P3 drive-by, and this repository optimizes against
parallel-PR conflicts. The deferred PR should also settle the split: the
cleanest end state is probably `window/fixture.rs` holding the truly shared
pieces (`fixture_window`, `fixture_core_window`, `full_draw`, `snapshot`,
`run_frames`, `REPO_KEY`) with `sidebar` keeping its own paint probes
(`PaintedProbes`, `within_device_pixel`, `VerifyChildGeometry`), decided by
counting who uses which.

## 4. Duplicate `Mutes::any` lookup per host in the rail

`SidebarView::render_rail` read `Mutes::any(cx)` once before its host loop
and then again inside the loop for each host's mute mark. The two calls take
the same `cx` and read the same global, with nothing mutating in between, so
the per-host test now uses the one read. No semantics changed: `Mutes::any`
returns `Option<&Mutes>`, which is `Copy`, and the earlier read was already
held across the loop by the `silenced` map.

No test: the change is a rebinding of an identical value, and the rail's
existing `sidebar::rail` and `sidebar::layout_tests` coverage (313
`sidebar::` tests, all green) exercises the muted-host path.

## 5. Needless clone in `presets()`, stale `Chrome::device_order` doc

`overview::presets::presets` cloned the tomorrow option only because the
next-week filter compares against it afterwards; the builder now passes each
preset's time by reference (`push` takes `&DateTime<Tz>`), so the option is
borrowed for the comparison and the push alike. The remaining `now.clone()`
calls are required: `DateTime<Tz>` is not `Copy` under the generic
`Tz: TimeZone` bound.

`Chrome::device_order` still carried the doc line "The notes panel's width,
once dragged; `None` is its default", left behind when the notes panel was
removed; it now documents only the device order.

No new test: both are a borrow and a comment, and the existing
`overview::tests::presets` and `preferences::tests::device_order` tests
cover the behavior.

## 6. A snooze preset past midnight showed a bare clock

"In an hour" and "In three hours" printed only `HH:MM`, so at 23:30 they
read `00:30` and `02:30` — times that look like earlier today. A preset that
ends on another day now shows the weekday with its time, reusing the
existing `overview-weekday-time` message (no new Fluent string, no locale
churn). The tomorrow, this-evening, and next-week presets keep their
formatting: their labels already name the day, and "Tomorrow" would read
oddly as "Thu 09:00".

Test `a_preset_past_midnight_names_its_day` in `src/overview/tests/presets.rs`
pins Wednesday 14 October 2026, 23:30 to `Thu 00:30`, `Thu 02:30`, `09:00`,
`Mon 09:00`, with no wall-clock read.

## Verified

- `cargo test -p sheprd-gpui --bin spaceport -- chat::` — 526 passed.
- `cargo test -p sheprd-gpui --bin spaceport -- sidebar::` — 313 passed.
- `cargo test -p sheprd-gpui --bin spaceport -- overview::` — 85 passed.
- The refusal test above was re-run against the pre-fix `window.rs` and
  failed as expected before being restored.
- `cargo fmt --all -- --check`, `just check-file-size`, `just check-sorted`,
  `just check-i18n` (2125 messages in 113 files), `just check-commits`.

Deferred: item 3, as decided above; and no native/desktop QA, which none of
these findings needs.
