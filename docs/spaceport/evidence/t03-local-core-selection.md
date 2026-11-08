# T03: local Core group actions ignore legacy endpoint selection

The typed Core group is a local presentation choice even while the old
endpoint index points to an SSH host. The group can be selected from Core's
fresh document, but `new_core_terminal` and member click used to refuse based
on `selected_is_remote()`, which reads that unrelated legacy index. This was
a retained-UX bug found in the review after `f8030595`.

`new_core_terminal` now applies the remote-host refusal only when no Core
group is selected. It still checks the exact active Core key, joined hub ID,
and current document before a spawn. A member click already checks its joined
Core ID, fresh observation, document membership, and terminal identity; the
legacy endpoint refusal was removed from that path. With no Core group
selected, an SSH-host New Core Terminal command still refuses.

The headless test first failed because the local group received the legacy
“local only” warning; after the guard change it passed 1/1 and the retained
remote-host refusal passed in the same test. A separate ignored test used the
explicit pinned Core 0.1.18 binary in a private sandbox, created one terminal
and document member, and selected a remote legacy endpoint. Its first run
found a test fixture missing the Store global, which was corrected. It then
failed because no Core tab opened and passed after the member guard was
removed (1/1). The sandbox stopped its own Core and reaped its child; no
personal socket, daemon, or terminal was used. Cargo reported only the prior
macOS debug linker compact-unwind and pinned `block` future-compat notices.

This test proves the two local action routes and exact member identity in a
private Core fixture. It does not prove native pointer/focus behavior, remote
Core parity, Core-only window construction, or full T03/A09/A24 acceptance.
