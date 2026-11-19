# Error-copy overlap with PR 123

Compared [PR 123](https://github.com/2389-research/spaceport/pull/123) with the local review fixes.
Reviewed PR head: `ea711ebc978ee81272cc9248cee60ffb279b540d`.
Its base matches this integration checkout: `91bda83a77e7e02bbc6d9ba02887cb95dc3876b9`.
This is a bounded reconciliation note, not a complete PR review or approval.
No PR code was applied or published by the original comparison.
Main later merged PR 123 at `57d6ea140722a613ce17beb9ec6d942e5d4c504f`.
The current integration retains its extra protocol copy alongside the safety guidance below.
See [the integration evidence](review-jargon-main-integration.md).

Both candidates improve client error text and localized connection guidance.
The local candidate also settles runtime names and split labels for Kata `cx2r`.
See [the local evidence](review-error-language.md).

PR 123 additionally rewrites protocol-error text and its Fluent catalog.
The local candidate still retains technical text there, including “surface patch identity mismatch.”
Those changes provide useful coverage beyond the local candidate.
Do not describe the whole PR as duplicate work.

Preserve the local candidate's stronger guidance during reconciliation:

- Timed-out input and requests may have taken effect. Check before retrying.
- Saved connection failures retain data. Do not replace that guidance with instructions to remove the file.

The local version and generation guidance also distinguishes which product needs an update.
Its consistently named recovery action is **Terminal → Reconnect**.
Keep typed errors, source chains, redaction, and wire values intact.

Kata `cx2r` remains open.
The extra protocol copy is reconciled in the integration branch.
Native presentation qualification remains pending.
This comparison does not establish that either PR's checks passed.
