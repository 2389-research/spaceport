# Core experiment guidance

Date: 2026-11-17. Scope: `jemh` under `9s44`.

Settings now explains that Herdr Core is a separate experimental runtime.
The opt-in description states the restart requirement and the effect of disabling it.
It explains how to show hidden Core tabs again without claiming their terminals stopped.
The Tabs setting names new local tabs rather than every new terminal.
Its description identifies New Tab, separation from Herdr TUI, existing tabs, and Core-group behavior.
The [Herdr client guide](../herdr-client.md) explains both settings and distinguishes Core groups from window splits.

This is a text-only change.
Decision D69's default-off, restart-only behavior remains authoritative.
No runtime, startup, visibility, or saved-reference code changed for this finding.
No new tests were added for copy alone.

Direct review traced `new_tabs_use_core`, the Core opt-in gate, and the Settings controls.
The routing uses Core for an existing Core group regardless of the new-local-tab preference.
Legacy SSH endpoints keep their host's Herdr route.
The existing settings tests cover pending enable/disable and restart behavior.
The retained-view tests cover preserving Core references while the feature is disabled.
Those existing contracts support the text; this copy change did not rerun the native suites.

`just check-i18n` passed all 14 checker tests and validated 2143 used messages.
`git diff --check` passed.
The row renderer wraps its description without a line clamp.
## Native qualification on November 18

The isolated native local-controls fixture passes both themes and 680/960-pixel Core layouts.
It verifies private preference writes, the restart boundary, and disabled Core fallback.
Root inspected `core-light-680-pending.png`; the full opt-in and recovery text wraps without clipping.
The capture is under `/tmp/spaceport-9s44-unlocked-controls.fnKDHa/`.
The passing driver log is `/tmp/spaceport-9s44-unlocked-local-controls.log`.

All three Core-enabled catalog row tests pass against the latest integration-feature unit executable.
They draw every available row and compare titles and descriptions with the search catalog.
The log is `/tmp/spaceport-9s44-core-copy-catalog.log`.
Current formatting, size, sorting, i18n, and diff checks pass.
Independent review found no remaining copy contract or material layout defect.

The default-off native fixture does not draw the hidden Core Tabs row.
Its native glyphs and screen-reader speech remain unqualified.
Those broader checks are not acceptance requirements for this copy-only finding.
The later sidebar rerun failed at Host decoy activation after its Settings/Core checks passed.
No whole-driver pass is claimed for that run.
