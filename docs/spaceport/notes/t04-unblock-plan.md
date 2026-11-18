# T04 unblock plan: the terminal link gap

Proposal for Doctor Biz, written at `50b9063be`. **Unblocking T04 needs an owner
decision.** This note changes no register: [`plan/tasks.json`](../plan/tasks.json)
keeps T04 `blocked` with `must_close_gaps: ["G-LINK"]`, and
[`plan/gaps.json`](../plan/gaps.json) keeps G-LINK `open`, until that decision is
recorded as a decision ID and applied by whoever owns the plan files.

## What G-LINK blocks

- **Closing T04, not landing families.** T04 is the only task whose
  `must_close_gaps` is G-LINK alone. Its acceptance set is A28, A29, A30, A33 and
  A34 ([`plan/acceptance.json`](../plan/acceptance.json)), all `not_run`. G-LINK
  names the A30 leg (no plugin/handler behavior on a click) and the A33 leg
  (retained wrapped URL/file/chat/native interaction still works). Its
  `required_evidence` asks for a side-effect-free typed link target and span for
  an exact terminal identity, content revision and viewport cell, or equivalent
  proven surface semantics; its `forbidden_workaround` forbids keeping
  plugin-capable activation for some clicks, guessing a wrapped URL from adjacent
  rows, and disabling wrapped links while claiming retained UX.
- **T10 behind it.** T10 ("Remove all excluded reachable behavior") depends on
  T04, so its deliverable cannot close either while T04 is blocked.

## The one live excluded route

Everything else in T04 is source-absent at HEAD (see the family table below).
What remains is the inherited terminal link activation, which is a live
daemon-side plugin execution path:

- A press with the link modifier held produces `PressedLink.cell`
  (`crates/sheprd-gpui/src/window/links.rs:467`), and
  `activate_terminal_link` sends `LinkRequest::Activate`
  (`crates/sheprd-gpui/src/window/links.rs:513`), gated on
  `live.supports_link_activate` (`crates/sheprd-gpui/src/state.rs:516`).
- `LinkRequest::Activate` maps to `Method::PaneLinkActivate`
  (`crates/sheprd-gpui/src/links.rs:45`, wire name in
  `crates/sheprd-client/src/method.rs:51`); `finish_activation` treats
  `handled: true` as "a plugin claimed it, act no further", a
  `plugin_link_failed` refusal as a spent click
  (`crates/sheprd-client/src/response.rs:51`), and a declined reply's `url` as
  the address to open.
- The behaviour is not hypothetical: the opt-in live fixture
  `crates/sheprd-client/tests/live/plugins.rs` drives a pane whose daemon runs a
  `[[link_handlers]]` entry — the handler writes `link.txt` for the file link it
  claims and the other link comes back `handled: false` with its URL.

## Why no local substitute closes it

Four independent constants, each visible in source:

1. **The activation reply is the only wrap-aware URL reader.** The row-local
   detector refuses a multi-row plain run whose target is not a disk-confirmed
   path (`crates/sheprd-gpui/src/terminal/links.rs:318` `frame_link`, guard and
   comment at `:374–380`).
2. **`pane.link.resolve` returns regions only.** `ResolvedLink::from_response`
   deserializes `regions` and nothing else
   (`crates/sheprd-gpui/src/links.rs:253`).
3. **The Herdr 1 cell cannot carry a wrap bit.** `CellData` is
   symbol/fg/bg/modifier/skip/hyperlink
   (`crates/sheprd-protocol/src/wire.rs:260`), and its vendored field order is a
   compatibility contract, so the client cannot learn that two rows are one line
   from the painted surface.
4. **Pinned Core advertises no link route.** `evidence/t00-core-runtime.json`
   (0.1.18, generation 1) lists 32 methods, none of them a link or URL
   inspection method.

So there is no shim, flag or local inference that satisfies the gap; the
remaining paths are a runtime change Spaceport cannot make, or an explicit
narrowing of retained link behaviour that only the owner can approve.

## Can T04 be split so families land without G-LINK? Yes — and it already is

