# T04/RM-MODS: legacy plugin Settings page removal

At `c6f7aea7`, Settings still offered a Plugins page built from Herdr 1
snapshot metadata. Its switches wrote custom rows into shared Herdr settings,
and its preview and native smoke fixture used the same report model. This was
excluded plugin behavior. The ordinary Sidebar layout parser and row styles
are retained for the existing customization controls.

The new Settings catalog test first failed because the Plugins section was
still in `Section::ALL`. The page, catalog row, search, preview, mockup
capture, English messages, pure report model and sole shared-config token
writer are now removed. The test passes and also checks that Integrations
remains listed. The writer's unused preference and helper types were removed
after the all-feature check exposed four new unused-code warnings. The
retained Sidebar parser tests now check typed default rows, custom styles and
per-agent overrides directly; no saved user config was read or changed.

On macOS, default and all-feature app checks passed without new code warnings.
The focused refusal tests passed 2/2, including a headless navigation and
search check; Settings search passed 9/9, navigation
groups 2/2, order 1/1, pseudo-locale pages 11/11, Sidebar parser 2/2, and
shared Settings tests 41/41. `just check-file-size`, `just check-sorted`,
and `just check-i18n` passed (5,141 used messages in 268 files). The first
format check found two line wraps; `cargo fmt` corrected them. The test link
reported the existing macOS compact-unwind warning, and Cargo reported the
existing `block v0.1.6` future-compatibility notice. Native GUI, Linux,
Windows and full workspace CI were not run.

The Herdr 1 snapshot, dynamic plugin actions in the palette/key paths, and
other legacy behavior remain reachable. T04/RM-MODS and A30 stay open; this
slice does not prove complete plugin or runtime removal.
