# T02 Core capability boundary evidence

T02 is in progress. A04 and A05 are not passed.

The pinned Core source `cd12ddd88bc11490228343291dc1096c9ed3bf77` and its live `rpc.discover` response both define `machine.list` with no parameters and a `MachineListResult` containing `MachineView` records. Spaceport's Core client now reads that result through a typed call. It retains Core's state and refusal words, including unknown future states, rather than creating a second machine connection policy. Before dispatch, the client checks `hello.methods` and returns a typed unsupported error when a method is absent.

The scripted-peer test failed to compile before the method and typed result existed, then both request/result and unsupported cases passed. The isolated Core integration test ran against the pinned `0.1.18` binary with a private home and socket; `machine.list` returned an empty catalog. This proves the local read route only. It does not exercise remote SSH/Tailscale, authentication, host-key refusal, reconnect, a second client, or terminal aggregation.

`RUSTUP_TOOLCHAIN=1.99.0 CARGO_TARGET_DIR=/tmp/spaceport-t00-target just test-core /tmp/spaceport-t00-target/debug/herdr-core` passed: 2 real-Core tests. `cargo +1.99.0 test --locked -p sheprd-core-client --lib` passed: 36 tests, 2 ignored. `cargo +1.99.0 fmt --all -- --check`, `just check-file-size`, and the package's `spec.py check --write-report` and 21 tool tests passed on macOS.
