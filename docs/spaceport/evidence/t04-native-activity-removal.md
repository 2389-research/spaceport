# Native Activity PR, Issues, and Tasks removal

Base: integration `749d3b4ff`. This cut removes the remaining native
Orchestrator route as one vertical unit: PR/Issues/Tasks activity panels and
counts, their tick polling and view state, the control/CLI panel names, GitLab
account UI and config, smart-dispatch history loading, Orchestrator menus and
pages, exclusive locale strings, docs, icons, screenshots and tests. The
control protocol rises from 28 to 29 because accepted UI operations change.
An old `ui.panel` request for `pull-requests`, `issues`, or `tasks` fails
strict JSON decoding and CLI parsing instead of selecting another panel.

The regression for Activity reachability failed before the cut (its old
`activity-pull-requests` selector remained visible), then passed 1/1. The
protocol regression likewise failed before the cut (`pull-requests` still
parsed) and the final protocol suite passed 60/60. The settings window test
first failed because it queried the removed Orchestrator-only
`activity-section` selector; it now queries the retained Overview selector
and passed 1/1. The CLI refusal test passed 2/2.

Retained headless filters passed: Activity 16/16, saved control tabs 9/9,
control panel 2/2, preferences 18/18, GitHub menu 26/26, agent Overview
83/83, device Overview 30/30, Core terminals 33/33, Usage 102/102, avatars
38/38 (two opt-in tests ignored), browser store 20/20. Generic credentials
passed 3/3. `cargo +1.99.0 check -p sheprd-gpui --all-targets --locked
--offline` passed with no crate warnings. The macOS test link emitted its
existing compact-unwind linker warning; Rust noted future compatibility for
the existing `block` dependency. These headless filters do not prove native
window behavior or Core's missing device-activity feed.

The saved `orchestrator` tab JSON still decodes under a retired type with
strict path validation. Loading reserves its ID, drops the retired tab, and
keeps Core group and page tabs. Earlier `pull-requests`/`issues`/`tasks`
activity preferences fall back to Workspaces. No saved user file is deleted
or migrated. Startup no longer reads or writes `dispatch-history.json`; an
older file stays untouched. The retained generic credential store, avatar
cache, private-file helpers, Overview agent rows, device history/chart,
Usage, diagnostics, independent file actions, and Core group/page identity
remain. GitLab avatar URLs are rejected by the retained avatar allowlist;
hostile GitLab URL fixtures remain as security tests.

`just format-check`, `just check-file-size`, `just check-sorted`, and
`git diff --check` passed. The package `tools/spec.py check` and
`tools/test_spec.py` passed; they check package integrity, not app behavior.
`just check-i18n` remains red at the integration baseline's sole unused
`browser-no-pages` key in `locales/en-US/browser.ftl:5`; this cut added no
other missing or unused key reports.

Source search across GPUI/control/commands found no live
`poll_orchestrators`, `OrchestratorView`, `GitLabRead`, `dispatch_history`,
`sheprd_orchestrator`, retired Activity enum variants, or GitLab menu/config
references. Remaining `orchestrator` and `gitlab` strings in those sources
are retired-tab and preference decoding, negative tests, and historical
protocol comments. `sheprd-orchestrator` stays a workspace member pending a
separate crate ownership audit; its GPUI dependency and matching lockfile
edge alone were removed. Separate Git status/PR actions remain reachable and
are excluded under T04. T04 stays `blocked` by G-LINK and A29/A30 stay
`not_run`; there was no native Mac window run because the machine was at
loginwindow.

Integrated as `d245f3b4d` with the backend doc-pointer fix `ee7dbcffe`.
On the combined branch, `cargo +1.99.0 test -p sheprd-control-protocol
--locked --offline --quiet` passed 60 tests, and `cargo +1.99.0 check -p
sheprd-gpui --all-targets --locked --offline --quiet` passed. Package
integrity and all 21 package tool tests passed. Format, file-size,
sorted-list, commit-message, and diff checks passed. The source reachability
sweep found no live Orchestrator worker, GitLab account, or removed Activity
panel enum reference in the desktop/control/command crates. Native UI and
release/package checks remain unrun.
