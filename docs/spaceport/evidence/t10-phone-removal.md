# T10 phone service removal: bounded progress

Source baseline `73fbabc1` still starts `remote::Supervisor` at normal GUI startup, exposes phone pairing in menus, settings, commands and the private CLI, and packages `sheprd-remote-control` plus `sheprd-nearby`. A source inventory found about 100 tracked application and release files that name the phone service or its helper. The phone crate also supplies pure tag rules used by desktop grouping. No personal service, credential, Core state or user file was read or changed.

## Local tags extracted

The first cut keeps local tag grouping, picker edits and the existing `device-tags.json` record shape (`tags`, `updated_ms`) while moving only the proven normalization and comparison rules into `device_tags/rules.rs`. It removes daemon tag exchange, SSH execution, per-window reach polling and the phone sync badge. Saved endpoint IDs remain local preferences; they are **not** assigned to Core machines by label or address. Opaque Core machine IDs need an exact identity path before those rows can inherit tags.

The new absence check failed first on the phone crate import, then passed after extraction. `cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd device_tags` passed 13/13 with `CARGO_TARGET_DIR=/tmp/spaceport-t00-target CARGO_INCREMENTAL=0`. The added retained regression reads an old-shaped JSON record, normalizes it, and writes the same shape. The macOS linker reported its existing `__eh_frame` warning; Cargo reported the existing `block v0.1.6` future-compat notice.

## Desktop phone routes removed

The desktop no longer starts a phone supervisor, exposes pairing/revocation menus or settings, accepts `sheprd remote`, or decodes the old phone control request. Protocol version 13 rejects that request. Old `[remote]` settings remain on disk but the parser reports the whole table as unknown and does not act on it. The new source absence check first failed on the old supervisor call, then passed. The focused CLI/config tests passed 2/2, the protocol rejection test passed 1/1, and retained local tag tests passed 13/13. The config test first expected per-key unknown reports; a failing run showed this parser reports an unknown table as `remote`, so the assertion now names that exact result. The only new compile warning found during this cut was a dead `Unit::None` from the removed phone port control; it was removed. The existing macOS `__eh_frame` linker warning and `block v0.1.6` future-compat notice remain.

## Package and service source cut

The workspace no longer builds `sheprd-remote-control` or `sheprd-nearby`.
Their source trees, macOS Swift helper, phone web assets, phone build scripts,
mobile plans/mockups, package entries, CI and Nix builds, installer definitions,
manifest host assets, and phone-only locale messages are removed. Release tests
now assert the retained app, chat companion and current pinned Core packaging
layout without phone binaries. No phone service is started or packaged by the
current source. The removed host manifest is not accepted by the updater; the
existing package, Windows and update manifest paths still have focused tests.

The package absence test first failed on the workspace member, then passed
3/3. The full Python release suite passed 194 tests with 9 skips after stale
phone fixture expectations were removed. `sheprd-updater --lib` passed 135
tests with 3 ignored and one filtered harness. `cargo check -p sheprd-gpui
--locked` passed with only the existing `block v0.1.6` future-compat notice.
`just check-i18n` passed with 5334 messages in 281 files valid and used;
`just check-file-size`, `just check-sorted`, `just format-check`,
`git diff --check`, and the specification package integrity check passed.
No native app, installer, or network inventory was run in this cut.

## Remaining gates

A31 remains open until the native process/package/network inventory and a
retained desktop CLI scenario prove no phone listener while local IPC works.
A54 remains open for its separate native chat and recovery scenario. T10 remains
in progress because its other removals and acceptance items are independent of
this phone cut. The old `docs/rebranding.md` is a historical upstream plan and
still describes phone identities; it is not a Spaceport release contract.

An installed personal phone service from an older build may outlive source removal. This work does not stop or uninstall it: that action could affect real user state. The final release needs an explicit, platform-specific owner-safe cleanup path and evidence that it targets only Spaceport's old service identity. No silent startup cleanup is authorized or implemented here.
