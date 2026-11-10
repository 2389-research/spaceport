# Core-owned adapter detach process contract

Test commit `e4303a6b0` adds an opt-in topic fixture beside the existing private
Core adapter permission-race test. Both were rerun after integrating the desktop
lifecycle removal, using a freshly built candidate adapter at source `9bbc81c70`.

## Exact fixture identities and operations

The explicitly selected separately installed Core reports `hc 0.1.19` and has
SHA-256 `98b0653b40f22028eea6020b6bf3e39ff77e6012286437ca9f8efd24b7ecd758`.
Its source revision is unknown. The old pinned 0.1.18 test binary is absent;
the read-only pinned source remains `cd12ddd88bc11490228343291dc1096c9ed3bf77`.
Those are distinct observations, not interchangeable test claims.

An unsigned candidate adapter was observed before preview assembly/signing with
SHA-256 `9945a938e7c3b37284b41b754d87c6d7f2a7bd6dea611c83e72a62bfd9d9d191`.
For exact delivered-byte provenance, both fixtures were then rerun successfully
against preview 20261110.1's actual signed helper: SHA-256
`69e8c5d98b834497e12f436eb3fbd04d55ebf9b51172e6f4e5259ffdb7533c00`,
unchanged before/after. [Preview evidence](t12-macos-preview-20261110.1.md)
records its source and package checks.
The existing Sandbox clears the environment, uses short private HOME/XDG and
HC_SOCKET paths, sets HC_LAUNCHD_DIRECT for the exact direct-serve child, stops
only that private Core and reaps its child before deleting the sandbox. No
personal socket discovery/fallback or process-name/group kill is used.

Core starts the real adapter with keep:false and a fixed deterministic ACP peer,
not a real provider agent or an app mock mode. The fixture drops two chat clients,
the terminal attachment and the public Core handle; reconnects; verifies original
terminal address, PID, endpoint label/socket, cursor epoch, session and transcript;
then completes another prompt, permission answer and response. Explicit fixture
terminal kill, Core child reaping and sandbox removal are asserted. The Core
executable hash is unchanged before/after the integrated test pass.

The drop operation has no server close acknowledgement. This process fixture
therefore proves continued original process/conversation through client drops,
not an observed zero-subscriber interval before the new attach. The separate
adapter Session unit deterministically exercises the zero-client state. A first
fixture draft incorrectly expected a snapshot to include the subscriber being
added; the source sends its snapshot before Change::Clients. It now waits for
that actual protocol change with bounded reads, without arbitrary sleeps.

## Executed checks

- `cargo +1.99.0 build --locked --offline -p sheprd-acp --bin sheprd-acp`: passed
  on the integrated candidate, with CARGO_INCREMENTAL=0.
- `HERDR_CORE_TEST_BINARY=/Users/harper/.local/bin/herdr-core
  SHEPRD_ACP_TEST_BINARY=/Users/harper/Public/src/2389/spaceport-t00/target/debug/sheprd-acp
  cargo +1.99.0 test --locked --offline -p sheprd-core-client --features adapter-live-test
  --lib live::acp -- --ignored --nocapture`: two passed, zero failed. These are
  the permission-race and detach/reconnect fixtures, with exact-child cleanup.
- The same explicit two-test command passed again with SHEPRD_ACP_TEST_BINARY
  pointing to the signed delivery's `Spaceport.app/Contents/MacOS/sheprd-acp`.
  Two passed, no failures; exact private cleanup assertions held.
- On the isolated test branch, Core-client default and all-feature suites each
  passed 65 tests; 13/15 live-resource tests stayed opt-in. The two relevant live
  fixtures were explicitly selected and run, not counted as unrun ignores.
- Format/sorted/file-size/diff checks and independent review passed. Core strict
  lint remains failed on the unchanged connection type-complexity diagnostic;
  complete diagnostic comparison found no new fixture warning.

The fixed ACP peer is a process/protocol component fixture. These results do not
prove a real agent, native GPUI input, remote Core attach, OS permission UI or
Windows/Linux behavior. They also do not test the absent old pinned binary.
No Core repository, installed binary, user daemon or real conversation changed.
