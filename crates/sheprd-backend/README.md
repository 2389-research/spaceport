# Terminal backends

The types the app shares with whatever runs its herdr-core terminals
(`TerminalId`, `TerminalInfo`, `Size`, `Spawn`, `Change`, `SurfaceUpdate`),
and two traits:

- `Backend`: attach one terminal on a host.
- `Terminal`: one attached terminal: semantic input, size, focus, and the
  next surface updates.

`sheprd-core-client` is the one implementation: herdr-core (Herdr 2) sends
raw bytes, which `sheprd-terminal` emulates in the client. Herdr 1
(`sheprd-client`) does not use this crate; its daemon emulates and sends
cells over `sheprd-protocol`.

Either way the app receives `sheprd_protocol` cells (`FrameData`,
`PaneSurfacePatchRow`) and sends `ClientPaneInputEvent`, so painting, hit
testing, and input routing do not depend on the generation.

Every method may block on I/O: call them from a worker thread, never from
the UI thread. This crate has no GPUI, runtime, or transport of its own.
