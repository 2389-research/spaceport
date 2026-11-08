# T02 explicit Core machine connect

The pinned Core `0.1.18` runtime advertises `machine.connect` with one `machine` name parameter and a `MachineView` result. It dials a saved machine now and returns Core's own state and reason. Spaceport added an optional typed call guarded by `hello.methods`; it does not create a client reconnect loop. The exact request-shape test failed to compile before the call existed, then passed. A separate test confirms an absent advertised method returns `Unsupported` before sending a request.

The private two-Core fixture added `machine.connect("lab")` after the remote fixture reached connected state. It returned the saved remote Core ID and `connected`. `just test-core` passed all three opt-in real-Core tests; the Core client library suite passed 40 tests with three live tests ignored in its default run. `cargo +1.99.0 fmt --all -- --check` and `just check-file-size` passed.

This is a Core route and private fixture result. It does not prove real SSH authentication, host-key refusal, Tailscale routing, GUI presentation or Windows/Linux behavior. T02 and A04/A05 remain open.
