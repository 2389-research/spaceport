# Core-only popup ownership

Date: 2026-11-17. Follow-up within `ydr4` and `kjkw`.

## Cause and behavior

Core-only windows intentionally contain no Herdr TUI endpoints.
Popup and batch capture nevertheless indexed the selected endpoint.
Three runtime regressions reproduced those panics.

An initial optional-endpoint fix removed the panics but blocked Core pane consent and local palette actions.
Independent review identified the shared assumption.
A captured typed popup origin now distinguishes legacy windows, Core-only windows, and client-only contexts.
Legacy windows retain their epoch/generation fence, including while showing Core views.
Endpoint-free Core windows retain their exact Core key and selection epoch.
Client actions check popup freshness; Herdr TUI actions still require legacy authorization.

Core pane consent retains its Store owner, pane membership, and exact backend checks.
Project palette actions retain their explicit local target, generation, and boot checks.
Configured daemon commands still require legacy authorization.
Whole-batch validation and serial close barriers retain their existing protections.

The palette source producer now handles an empty endpoint catalog.
Local sidebar toggles finish without falling into a daemon request or stealing Core pane focus.
Unavailable detach/reconnect and legacy commands refuse instead of indexing an absent endpoint.
Daemon reload skips the absent TUI endpoint; application config reload retains its own route.
CLI connection controls return `Unsupported` for that unavailable target.
This does not invent a Core detach or reconnect API.

## Verification

Runtime red evidence:

- `/tmp/spaceport-9s44-core-popup-red.log`: three capture/open panics.
- `/tmp/spaceport-9s44-core-pane-consent-red.log`: valid Core-only pane consent refused.
- `/tmp/spaceport-9s44-core-popup-final.log`: palette source enumeration indexed an empty catalog.
- `/tmp/spaceport-9s44-core-palette-dispatch-red.log`: local sidebar command fell into legacy dispatch.
- `/tmp/spaceport-9s44-core-only-dispatch-red.log`: reload and connection controls indexed missing endpoints.

The broader check also caught a sidebar focus regression.
The corrected toggle preserves the focused Core pane.
An additional refusal fixture originally omitted the selected Core key.
It now represents the actual Core-only window before exercising palette actions.
The invalid fixture's reconnect-card panic is not claimed as a fixed production scenario.

The final scoped script passed:

- 11 Core-only cases.
- 22 close-modal cases.
- Eight Core-pane close cases, including an actual `terminal.kill` frame.
- 103 palette cases.
- One retry-dialog fence case.
- 31 workspace-menu cases.
- Three Close Others group cases.
- 17 application-control cases.
- GPUI Clippy with all targets, all features, the pinned Sparkle SDK, and `-D warnings`.

Script: `/tmp/spaceport-9s44-core-popup-check.sh`.
Log: `/tmp/spaceport-9s44-core-popup-final-checks-focus.log`.
The expanded same-epoch/different-Core-key refusal case also passed.
Log: `/tmp/spaceport-9s44-core-origin-key-green.log`.
Formatting, file-size, sorted-region, localization, and diff checks passed.
Localization validated 2,158 messages in 114 files.

The independent reviewer inspected the corrected ownership and dispatch paths and the final logs.
No material findings remain in this delta or the captured batch lifecycle.
Existing linker unwind and `block` future-incompatibility notices remain.

## Limits

These checks use headless GPUI and private framed peers.
Native keyboard, visuals, accessibility, and platform qualification remain pending.
Core warning flashes still need the shared all-view feedback placement under `kjkw`.
The broader `ydr4` motion and `kjkw` feedback findings remain open.
