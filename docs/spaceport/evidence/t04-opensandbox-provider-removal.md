# T04/RM-PROVIDERS: OpenSandbox provider cut

At `1d9f58c7`, OpenSandbox was a default cloud feature with an account/API-key
Settings card, REST create/resume/delete calls, an execd relay helper dispatched
by the desktop binary, a provider registry row, Add Device rows, an icon,
locale errors, and current docs and website claims. This cut removes that
provider leaf and its live claims. `design/sheprd-website/` is an inherited
design archive and remains unchanged. The live website is upstream Sheprd
marketing, not Spaceport release proof; it was not deployed.

The existing saved-device catalog treats an old `provider: "opensandbox"` row
as foreign data. A mixed-row test failed while the provider was registered;
after removal it passes: the old row is hidden, an active Container row stays
visible, and a catalog write retains the old row's structured JSON value and
extra field. OpenSandbox had no image-provider entry. The generic image
catalog, `cloud/socket.rs` used by Sprites, shared relay/credentials, manual
SSH, Core machine onboarding, and other providers remain. No disabled
OpenSandbox tile or helper remains.

An old root `[opensandbox]` table is unknown and inert. The strict `[cloud]`
parser rejects `[cloud.opensandbox]`; a private-file test checks that failure
leaves the file untouched. There is no compatibility parser or personal
credential cleanup. A user with the old setup table must remove it before
these settings load; release review still needs to cover this.

Red checks: source absence failed on 37 live paths/asset checks; live website
copy failed in two files; the mixed saved-device test failed against the old
registered provider. An `opensandbox`-only Rust test build could not compile
pre-existing image tests that assume the Container variant; adding the
Container feature reached the intended behavior failure. Green checks:
source absence 2/2; website 22/22; Container catalog 10/10; i18n valid with
4,663 used messages in 252 files after removing seven now-unused shared card
messages. No-default and default GUI retired-config tests passed 2/2 each;
default cloud-setup tests passed 8/8. Default and all-targets/all-features
GUI checks, formatting, file-size and sorted-list checks, JavaScript syntax,
diff checks, the specification package validator and its 21 tool tests passed.
The package validator checks schema and links, not application behavior. The
GUI links emitted the existing macOS `__eh_frame` warning; Cargo repeated the
existing `block v0.1.6` future-compatibility notice.

Native GUI and real provider network/service behavior were not run. T04 stays
blocked on G-LINK while other excluded routes remain. A30 stays `not_run`.

## Integrated branch verification

After cherry-picking this cut onto `wip/spaceport-t00` at `33600841`, the
default and all-feature `cargo +1.99.0 check --locked --all-targets -p
sheprd-gpui` commands passed. The targeted `retired_opensandbox` catalog test
passed 1/1. Source absence passed 2/2, the combined site suite passed 28/28,
and `just format-check`, `just check-file-size`, `just check-sorted`, and
`just check-i18n` passed. The package report was regenerated and its 21 tool
tests passed. Cargo reported the existing `block v0.1.6` future compatibility
notice. These checks do not cover native windows or provider network behavior.
