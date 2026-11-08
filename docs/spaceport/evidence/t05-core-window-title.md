# T05 Core-selected window title

Source base: integration `d41064c8`. The plain-launch window still has a
Herdr 1 endpoint, but a selected local Core group now supplies the OS window
title from the app-global fresh `spaceport` presentation document. The title
reader checks the selected key against the current hub ID, exact document
name and group UUID through `core_group_available`. A stale document or
changed hub cannot supply its old label. A window without a selected Core
group keeps the existing legacy daemon title and focused-workspace rule.
Document labels are validated at 200 Unicode characters by
`workspace_document.rs`; the tick reads memory only and adds no Core RPC.

The headless regression first failed with `Legacy agent title` instead of
`Spaceport — North` while a Core group was selected. It now covers two Core
groups with distinct labels, stale observation, changed hub identity and
return to the legacy title. The retained legacy focused-space test passes.
The focused Core test passed 1/1 and legacy test passed 1/1 under
`cargo +1.99.0 test --locked -p sheprd-gpui --bin spaceport` with
`CARGO_INCREMENTAL=0` and the shared warm target used exclusively for these
runs. Default and all-feature
`cargo +1.99.0 check --locked -p sheprd-gpui --bin spaceport` passed before
the final import alias; the final-source focused test compiled and passed. The
existing macOS compact-unwind linker warning and `block v0.1.6` future
compatibility notice appeared; there were no new source warnings.

This is one retained reader, not Core-first window construction or parity.
The legacy endpoint still owns other window state and `window list` still
reports its legacy workspace/hosts. No native title check, Core process,
Windows/Linux check or full app scenario ran. T05 stays `in_progress`; A09
and A33 remain `not_run`.
