# T04/RM-PROVIDERS: Railway provider cut

At `33600841`, Railway was a default cloud feature. Its key setup could run
`ssh-keygen`, its VM creation and bridge could run `ssh railway.new`, and the
desktop offered Settings and Add Device routes to those jobs. The provider
also had a registry row, icon, locale errors, current README text and two
live website claims. This cut removes those routes and claims. The inherited
`design/sheprd-website/` archive remains unchanged; the live site was not
deployed.

The shared saved-device catalog keeps an old `provider: "railway"` row as
foreign data. A mixed-row test failed while Railway was registered, then
passed after removal: the old row is hidden, an active Container row stays
visible, and a write retains the old row's structured JSON value and extra
field. The strict `[cloud]` parser rejects `[cloud.railway]` without changing
the user's file. No compatibility parser, saved-key deletion, provider
endpoint call or personal state change was made.

The generic SSH target and host-key handling, Core machine onboarding,
shared cloud install code used by other providers, other provider rows, and
the generic saved-device and image catalogs remain. The Railway-only job
step, headless reach helper and settings-job query were removed with their
last callers. No disabled Railway tile or helper remains.

Red checks: source absence failed 32 live source/asset checks; live website
copy failed in two files; the mixed saved-device test failed with
`Invalid(Catalog)` while Railway was registered. Green checks: source
absence 2/2; website 23/23; Container catalog 11/11; no-default GUI retired
config 1/1 and retained cloud setup 6/6; default and
all-targets/all-features GUI checks; i18n 4,619 messages in 250 files;
formatting, file-size, sorted-list, JavaScript syntax
and diff checks; specification package validator and its 21 tool tests. The
validator checks schema and links, not application behavior. The GUI test
link emitted the existing macOS `__eh_frame` warning; Cargo repeated the
existing `block v0.1.6` future-compatibility notice.

Native GUI and real provider network/service behavior were not run. T04
stays blocked on G-LINK while other excluded routes remain. A30 stays
`not_run`.
