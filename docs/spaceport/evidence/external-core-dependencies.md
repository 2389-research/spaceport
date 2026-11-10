# External Core dependencies for retained behavior

T06 is blocked on G-CHAT. A chat-specific RPC is not required: the existing adapter
already has a transparent stdio connector. The missing verified contract is a
Herdr-owned authenticated, non-PTY duplex byte route to that installed connector
on an exactly identified remote machine/terminal. Installed Core 0.1.19's schema
and pinned 0.1.18 source provide batch `machine.exec`, which takes all input and
returns bounded output after exit, rather than an interactive channel. Terminal
streams have repaint/backlog-loss semantics unsuitable for assuming lossless
adapter framing. The [duplex route audit](t06-core-duplex-route-audit.md) records
source/schema evidence and the narrower external dependency. Local process,
permission and detach fixtures do not close native or remote parity. Spaceport
must not add another SSH/transport manager, remote server or automatic helper
deployment to hide the gap.

T07 is blocked on G-PORT. The same Core's 32-method live `rpc.discover` table has no service-forwarding method. The smallest external dependency is a Core-owned way to forward a named remote machine's loopback-only TCP service to a client-local loopback endpoint, with explicit target identity, authorization/refusal, endpoint and listener lifetime, cancellation, and failure behavior. A private loopback-service test must prove A27; opening an already reachable URL does not. G-FILE is still open separately: `terminal.put` and text file RPCs have not proved all binary upload, download, overwrite and cancel semantics. No new Spaceport transport manager or remote service is authorized.

The source and live method evidence is in [T00 Core runtime](t00-core-runtime.json) and [T00 baseline](t00-baseline.md). These are task blockers, not claims that all independent T06/T07 preparation is complete. Continue T01–T05 and the independent local chat, file and UI work while the external contracts are resolved.

T02 and T07 host actions are also blocked on G-MACHINE: Core exposes each machine's permanent ID but Connect, Rename and Remove target mutable names only. The smallest Core addition is ID targeting or an atomic expected-ID guard; [the T02 identity finding](t02-machine-identity-gap.md) gives the source and test evidence. Catalog reading and the independent [Core Add form](t02-core-add-ui.md) do not satisfy A04.
