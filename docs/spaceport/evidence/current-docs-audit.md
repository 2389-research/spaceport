# Development documentation audit

Scope: root `README.md` and `docs/cli.md` at `7440fad3`. This checks the
current development branch, not the intended finished product. Source and
control-protocol types are the evidence; no desktop command was run.

## Direct claims

| Claim | Source observation | Result |
| --- | --- | --- |
| `README.md` says this branch still has excluded paths and no qualified release. | `plan/removals.json` keeps the affected audits in progress; `sheprd-updater/src/lib.rs` rejects release-version builds. | Accurate. |
| `docs/cli.md` said `tab list` includes reviews. | `control/server/tabs.rs::kind` omits retired source and review locations; `TabKind` still includes page, Orchestrator, Devices, Core terminal and editor tabs. | Corrected in `docs/cli.md`. |
| `docs/cli.md` described SSH native chat without a Core boundary note. | `chat/core_pane.rs` rejects machine-qualified Core IDs; `chat/chats.rs` dials a local adapter while inherited SSH uses a separate process. | Marked as a development-only legacy route and linked G-CHAT. |

## Pattern expansion

The stale review-tab row led to a search of current `TabKind`, `TabOp`, the
CLI parser and `docs/cli.md` browser commands. The browser commands and editor
tab remain reachable in this development branch, so the CLI page keeps them
while RM-BROWSER and RM-IDE are unfinished. The chat claim led to a check of
the pinned Core schema and current adapter route; raw remote terminal attach
does not establish structured remote chat. A broader audit of inherited crate
guides and install docs remains open as feature cuts continue.
