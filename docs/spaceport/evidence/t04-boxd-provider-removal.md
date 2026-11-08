# T04/RM-PROVIDERS: Boxd provider cut

At `0cdb3c3a`, Boxd creation was already absent from Settings and dispatch,
but its account card still read and saved API keys, its saved devices still
connected or deleted through the provider table, Add Device still suggested
Boxd from config or environment, and the app could run its exec relay helper.
This cut removes those routes, the provider implementation and feature flags,
its icon, error messages, example config and current user docs. Other cloud
providers keep the shared credential store and exec socket. The separate Core
machine catalog and Add Device gate stay in place. Generic `host` control
targeting is unchanged; without a Boxd endpoint in the catalog it cannot
select one.

The saved cloud catalog's `Foreign` entry path keeps providers this build
does not know. A focused test now writes an old `provider: "boxd"` device,
requires it to be hidden, adds an active provider's device, and compares the
old entry after that write. This is a structured value comparison; JSON
formatting may change. No saved device file or credential was changed outside
the test's temporary directory.

Old root `[boxd]` settings become unknown and inert. An old `[cloud.boxd]`
table is unsupported by the strict `[cloud]` parser and makes config load
fail with a parse error. A new test checks that startup rejects it without
rewriting the file. No compatibility parser, silent migration, or credential
cleanup is included. A person with that old table must remove it from their
Spaceport config. This remains a startup compatibility limit to verify before
shipping.

The source/asset absence test first failed on 42 live Boxd references and
the icon/locale files; after the cut it passed 2/2. The focused catalog test
passed 1/1 with Boat, Container and Daytona features. The two config tests
passed 2/2 with GUI default features off. The no-default compile found a type
inference gap in the device rail when no cloud feature exists; an explicit
`Option<Div>` fixes it. The GUI linker emitted its existing compact-unwind
warning and Cargo reported the existing `block v0.1.6` future-compatibility
notice. A first isolated GUI build was stopped at 8.8 GiB free, and a later
default-feature GUI rebuild was stopped on disk coordination before finishing.
Default/all-feature GUI and native checks remain unrun for this branch.

The original `--locked` attempt only reported that the lock file needed
regeneration; `cargo metadata --offline` updated it. `just format-check`,
`just check-i18n` (5,032 messages in 264 files), `just check-file-size`,
`just check-sorted`, `git diff --check`, package `spec.py check` with JSON
Schema, and the package's 21 tool tests passed. No native UI, personal
provider, Core service or installed app was started.
T04 remains blocked on G-LINK, and A30 remains `not_run` until all excluded
cloud and other runtime routes are absent and native/process checks pass.
