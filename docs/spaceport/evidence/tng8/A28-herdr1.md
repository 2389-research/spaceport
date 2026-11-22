# A28 external-only browsing — headless half (partial, Herdr 1 leg)

Session `H`, commands `H8` and `just audit-deps`. This is a partial attestation:
A28's pass rule also needs a native open in the macOS session (`M·M8`), which no
session has run. The Herdr 1 leg stays `not_run`.

## What ran

| Field | Value |
| --- | --- |
| Revision | `708ca09f3a2d7a61c3b56e7d9bd3537ed9125c52` (`feat/wx3-session-h`, tree clean; only `target/` was built) |
| Commands | `cargo test --locked -p sheprd-gpui --test cli`; `cargo test --locked -p sheprd-gpui --test removed_updater`; `just audit-deps` (`cargo deny check`) |
| Machine | macOS 26.7.1, arm64 |
| Runtime | none: these are CLI-parsing and dependency-graph checks, no daemon is contacted |
| Logs | [`logs/H8-cli-removed-updater.log`](logs/H8-cli-removed-updater.log), [`logs/A28-audit-deps.log`](logs/A28-audit-deps.log) |

## Result

`--test cli`: **21 passed, 0 failed**, including every removed command A28 names.

```
test browser_commands_are_rejected_before_socket_lookup ... ok
test code_open_is_rejected_before_socket_lookup ... ok
test code_edit_is_rejected_before_socket_lookup ... ok
test removed_design_feedback_is_rejected_before_socket_lookup ... ok
```

`--test removed_updater`: **2 passed, 0 failed**
(`removed_updater::updater_commands_and_helpers_exit_before_app_startup ... ok`).
The same suite also covers the neighbouring removals
(`removed_project_open::removed_folder_open_exits_before_starting_the_app`,
`removed_scripts::removed_script_commands_exit_before_starting_the_app`).

`just audit-deps` passed: `advisories ok, bans ok, licenses ok, sources ok`.

## Partial claims this record re-attests

The Herdr 1 leg already carries five `t04-*` removal records (visual annotation,
mockup feedback, integrated route cuts, browser control family, browser tab
protocol). This session does not re-derive those removals; it retests the
behaviour they imply — that the removed commands exit **before** any socket
lookup — and re-runs the dependency audit that the leg's previous record cites.

## What this record does not prove

A28's pass rule also requires that "a URL opens the system browser (not an
in-app view)", which is a native step (`M·M8`) and is not covered here: no
window was opened, so no URL was opened. The source-tree half of the audit is
the `t04-native-browser-engine-removal.md` record, not this log. The leg
therefore stays `not_run`.
