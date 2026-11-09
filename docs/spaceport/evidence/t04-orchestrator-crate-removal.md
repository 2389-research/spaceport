# Orphan Orchestrator backend crate removal

Base: integration `b51283b6d`. The native Activity PR/Issues/Tasks UI,
polling and dispatch route was already removed. A direct/type/string/config
search found no production caller of `sheprd-orchestrator` outside its own
crate. Its `cache` and `private_file` helpers served only the crate's store
and worker. Retained avatar cache, GitHub PR cache, credential storage and
private files have independent owners. No attribution or license file was
inside the crate. The retained `sheprd-github`, `sheprd-vcs`, Overview, Usage,
Core, and diagnostics crates and their app routes remain.

This cut removes the orphan crate, its workspace member/dependency entries,
and the stale architecture row in root `AGENTS.md`. `cargo +1.99.0 metadata
--format-version 1 --offline` removed exactly the 26-line
`sheprd-orchestrator` package stanza from `Cargo.lock`; it changed no
third-party package or version. A source/manifest absence test failed before
the cut because the crate directory existed, then passed 2/2. The test also
checks that retained credential, avatar, GitHub PR cache, and Core
presentation files still exist.

`cargo +1.99.0 check -p sheprd-gpui --all-targets --locked --offline` passed
with no crate warnings; Cargo reported the existing future-compatibility note
for `block` 0.1.6. `just format-check`, `just check-file-size`,
`just check-sorted`, and `git diff --check` passed. Linked headless filters
on this revision passed: agent Overview 83/83, device Overview 30/30, Core
terminals 33/33, Usage 102/102, CLI 46/46, saved control tabs 9/9, and
avatars 38/38 (two opt-in tests ignored). Diagnostics passed 23/23 with one
deliberately ignored child entry; credentials passed 3/3. The app test link
emitted the existing macOS compact-unwind warning. The package
`tools/spec.py check` and `tools/test_spec.py` passed; they validate package
integrity, not the app's native behavior.

T04 remains `blocked` under G-LINK, and A29/A30 remain `not_run`. Separate
Git/PR actions and their owners remain for later removal; this crate cut does
not prove complete negative reachability or native window behavior.
