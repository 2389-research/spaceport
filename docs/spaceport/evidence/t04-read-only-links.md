# Terminal links open read-only, without a plugin link handler

## What shipped

PR #142 (crew task apwa, Doctor Biz decision 2026-11-18) removed the client's
last live plugin-execution path reachable from an ordinary click.
`window/links.rs` no longer sends `LinkRequest::Activate`, and Herdr 1's
`pane.link.activate` — the route that could answer `handled: true` after a
`[[link_handlers]]` plugin had already run and claimed the click — is never
called. Links open read-only: Spaceport reads the target itself and opens it
(URL to the browser, file to the default app).

## Why

An ordinary link click must never run a plugin handler as a side effect. The
route also carried a dilemma the superseded audit
(`t04-legacy-plugin-link-dependency.md`) documents: activation supplied wrapped
URLs that the row-local parser cannot safely infer, but keeping it meant plugin
handlers could run from any click. Removing activation resolves the dilemma on
the safety side; wrapped-URL support waits on the G-LINK read-only route.

## Register

The narrowing was never given a decision number. Decision **D72** records it
(added 2026-11-25 during the p3es register reconciliation).

## What remains open

- G-LINK stays open: the verified read-only typed link target/span contract
  (exact terminal identity, content revision, viewport/cell) does not exist yet.
- Acceptance scenarios A30 (negative plugin activation) and A33 (retained
  wrapped URL/file/chat/native interactions) remain `not_run` in
  `acceptance.json`; crew task p3es tracks them.
- Whether `pane.link.resolve`'s reply carries anything beyond `regions`, and
  whether `pane.selection.read` joins soft-wrapped rows, needs one opt-in
  Herdr 1 sandbox probe (the I0 probe). Declined for now by the owner
  (2026-11-25); revisit when a sandbox machine is available.
