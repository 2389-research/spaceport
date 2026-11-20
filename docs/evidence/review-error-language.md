# Error language and runtime names

Finding: `cx2r`.

Client diagnostics now describe the failed action in plain English.
Recovery guidance names **Terminal → Reconnect** consistently.
Request timeouts warn that the action may have happened before the connection failed.
Input timeouts warn that some input may have arrived.
Neither message encourages an unsafe blind retry.

Rust `Display` and Fluent retain the same English contract.
Error variants, source chains, retry classification, and wire values remain unchanged.
Version guidance still distinguishes an older runtime from a newer connection generation.
Only a newer generation asks for a Spaceport update.
Older or equal generations retain the `herdr update` instruction.

Visible runtime names use **Herdr TUI** and **Herdr Core**.
The actual `herdr-core`, `hc`, and helper executable names remain intact.
The [runtime guide](../herdr-client.md) explains runtime, connection, machine, and shortcut ownership.
It distinguishes public command names from the private Herdr-compatible key table.
Split Right and Split Down now match the command palette labels.

## Verification

- Client unit suite: 413 passed; four opt-in tests stayed ignored.
- Localized error suite: 53 passed.
- Version guidance consumers: three passed after strengthening the negative update assertion.
- Find consumers: 25 passed.
- Connection status consumers: six passed.
- Client all-target, all-feature Clippy passed.
- `just check-i18n` passed: 2,143 messages in 113 files.
- `cargo fmt --all --check` passed.

The first localized check caught four mismatches with Rust diagnostics.
The retained equality tests passed after correcting the producers.
The first client run also caught stale English assertions; typed error checks now cover relevant internal contracts.
Logs are `/tmp/spaceport-9s44-error-copy-*.log` and `/tmp/spaceport-9s44-error-guidance-final.log`.

Root reviewed the changed diagnostics, compatibility formatting, call sites, and documentation.
Native layout, OS keyboard routing, and screen-reader announcements remain pending.

## Presentation consumer correction

The image qualification's broad `presentation` filter also selected the application error tests.
It exposed one stale assertion for the replaced catalog-schema message.
Root updated that assertion to the current plain-language contract.
The test now supplies a synthetic unknown field and proves presentation hides that field.
It still requires the original typed JSON source through the application wrapper.
Both application error tests pass from `/tmp/spaceport-9s44-gpui.EcPUAT/spaceport-tests`.
Log: `/tmp/spaceport-9s44-error-presentation-final-green.log`.
No production error behavior changed in this correction.
