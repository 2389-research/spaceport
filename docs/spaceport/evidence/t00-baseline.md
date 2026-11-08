# T00 source and runtime baseline

This is evidence for the **reference**, not Spaceport application parity. The
application checkout was `ed14ccbccf1e730df7ca5b9bee3e7c5aeac9301f` on
`wip/spaceport-t00`, branched from `main`. The package's Sheprd source pin
`806b49870ac7761b98cbcea4bd115aa2f2b00d75` was fetched into a separate,
clean reference checkout and built and launched there. The Core reference was
`cd12ddd88bc11490228343291dc1096c9ed3bf77`, with its recorded Ghostty
submodule `1e5cdb865807abc990181efb8ebfa9122955cce2`. `git status` in that
reference stayed clean. No Herdr source was changed.

The host was macOS 27.2 on Apple Silicon. The repository pins Rust 1.99.0.
An inherited Rust 1.97.1 override caused the first `just test-core` attempt to
refuse compilation; the rerun used `RUSTUP_TOOLCHAIN=1.99.0`. `hc` was not
installed in `PATH`; the test binary was built from the pinned Core source into
`/tmp/spaceport-t00-target`, never installed. The original checkout retained
its two untracked spec input files throughout the worktree setup.

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo +1.99.0 metadata --locked --format-version 1 --no-deps` | 27 crates in the fork checkout, 30 in pinned upstream; direct dependencies and features inventoried | [fork inventory](t00-dependencies.json), [upstream inventory](t00-upstream-dependencies.json) |
| `cargo +1.99.0 build --locked -p sheprd-gpui --bin sheprd` | Passed in both checkouts; linker compact-unwind warning and `block` future-compatibility notice were already present | [fork build](t00-app-build.log), [upstream build](t00-upstream-build.log) |
| `cargo +1.99.0 build --locked -p herdr-core` | Passed after populating the pinned submodule | [Core build log](t00-core-build.log) |
| `just test-core /tmp/spaceport-t00-target/debug/herdr-core` with pinned toolchain | 2 passed: real stream/store and backend-trait tests, private Core homes/sockets | [Core test log](t00-core-test.log) |
| `just test-sidebar` with pinned toolchain | Native windows launched in both checkouts; each had 2 fixture tests pass and `native_sidebar` fail: child label width 127px vs 179px | [fork native log](t00-native-sidebar.log), [upstream native log](t00-upstream-native-sidebar.log) |
| Spec `check --write-report`, `test_spec.py`, `ready`, `context T00` | Package validation passed, 21 tool tests passed; application scenarios were not checked by these tools | [validation](validation.json) |

The native sidebar failure reproduces at the **unmodified pinned upstream**
revision as well as the fork checkout. The failing probe and other native
output are preserved in both logs. It must not be attributed to Spaceport
code, which had not changed. No Windows or Linux native run occurred.

The fixture's `hello` returned Core `0.1.18`, wire generation `1`, seven
features (`framed`, `attach`, `events`, `status`, `shell`,
`resize_notifications`, `automatic_input`) and 32 methods. Its live
`rpc.discover` schema matched both `hello.methods` and the pinned source's
`schema/hc.openrpc.json` after canonical JSON sorting; the schema SHA-256 is
`60942ea9cba377e6732051d156b37fd5dc10b4d6b936a33d17437f229b9776ca`.
See [the captured response](t00-core-runtime.json). The capture ran a Core
child with private `HOME`, `HC_SOCKET` and XDG paths, then reaped it. A second
run inherited fake personal-looking `HC_SOCKET` and `XDG_STATE_HOME` values;
those paths stayed absent because the script overrides them for its child.
Relative binary paths were refused before launch. No publish command ran.

| Capability | T00 evidence | Remaining gate |
| --- | --- | --- |
| Core handshake, methods, documents, local terminal stream | Runtime advertised; two private integration tests passed | Version range and cross-platform behavior remain unproved |
| Core machine catalog/remote aggregation | `machine.*` and `events.subscribe` advertised | No remote fixture exercised; no second client host manager should be built |
| Workspace grouping | `doc.get`/`doc.set` and CAS exercised as primitives | No Spaceport document schema or UI projection exists yet |
| Native chat | No `chat.*` method; existing host adapter is separate | **G-CHAT:** prove discovery, identity, authentication, permissions and a Core-owned remote route |
| Service forwarding | No forwarding RPC in the 32-method table | **G-PORT:** Core-owned loopback forwarding contract with target identity, lifetime and end-to-end test; do not expose the service publicly |
| General transfer | `terminal.put`, text `file.*`, and `machine.exec` advertised | **G-FILE:** binary download, overwrite, cancellation and safety parity unproved |
| Usage and automation visibility | No complete Core feed observed | **G-OBS:** identify an authoritative source without provider probes or a scheduler |
| Desktop platforms | macOS reference launch attempted; one layout test failed | **G-PLATFORM:** Windows, Linux and final Spaceport native tests remain open |

The current GUI still imports excluded crates, including VCS, GitHub,
orchestration, remote control, mods, automation and remote provisioning. The
inventory is a deletion map input, not proof that any feature has been removed.
