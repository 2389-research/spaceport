# T04/RM-PROVIDERS: Boat provider cut

At `b499b112`, Boat was a default cloud feature with a provider registry row,
API-key account settings, a persisted creation journal, create/recover/delete
and SSH gateway code, a Settings card, Add Device rows, localized errors, and
current desktop documentation. This cut removes those routes, the Boat feature
and modules, the two live upstream website claims, and their locale strings.
The website is inherited Sheprd marketing, not Spaceport release proof; it was
not deployed. `design/sheprd-website/` is an inherited design archive and was
not edited.

The existing catalog treats an old `provider: "boat"` saved-device row as
foreign data. A mixed-row test passes after removal: the old row is hidden,
the active Container row remains visible, and writing the active row retains
the old row's structured JSON value
including its extra field. Boat had no image-provider entry; the generic image
catalog is unchanged. The shared SSH `connect_command`, generic credentials,
Core machine Add Device path, manual SSH targets, and other provider rows stay.
There is no Boat operation behind a disabled tile.

An old root `[boat]` table is now unknown and inert. The strict `[cloud]`
parser rejects `[cloud.boat]`; a private-file test proves a failed load leaves
the file unchanged. There is no compatibility parser, migration, personal
credential cleanup, provider network call, or user service action. A user with
that old setup table must remove it before these settings load; release review
still needs to cover this.

Red source absence failed on 38 production paths/locale checks. Green source
absence passed 2/2; remote catalog passed 9/9 with Container enabled; the
no-default GUI retired-config tests passed 2/2; default-feature cloud setup
passed 8/8; default-feature retired-config tests passed 2/2; website tests
passed 21/21. Default and all-targets/all-features GUI checks passed. Source
format, file-size, sorted-list, JavaScript syntax and diff checks passed;
the specification package validator and its 21 tool tests passed. The package
validator checks schema and links, not application behavior. `just check-i18n`
passed with 4,703 used messages in 254 files after removing five now-unused
shared card messages.

The no-default GUI link emitted the pre-existing macOS `__eh_frame` warning;
Cargo repeated the existing `block v0.1.6` future-compatibility notice.

Native GUI and real provider network/service behavior were not run. T04 stays
blocked on G-LINK while other excluded routes remain. A30 stays `not_run`.
