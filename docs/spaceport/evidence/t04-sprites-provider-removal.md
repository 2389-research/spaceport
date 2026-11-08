# T04/RM-PROVIDERS: Sprites provider cut

At `40f6423b`, Sprites was a default cloud feature. Its API token and REST
jobs created and deleted machines; an internal `__sprites-exec` route relayed
their exec WebSocket into the Herdr bridge. Settings, Add Device, the cloud
registry, worktree dispatch and live docs/site all offered it. This cut
removes those routes, their config, credential use, socket helper, locale
messages and icon. The live website is upstream Sheprd marketing, not
Spaceport release proof; it was not deployed. The inherited
`design/sheprd-website/` archive remains unchanged.

The saved-device catalog keeps an old `provider: "sprites"` row as foreign
data. A mixed-row test failed while Sprites was registered, then passed
after removal: the old row is hidden, an active Container row stays visible,
and a write retains the old row's structured JSON value and extra field.
The root `[sprites]` table is unknown and inert; the strict `[cloud]` parser
refuses `[cloud.sprites]` without rewriting the file. There is no compatibility
parser or personal token/device cleanup. A user with the old cloud setup
table must remove it before those settings load; release review still needs
to cover this.

Azure's shared relay and credential store remain because its sandbox code
still uses them. `cloud/socket.rs` was Sprites-only and is gone. Core machine
onboarding, manual SSH, the generic saved-device/image catalogs and the
Azure and Container provider routes remain. The retained GUI image test now
uses Azure, which has no project images, and Container, which does. The
unrelated website mock task named "Sprite sheet export" remains.

Red checks: source absence failed 38 live source/asset checks; live website
copy failed in two files; the mixed catalog test failed with
`Invalid(Catalog)` while Sprites was registered. Green checks: source
absence 2/2; website 24/24; Container catalog 12/12; default GUI retired
config 2/2 and retained image picker 1/1; default and all-targets/all-features
GUI checks; i18n 4,576 messages in 248 files; formatting, file-size,
sorted-list, JavaScript syntax and diff checks; specification package
validator and its 21 tool tests. The validator checks schema and links, not
application behavior. The GUI link emitted the existing macOS `__eh_frame`
warning; Cargo repeated the existing `block v0.1.6` future-compatibility
notice. An initial default check found a dead helper left by removal; it was
removed, and both final checks emitted no new code warnings.

Native GUI and real provider network/service behavior were not run. T04
stays blocked on G-LINK while other excluded routes remain. A30 stays
`not_run`.
