# Integrated Boxd config regression

After the Boxd provider cut (`1a1130d0` and `927212d0`) and the independent
checkpoint-worker cut (`f11c641c`), the merged GUI test binary ran both
`config::tests::retired_boxd` cases: 2 passed. One checks an old root account
table is reported as unknown; the other checks an unsupported nested setup
table fails to load without rewriting the private test file. The retained
workspace menu and control error-chain tests also passed 1/1 each.

The test binary linked on macOS with the pre-existing compact-unwind warning;
Cargo also repeated the existing `block v0.1.6` future-compatibility notice.
No native Settings window, real credential store, user config, or provider
endpoint ran. A30 remains not_run and T04 stays blocked by G-LINK.
