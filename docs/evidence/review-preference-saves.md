# Preference save lifecycle

Date: 2026-11-17. Scope: `dzyx` under `9s44`.
Doctor Biz authorized fixes and validation without commits or external writes.

## Cause and behavior

Preferences previously queued every snapshot on an unbounded channel.
Dropping the window detached its writer without joining during app shutdown.
The app could exit before the final snapshot reached its endpoint file.

Preferences now reuse the existing `state_file::Writer` and atomic replacement policy.
One pending snapshot and one wake token coalesce a burst to its latest value.
The initial read finishes on that worker before queued snapshots can replace the file.
Endpoint paths, stored fields, validation, and existing load behavior remain unchanged.

Dropping preferences closes the sender without waiting for disk work.
The worker retains its owner until pending writes drain.
An app quit callback can therefore find a worker after its window closes.
That callback joins only through GPUI's background executor.
The join takes the writer out of its mutex before waiting.

A foreground completion task owns each UI-only quit subscription.
The worker closes its completion signal after draining and releasing its owner.
The foreground task then removes the subscription.
Completed windows therefore leave no stale quit observers.
The completion channel uses the already locked `futures-channel` package.
No runtime or package version was added.

## Regression evidence

The real headless app shutdown test failed before the lifecycle fix.
After 256 saves and owner drop, shutdown observed no final sidebar width.
The fixed app shutdown reads the final width, 256, from the actual file.

Independent review found that detached quit observers accumulated after workers finished.
The repeated eight-lifetime regression failed with one retained weak reference.
It now observes zero weak references after each worker drains.
The root agent independently reviewed and approved the final ownership and pruning code.

The coordinated coalescing test blocks initialization, queues 256 snapshots, and closes the writer.
It observes one serialization, the final value, and no temporary files.
The blocked-worker test proves owner drop returns before disk work can proceed.
Serialization and replacement failures preserve existing destinations and remove their temporary files.

## Verification

All Cargo commands used `--locked` and the shared Cargo lock and target directory.

- `cargo test -p sheprd-gpui --bin spaceport preferences::`: 32 passed.
- `cargo test -p sheprd-gpui --bin spaceport state_file::`: four passed.
- `cargo clippy -p sheprd-gpui --all-targets --all-features -- -D warnings`: passed.
- Clippy used the validated Sparkle SDK at `/private/tmp/spaceport-cli-sdk.7S7nty/sdk`.
- `just check-i18n`: 14 checker tests passed; 2134 messages were valid and used.
- `git diff --check`, `just check-file-size`, and `just check-sorted`: passed.

Logs: `/tmp/spaceport-qnz9/dzyx-preference-tests.log`, `dzyx-state-tests.log`, and `dzyx-clippy-all.log`.
The debug test linker reports a compact-unwind size warning.
Cargo also reports the existing `block 0.1.6` future-incompatibility notice.
Clippy reports no code warnings.

## Limits

The shared writer flushes and atomically replaces files without per-save fsync.
This follows the existing app-state policy and avoids blocking past GPUI's quit budget.
Pinned GPUI 0.3.10 limits shutdown to 200 milliseconds.
Slow or failed storage can still exceed that budget or reject a save.
This change does not promise persistence after forced termination or power loss.

These tests use real files and headless GPUI app shutdown.
They do not establish native Windows runtime behavior or native keyboard/performance results.
Those checks and full workspace CI remain pending.
