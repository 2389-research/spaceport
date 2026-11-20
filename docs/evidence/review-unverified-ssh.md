# Unverified SSH forwarding

Review child: `hv7g`, under `9s44`.
Verified on 2026-11-17 in `fix/review-squad-findings`.

`HostKeys::Skip` previously disabled host-key checks and used empty known-hosts files.
It left agent forwarding enabled when SSH config requested it.
The shared bridge policy already disabled X11 forwarding.
The skipped-host policy did not itself prohibit either forwarding feature.

The skipped-host policy now passes `ForwardAgent=no` and `ForwardX11=no` before other connection options.
Verified-host options remain empty.
The bridge, explicit port forward, private master, and private channel reuse this policy.
See `crates/sheprd-client/src/ssh/unverified.rs`, `ssh.rs`, and `ssh/mux/shared.rs`.

The config example and app README explain that skipped hosts lose agent and X11 forwarding.
There is no UI description for this manually configured setting.

## Regression evidence

The new `ssh::unverified::tests::forwarding` tests run the installed OpenSSH parser with `-G`.
Each subprocess uses a private `-F` config that enables agent and X11 forwarding.
The config disables hostname canonicalization.
The subprocess clears its inherited environment and uses a private HOME.
No test connects, authenticates, reads private keys, or uses a real host.

The baseline run produced two expected failures on `forwardagent no`.
The verified-host control passed with `forwardagent yes`.
After the fix, all seven unverified-host tests passed.
The explicit policy test also proves X11 refusal independently of the bridge policy.

All Cargo commands used Rust 1.99.0 and this shared build prefix:

```sh
/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock \
  env RUSTUP_TOOLCHAIN=1.99.0 CARGO_INCREMENTAL=0 \
  CARGO_TARGET_DIR=/Users/harper/Public/src/2389/spaceport/.worktrees/upstream-fixes/target \
  cargo
```

| Command suffix | Result |
| --- | --- |
| `test --locked -p sheprd-client ssh::unverified::tests::forwarding -- --nocapture` before fix | Two expected failures; one control passed |
| `test --locked -p sheprd-client ssh::unverified::tests -- --nocapture` after fix | Seven passed |
| `test --locked -p sheprd-client ssh:: -- --nocapture` | 67 passed; one existing real-host mux test ignored |
| `clippy --locked -p sheprd-client --tests -- -D warnings` | Passed after scoping fixture unwrap allowances to the test module |

`/usr/bin/ssh` reported OpenSSH 10.5p1 with LibreSSL 3.3.6.
Changed Rust files passed `rustfmt --check --edition 2024`.
`git diff --check`, `just check-file-size`, and `just check-sorted` passed.

## Limits

These tests prove effective OpenSSH configuration, without making a remote connection.
They do not prove remote agent sockets or X11 sessions over a live SSH server.
Windows and Linux execution remain CI checks.
Full workspace CI and native UI checks were not run for this policy-only change.
The parent task owns the independent security review.
