# T04: default Herdr 1 bridge dependency

Source baseline: Spaceport `6e61e119`. This is a read-only dependency map, not
an application scenario. No daemon, Core process, user socket, or personal
configuration was opened or changed.

## Current default route

- `crates/sheprd-gpui/src/cli.rs:156–245` selects `ConnectTarget::Local` for a
  plain launch. `app.rs:553–571` opens the initial window on that target and
  repeats it for restored windows. `window/construct.rs:189–194` creates the
  first `Endpoint` with a `ConnectionBridge` (`endpoint.rs:226–232`).
- `endpoint/polling.rs:48–185` connects and retries endpoints. The bridge in
  `connection.rs:152–238` calls `daemon::connect` off the UI thread; its event
  reader updates `LiveState`. `daemon.rs:40–72,257–322` dials the Herdr 1
  socket and, for a local missing/refused socket, starts `herdr server` and
  waits for it. Only executable-not-found is tagged `MissingInstallation`.
- `state.rs:80–82` holds the Herdr 1 `ClientShellSnapshot`. This is still a
  working input to `sidebar/rail.rs:259–271` and `sidebar/render.rs:213–269`
  (workspaces), `window/pending_input.rs:334` and `window/selection.rs:309–379`
  (input/selection), and `window/system_notifications.rs:304` (notifications).
  These are representative dependencies, not the full graph.
- `control/server/app.rs:195–252` still derives a new window's target from an
  exact `--window`, the caller daemon socket, or the frontmost/default target.
  The previous cut removed its named-session override, but a plain new window
  can still inherit `ConnectTarget::Local` or an existing Herdr 1 socket.

## Core route and cut decision

`core_terminals/hub.rs:92–157` connects to a separately installed Core on a
worker; `core_terminals/worker.rs:83` owns terminal attachments. The app-wide
`core_terminals/observation.rs` projects Core terminal, machine and Spaceport
workspace-document facts. `core_terminals/workspace_ui.rs` reads that document
and edits it with a version/epoch guard. This is separate from the default
`LiveState` snapshot. `core_terminals/fallback.rs:27–115` shows one named Core
shell only after `LiveState.missing_installation`; it is not the primary
workspace, chat, notification and input projection.

The default bridge cut is **blocked as an internal UI dependency**. Simply
removing `daemon::connect` or its auto-start would leave the default endpoint
without the snapshot those retained views consume. It would not trigger the
existing Core fallback for an ordinary missing socket, because that fallback
keys on executable-not-found. Reclassifying the failure as a fallback would
silently switch the user's default workspace/agent surface and would not prove
equivalent behavior. A Core-first window must render and route input,
workspaces, notifications and exact window targeting from the Core projection
before retiring the legacy endpoint. T05's private Core client test proves
terminal survival across detach, not a native window replacement.

`chat/core_pane.rs:32–62` finds only local Core adapter labels; remote Core
terminal addresses are filtered. `chat/chats.rs:60–78` still uses a separate
SSH route for remote structured chat. This part depends on external G-CHAT,
while local GUI/adapter parity needs its own native proof. Neither gap justifies
silently keeping a second transport owner or deleting rich chat. At this
source baseline, the Herdr 1 install offer (`menu/chrome.rs:20–26`,
`window.rs:716–721`) was another bounded UI removal unit. That offer and its
menu, keyboard, locale and test references were removed at `a13aecf3`, with
truthful Core-unavailable guidance; see
[t04-legacy-install-offer.md](t04-legacy-install-offer.md).

No application code changed in this slice. T04 and RM-LEGACY remain
`in_progress`; A32 remains `not_run`. This map does not claim full Core
terminal, chat or workspace parity or the absence of legacy runtime behavior.
