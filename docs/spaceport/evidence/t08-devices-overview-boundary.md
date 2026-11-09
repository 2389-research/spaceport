# T08: Devices overview dashboard and retained timeline boundary

Historical read-only source audit at `cce928f7`; no app or Core behavior changed
in that audit. Its Core-only retirement recommendation below is superseded
by the D58 qualification at the end of this note. The
Devices activity card still calls `open_devices_overview`. When a Core group
is selected, `browser_key()` returns no legacy workspace, so the button
flashes “no workspace” (`menu/devices/activity_card.rs:101–103`,
`devices_overview.rs:66–69`, `browser/view.rs:178–183`). Changing that call
to `active_browser_key` alone would open a Core-scoped tab whose data still
comes from legacy endpoints and snapshots (`devices_overview.rs:138–176,
225–237`, `model.rs:121–185`). Its agent-row actions still navigate legacy
panes (`table.rs:305–318`). That would show the wrong host under a Core key.

The same page draws CPU, memory and disk metrics (`devices_overview/view.rs`,
`table.rs`). SP-011 removes detailed host resource dashboards. Core host
management belongs in the existing machine catalog in
`menu/devices/core_catalog.rs`, which reads Core machine IDs, names, targets
and connection states from the shared observation. It is not a substitute
for agent history or resource metrics.

The overview also owns the current two-hour activity chart
(`devices_overview/chart.rs`, `history.rs`), while `sidebar/activity.rs` is
panel navigation. SP-011 and D49 keep that timeline without adding a new
history service. The next coherent cut must extract its retained activity
presentation to an existing activity surface, feed only supported Core
status/events, and mark missing history as unavailable. Then retire the
dashboard page, picker card, legacy host metrics and pane actions together.
Core 0.1.18 has no verified timeline-history feed; G-OBS and T08 remain
blocked. No source absence, native UI, or A20/A24/A41 acceptance is claimed.

A regression should select a Core group with two exact Core machine IDs and
an unrelated legacy endpoint with a working agent. The host manager must
show the Core identities; no legacy count, metric, history or pane action
may appear as Core activity. Stale Core facts must read as unavailable, and
legacy content must not be silently relabeled as Core data.


## D58 qualification and bounded cut (2026-11-09)

D58 permits the working Herdr 1 route. The earlier recommendation to retire
this whole page and its legacy pane actions was too broad: it would remove
the retained Devices activity chart, history, host/agent rows and terminal
navigation. The audit's warning against relabeling legacy observations as
Core facts still applies. No Core capability gap justifies deleting the
working Herdr 1 path or moving a session to another runtime.

The [resource-dashboard removal](t08-resource-dashboard-removal.md) removes
only CPU/GPU/memory/disk/uptime sampling and dashboard controls. Devices keeps
its existing bounded in-memory chart, host scope, row expansion, search and
exact endpoint/pane navigation. No new history service or Core feed was
added. G-OBS stays open and T08 stays blocked; native and platform acceptance
remain unproved.
