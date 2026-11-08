# T04/RM-PROVIDERS: Coder provider cut

At `b7e6db6e`, the Coder provider still had an account and OAuth settings
card, an Add Device workspace picker, a cloud provider registry entry for
connect/delete/creation, a remote API and `coder ssh` bridge, and a special
`.coder` sidebar mark. This cut removes those routes, the Coder feature and
provider modules, its icon, locale messages, current user docs, and the two
live upstream website claims. The site is Sheprd marketing, not proof of a
Spaceport release; it was not deployed. CodeRabbit usage is a separate feature
and remains.

The existing saved-device and sandbox-image catalogs now treat old
`provider: "coder"` rows as foreign data. Mixed-row tests first failed while
Coder remained registered: the old row appeared beside an active provider.
After removal, both tests pass. The typed catalogs hide the foreign rows and
keep their structured JSON values when an active device or Container image is
written. The catalogs do not enable Coder operations. Manual SSH host names
ending in `.coder` remain ordinary SSH targets; their host-key choice and the
shared `connect_command` bridge stay for retained routes.

An old root `[coder]` table is now unknown and inert. The strict `[cloud]`
parser rejects `[cloud.coder]`; a private-file test checks that failure does
not rewrite the file. No compatibility parser, migration, credential removal,
personal file read, or service action is in this cut. Users with that obsolete
setup table need to remove it before the app can load their settings; this is
an open release review item.

Red phase: source absence failed on 56 production paths, the live website
check failed in two files, remote catalog tests failed 2/2, and no-default
GUI config tests failed 2/2. Green phase: source absence 2/2, website 20/20,
remote catalog 2/2 with Boat and Container features, no-default GUI config
2/2, default-feature endpoint catalog 1/1, and ordinary SSH/Tailscale origin
1/1. `cargo check --all-targets --all-features -p sheprd-gpui`,
`just check-i18n` (4,873 messages in 258 files), format, file-size, sorted
regions, `node --check` on the edited site script, and diff checks passed.
The no-default GUI test emitted the same linker and dependency warnings as
its red baseline. Native UI and actual provider network/service behavior were
not run. T04 remains blocked on G-LINK, and A30 remains `not_run` while other
excluded provider and runtime routes remain.