The removal work has been landing as separate mergeable slices all along: the
`evidence/t04-*.md` records are those slices. A source read at HEAD
(`grep` over `crates/` for each family's markers, excluding tests) finds no live
excluded route in any family except the link activation:

| Family | Live at HEAD | Boundary that shows it |
| --- | --- | --- |
| RM-IDE | no | no `px0`/`code_view`/`code_search`/`open_in_editor` production route; `scripts/release/tests/test_px0_source_absence.py` |
| RM-GIT | no | `sheprd-vcs`, `gix`, `git2` absent; `crates/sheprd-gpui/src/menu/git_read_removal_tests.rs` |
| RM-GITHUB | no | crate and `octocrab` absent; `config/tests/removed_github.rs`, `Error::RemovedGitHub` (`crates/sheprd-gpui/src/config.rs:589`) |
| RM-DISPATCH | no | orchestrator crate absent; `group_menu` and `cli` refusal tests |
| RM-BROWSER | no | no `webview`/`wry`; `scripts/release/tests/test_webview_source_absence.py` |
| RM-CLOUD | no | `crates/sheprd-remote` holds only `agent`, `host_script`, `snapshot`; provider source-absence tests |
| RM-TELEPORT | no | `test_teleport_engine_absence.py`, `test_teleport_native_absence.py` |
| RM-AUTOMATION | no | `test_automation_absence.py` |
| RM-MODS | **yes, one route** | client mod/Luau crates absent; the daemon link activation above |
| RM-PHONE | no | `test_phone_absence.py` |
| RM-PROVIDERS | no route | probe/credential graph absent (`test_provider_probe_absence.py`); display metadata retained under D12, and G-OBS is a separate gap |
| RM-ANNOTATION | no | `config/tests/terminal_annotations.rs` |
| RM-LEGACY / KEEP-* | retained | D58 property, not T04 work |

Two corrections to the register's own picture:

- The reachability snapshot in T04's evidence,
  [`removal-reachability-54cb41c2.md`](../evidence/removal-reachability-54cb41c2.md),
  is from 2026-11-08 and 518 commits behind HEAD. It still records RM-GIT,
  RM-GITHUB and RM-DISPATCH as present, which the merge history has since
  removed, so it understates how much of T04 has landed.
- A34 (`Customization/accessibility`) has no evidence list at all, so T04 still
  owes its retained-customization leg whatever happens to G-LINK.

Three small residuals sit outside the gap and can ride with any cleanup slice:
`crates/sheprd-gpui/src/browser/code/tests.rs` (56 lines) has no `mod code;`
declaration, so its inherited `with_token` tests never compile or run;
`icons/azure-devops.svg` is still embedded with no remaining user
(`crates/sheprd-gpui/src/icons.rs:234`); and the `worktree`/`notes` CLI rows a
2026-11-17 scan flagged are retained notification features, not Git workflows.

Conclusion: a *further* split is not what T04 needs. Independent families are
already separable and mostly merged; what is left is one decision and one small
slice.

## Staged order, each stage mergeable alone

0. **Reachability refresh (docs).** A new `evidence/removal-reachability-<rev>.md`
   at current main, replacing the Nov 8 snapshot as the current navigation record.
   No product change; mergeable now.
1. **Owner decision on the link contract** (options below). Nothing in stages 2–4
   is safe to write before it, because the decision fixes what the retained-link
   tests must assert.
2. **The link slice** (next section). One crate pair, no wire or runtime change.
3. **Acceptance legs.** A30's negative plugin-activation check plus retained
   wrapped-URL/file/chat interaction, then A28, A29, A33 and A34 runs; the native
   legs need macOS, Windows and Linux, and A34 needs its own evidence list first.
4. **Register update.** T04 status, G-LINK status and any new decision ID. D71 is
   the working precedent for narrowing a gap's scope by decision rather than by
   editing prose; today D71 explicitly leaves G-LINK "unchanged".

## Smallest first code slice

Under either substantive option the cut is the same edit, and it is the only
T04 family route left:

- `crates/sheprd-gpui/src/window/links.rs` — drop `PendingActivation`,
  `links.activating`, the request in `activate_terminal_link`, and
  `finish_activation`.
- `crates/sheprd-gpui/src/links.rs` — drop `LinkRequest::Activate`, `Activation`
  and the `pane_link_activated` / `plugin_link_failed` mapping.
- `crates/sheprd-gpui/src/state.rs` — drop `supports_link_activate`.
- `crates/sheprd-gpui/src/terminal/links.rs` — `pane_hyperlink_at` exists to hand
  an unreadable hyperlink to a handler (`window/links.rs:165`, claimed at `:455`)
  and also drives the hand cursor (`window/input.rs:114`); keep or narrow it as
  the decision requires.
- `crates/sheprd-client/src/method.rs` — drop `PaneLinkActivate`.
- `crates/sheprd-client/src/response.rs` — drop `PluginLinkFailed`.
- Tests: `window/links/tests/file_handlers.rs` (the three handler behaviours)
  becomes an absence check; `window/links/tests/cases.rs` (wrapped URL),
  `content_updates.rs`, `crates/sheprd-gpui/src/links/tests.rs`,
  `crates/sheprd-client/src/method/tests.rs`, and the opt-in
  `crates/sheprd-client/tests/live/plugins.rs` follow the decision.
- Add `scripts/release/tests/test_link_activation_absence.py` next to the other
  `test_*_absence.py` guards.

Rough size: 250–400 lines removed across two crates, no vendored type and no
control-protocol version change. Focused checks would be the `window::links` and
`sheprd-client` suites, `just format-check check-file-size check-sorted
check-i18n`, and the new Python guard.

## Options that need an owner decision

**A — Wait for a read-only link route from the runtime.** The gap's own wording:
a bounded, typed link target and span for an exact terminal identity, revision
and viewport cell, with stale and no-link results and no handler execution, from
Herdr 1 and later Core. That is a runtime change outside this repository. Cost:
T04 and T10 stay blocked, the release build keeps the last plugin-executable
path, and only stage 0 is mergeable meanwhile.

**B — Remove activation and accept single-row links (explicit scope change).**
What a user loses: a plain-text web address that soft-wraps onto the next row
stops being clickable, because the only reader of that URL is the activation
reply. What stays: OSC 8 hyperlinks, wrapped ones included, still open once on
one click; single-row plain addresses, files with their disk check, `mailto:`,
the unknown-scheme confirmation, and rich-chat links are untouched. B also needs
one sub-choice: keep `pane.link.resolve` (the hover underline would then show a
span a click cannot open) or remove both routes and keep
`terminal/links.rs`'s row-local detector alone. This needs a new decision ID —
D71 kept G-LINK in scope — and it restates A33 to record the narrower retained
behaviour instead of claiming full retention.

**C — Keep activation and keep T04 blocked (status quo).** Nothing lands and the
gap stays open; it is a deliberate deferral, not a resolution.

For what it is worth: if a runtime-side read-only route is not imminent, B with
`pane.link.resolve` removed is the only self-consistent way to close T04's source
obligation this cycle; A remains the better answer if the wrapped-plain-text
click is judged part of the approved experience.

## Not proven

- No build, app, daemon, package build or native interaction was run for this
  note.
- The family table is a source read at HEAD. Like the 2026-11-17 strings scan
  ([`t10-removal-scenarios-20261117.md`](../evidence/t10-removal-scenarios-20261117.md)),
  it can show absence of named routes, not absence of behaviour.
- A28/A29/A30/A33/A34 stay `not_run`; the plugin-activation refusal in particular
  has no failing-then-passing test yet, because writing it before the decision
  would encode the answer.

## Checks run for this note

- `python3 tools/spec.py context T04` from `docs/spaceport/`, for the task
  capsule; it changed nothing.
- `git log`/`git diff` reads for the commits named above, plus the source greps
  behind the family table.
- Package and diff checks are recorded when the note is committed.
