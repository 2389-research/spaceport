# T02 Core-owned machine add contract

T02 remains in progress. A04 and A05 remain not run.

Pinned Core source `cd12ddd88bc11490228343291dc1096c9ed3bf77` defines `machine.add` in `crates/herdr-core/src/api.rs` and `MachineAddParams` in `api/types.rs`. The RPC shape accepts `target`, optional `name` and `session`, and `temporary`; the CLI-only `install` field has `#[serde(skip)]`. The RPC handler calls Core's remote hello before it saves a machine. It returns a typed `Machine` record or a Core error such as `hc_missing`, `incompatible`, or `unreachable`.

The Spaceport client now exposes an SSH-target-only typed `machine_add` call. Its request type has no `install` or `via` field. The scripted-peer test failed to compile before the method existed, then passed with an exact request-shape assertion and parsed Core identity. `MachineView` and the add result share one `Machine` type; the connection state remains a separate Core-supplied field. The method is optional in `hello.methods`, so an unadvertised route is unsupported rather than sent speculatively.

No remote Core was contacted by this test. A refused or timed-out add must not be retried without resolving its outcome from Core; no automatic retry was added. Host-key, password, remote lifecycle, and terminal aggregation scenarios require a private SSH fixture before acceptance.

`cargo +1.99.0 test --locked -p sheprd-core-client --lib` passed 38 tests with 2 ignored. The pinned private-Core `just test-core` rerun passed 2 tests. `cargo +1.99.0 check --locked -p sheprd-gpui --bin sheprd` passed on macOS; its only notice was the existing `block` future-compatibility warning. The Core client remains only a partial capability boundary, and the GUI still has legacy remote paths to remove.
