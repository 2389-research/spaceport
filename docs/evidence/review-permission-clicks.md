# Chat permission clicks

Date: 2026-11-17. Scope: `m225` under `9s44`.

Permission buttons now accept only the first click or tap in a sequence.
Keyboard activation keeps working because GPUI reports its click count as one.
Rejected repeat clicks still stop propagation into the underlying pane.

The regression renders a real permission button against an isolated framed peer.
After its first answer, the peer immediately replaces the permission.
The second click must not answer that replacement.
A later Cancel command forms a FIFO barrier, so the test needs no sleep-based absence check.
A fresh first click then answers the replacement normally.

The corrected baseline failed without the guard: Answer for permission two arrived before the barrier.
The restored guard passed the same regression.
The fixture uses its actual pane target; the initial hard-coded barrier target was corrected.
Independent security review found no material findings in the guard, command path, or protocol-order assertion.

Logs: `/tmp/spaceport-9s44-permission-clicks-corrected-red.log` and `permission-clicks-verified.log`.
These are headless GPUI pointer events and real protocol frames.
Native OS double-click delivery remains a platform QA check.
