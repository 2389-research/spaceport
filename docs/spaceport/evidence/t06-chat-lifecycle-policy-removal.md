# Desktop chat lifecycle policy removal

Source `2041aa09c`, integrated with the adapter detach fixture at `9bbc81c70889f1f6da29ed2cecee493e8c3afe35`.
SP-009 and reconciliation R4 already require removing desktop-owned
conversion/restart/recovery policy; Doctor Biz requested continuing this work.
No new product-scope decision was inferred or needed.

## Removed and retained contracts

Removed the GPUI conversion worker that sent Ctrl+C and typed a resume line;
its palette/window/tab-menu action, CLI, control request/reply, saved-shortcut
catalog entry, errors, labels and help; and the reboot recovery poll/claim/stamp
worker. Removed the adapter Keeper, its callback-only Opened report and the
chat-protocol restore-record filesystem subsystem. No actual saved record was
read, erased, migrated or replayed. Proven orphan helper methods were removed;
shared host scripts, pane/process readers and explicit overview reopen remain.

The private desktop control protocol is now 37; retired conversion request and
reply shapes reject. The chat attach protocol, endpoint namespace, reducer,
permission semantics and manual host-side adapter CLI retain their contracts.
This is not a namespace migration. Existing observed rich chat, terminal input,
attachment, chat show/hide/toggle and explicit selected-runtime chat creation
remain. The frontend neither restarts an absent adapter nor types agent commands
when restoring a view. Manual `sheprd-acp --resume` remains an explicit user
command in a runtime terminal; native Pi's session-loading behavior retains its
existing narrower guarantee, documented without changing runtime behavior.

## Regression and review evidence

Before removal, command/catalog, control-wire and app CLI/source guards failed
on the actual live conversion/recovery routes. A saved-shortcut test now refuses
the absent command. The headless GPUI fixture toggles chat presentation and
reconciles/disconnects; the adapter wire closes without any command, and an
ordered runtime sentinel proves no preceding terminal input or endpoint request.
This uses isolated protocol peers and is a contract test, not native end-to-end
proof. The real Session unit starts with headless exit-with-agent policy and no
local view, detaches its sole GUI, deterministically observes zero clients and
Flow::Continue, preserves session identity, emits no ACP command, and accepts a
later prompt in that same session.

Independent review found a stranded POSIX cfg after deleting a module, which
would have disabled an unrelated unsupported-platform test, and an overstated
manual Pi resume guarantee. The orphan cfg/comment and the docs were corrected;
all changed module parents were checked for stranded attributes. The retained
Windows test is not proven on macOS. A broad release suite also found an old
assertion reading the removed planner. It now checks the live process reader
and inferred foreground PID method; its focused tests and the full suite pass.
Both component reviews and the final code-phase review approved the source.

## Integrated verification on macOS arm64

Cargo used Rust 1.99.0, CARGO_INCREMENTAL=0 and locked/offline dependencies.

| Check | Result |
| --- | --- |
| ACP adapter binary tests | 378 passed |
| Shared library tests, all features | Chat protocol 58, commands 30, control 67, remote 10 passed |
| Full app default | 3,808 passed; nine existing ignores |
| Full app all features | 3,837 passed; nine existing ignores |
| CLI and build/identity integration | 34 passed, including 21 real CLI executable cases |
| Updated adapter in a private Core process | Two opt-in fixtures passed; see [process evidence](t06-adapter-detach-process.md) |
| Release Python discovery | 279 cases, eight skipped, zero failures after the stale source guard was fixed |
| Release security | 24 cases, one skipped, zero failures |
| Format, file size, sorted regions, i18n, commits and diff checks | Passed |

Strict app Clippy still fails on exactly the same 253 inherited diagnostic
code/message/primary-file/multiplicity entries; control still has its same three
constant-assertion findings. Core strict lint retains one type-complexity issue,
emitted for lib and lib-test; collecting complete diagnostics confirmed no
additional fixture warning. No allowance or hook bypass was added. Existing
block v0.1.6 future-compatibility and test-linker compact-unwind notices remain.

The lockfile changes only the removed chat-protocol dev dependency edge;
no package version upgrade or new crate was introduced. No upstream runtime
source, personal daemon, actual chat session or saved user record was modified.
Native GUI, real agents and remote/cross-platform chat parity were not tested.
The separate [Core duplex-route audit](t06-core-duplex-route-audit.md) keeps the
smallest external dependency explicit; a chat-named RPC is not inherently needed.
The exact delivered signed helper also passed both private Core fixtures; see
[preview evidence](t12-macos-preview-20261110.1.md). Specification tool tests
passed 21 cases; render/check/context passed from the package root. These only
validate the package and records, not application acceptance. Registry status
comparison confirmed that no task, acceptance or gap was promoted by these
component results. Final independent evidence review approved the claims and
artifact hashes; two stale release/platform gap descriptions were narrowed.
T06/G-CHAT and full native acceptance remain open/blocked. PACT writes remain
paused on the earlier incident; task pointers are in Kata and Chronicle.
