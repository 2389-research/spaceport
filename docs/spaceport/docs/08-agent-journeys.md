# End-to-end agent journeys

These journeys exercise the complete system rather than isolated modules. Their corresponding automated/manual scenarios live in `plan/acceptance.json`.

## J1 — Find the agent that needs attention

An operator asks, “Which agent in this workspace needs me?” The controlling agent reads client status/capabilities, then a bounded workspace summary. It receives stable Core/terminal/conversation references, reported status, pending attention where authoritative, freshness and coverage. It opens the exact view using the local client command.

It does not fetch every transcript, scan process names, parse an idle-looking screenshot or guess that the most recent terminal is the right one. If status was never reported, it says so. If the host disconnected, it does not use an old green indicator as current evidence.

**Success:** correct target and uncertainty, bounded metadata read, no runtime mutation. **Durable improvement:** any ambiguity found becomes a targeting fixture and catalog clarification.

## J2 — Respond without double-submitting

A user chooses a pending permission answer. The client binds the response to the precise conversation and request, invokes its existing backing contract, then observes the resulting state. Another view may answer first; the second receives the source protocol's stale/already-answered outcome.

If the connection drops after send, Spaceport does not resend on reconnect. It refreshes pending requests and reports an unknown result until reconciled. A generic tool-output card never becomes a permission prompt merely because it says “approve.”

**Success:** no fabricated consent, wrong-session answer or duplicate effect. **Durable improvement:** disconnect/dual-view fixtures and a regression test.

## J3 — Work across a tailnet without owning the network

The user supplies a Tailscale hostname/SSH alias. Spaceport asks Core to add/connect the machine using a non-installing path. Core exposes terminals/events from it. Spaceport draws host and workspace views from that source. When connectivity drops, Core retries while the client displays stale/disconnected state.

Spaceport does not create another SSH daemon, request tailnet admin tokens, install `hc`, update a remote agent or run a second retry scheduler. On Windows, interactive-login limitations are surfaced rather than worked around by unsafe credential capture.

**Success:** a remote program survives disconnect and client quit, and reconnect does not duplicate it. **Durable improvement:** exact supported topology and version recorded in capability evidence.

## J4 — Restore work without restarting it

A user closes the last window. The client exits, leaving Herdr running. On next launch, it restores geometry and references, obtains fresh Core/document state and reattaches to surviving terminals. Missing terminals are not recreated from remembered shell commands.

If a workspace was renamed by another client while this one was closed, the fresh Core document wins. There is no offline transcript browser, hidden prompt queue or separate agent recovery loop.

**Success:** views restored, processes not duplicated, current shared metadata respected. **Durable improvement:** lifecycle regression with actual process IDs and authoritative terminal identity.

## J5 — Modify the fork efficiently

A coding agent receives T04. The task capsule contains the relevant user decisions, deletion units, requirements, source pins and acceptance scenarios. It inspects dependency edges, extracts a shared pure type if needed, removes one coherent feature, runs focused tests and records actual evidence.

It does not read every unrelated source file, ask the user again whether to keep menus, remove a needed renderer type because its name says protocol, or change the terminal engine. Its handoff lists the changed contract and next executable step.

**Success:** unwanted behavior is unreachable, retained behavior is demonstrated, context and changes stay bounded. **Durable improvement:** updated removal record plus negative reachability and retained-feature tests.

## J6 — Discover a backend gap without expanding scope

The agent tries to preserve remote native chat or loopback forwarding and cannot find a supported Core route. It records the inspected method table, existing adapter behavior and the smallest missing contract. It keeps the required UI/renderer work isolated, marks the integration gate open and continues unrelated tasks.

It does not resurrect the phone server, add a custom tunnel manager, expose loopback servers publicly or call a terminal-only workaround full parity. It does not repeatedly ask preference questions to avoid the technical work.

**Success:** the blocker is precise and independently actionable; the product boundary stays intact. **Durable improvement:** a capability contract/test fixture that can be exercised when the backend gains support.
