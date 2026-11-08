# T10 phone service removal: bounded progress

Source baseline `73fbabc1` still starts `remote::Supervisor` at normal GUI startup, exposes phone pairing in menus, settings, commands and the private CLI, and packages `sheprd-remote-control` plus `sheprd-nearby`. A source inventory found about 100 tracked application and release files that name the phone service or its helper. The phone crate also supplies pure tag rules used by desktop grouping. No personal service, credential, Core state or user file was read or changed.

## Local tags extracted

The first cut keeps local tag grouping, picker edits and the existing `device-tags.json` record shape (`tags`, `updated_ms`) while moving only the proven normalization and comparison rules into `device_tags/rules.rs`. It removes daemon tag exchange, SSH execution, per-window reach polling and the phone sync badge. Saved endpoint IDs remain local preferences; they are **not** assigned to Core machines by label or address. Opaque Core machine IDs need an exact identity path before those rows can inherit tags.

The new absence check failed first on the phone crate import, then passed after extraction. `cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd device_tags` passed 13/13 with `CARGO_TARGET_DIR=/tmp/spaceport-t00-target CARGO_INCREMENTAL=0`. The added retained regression reads an old-shaped JSON record, normalizes it, and writes the same shape. The macOS linker reported its existing `__eh_frame` warning; Cargo reported the existing `block v0.1.6` future-compat notice.

## Remaining gates

The desktop supervisor, phone menus/settings/CLI, phone crate, nearby helpers and packaging/service routes still need removal. This intermediate source cut does not satisfy A31, A54, or RM-PHONE. T10 is in progress. A fresh native app/package/network inventory and retained local CLI scenario remain necessary.

An installed personal phone service from an older build may outlive source removal. This work does not stop or uninstall it: that action could affect real user state. The final release needs an explicit, platform-specific owner-safe cleanup path and evidence that it targets only Spaceport's old service identity. No silent startup cleanup is authorized or implemented here.
