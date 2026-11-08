# Git, IDE, GitHub and dispatch removal reachability

Read-only source audit at `17e43f13`. T04 and RM-IDE/RM-GIT/RM-GITHUB/RM-DISPATCH audits remain `in_progress`; A29 and A30 remain `not_run`. No personal checkout, GitHub account, credential, provider, Core or user state was accessed or changed.

## Reachable graph

- `crates/sheprd-vcs/src/{git,checkpoint,code_index}.rs` owns checkout state, Git actions/PR creation, checkpoint refs and the Git-backed source index. GPUI reaches it through `code_search.rs`, `code_view.rs`, Git/menu/review/worktree/checkpoint modules, `window.rs::update_git` and `update_checkpoints`, and the code-search index held in window state. `app.rs` installs review-viewed and worktree state; `browser/{store,view}.rs` owns code, editor and review tabs, with `poll_code_views`/`poll_reviews` on window ticks. `code_server.rs` installs VS Code launcher/host workers at app startup.
- `crates/sheprd-github/src/{auth,credentials,device,token,pull_request,pr_actions}.rs` supplies sign-in, token storage, account/PR reads and outward PR actions. `crates/sheprd-gpui/src/{github,pull_request,repo_items,security,avatars}.rs`, menu account/PR flows, sidebar and settings/config expose it; `window.rs::poll_github` and Security's timed reader remain active. `sheprd-vcs`, `sheprd-orchestrator`, `sheprd-control-protocol` (quota wire type), and the phone crate also depend on `sheprd-github`. The profile shell may stay, but no GitHub/GitLab token, account or avatar fetch should survive merely to fill it.
- `crates/sheprd-orchestrator/src/{service,store,actions,beads,github,gitlab,azure}.rs` syncs issues/PRs/Beads into its SQLite store and dispatches agents. GPUI `orchestrator.rs`, `dispatch.rs`, `repo_items.rs`, activity/sidebar views, `browser/view.rs::poll_orchestrators` and app-installed dispatch history make it reachable. `config.rs` still parses GitHub/GitLab settings and credentials. Its `cache.rs` is currently used by the excluded Security reader and repository lookup; `private_file.rs` prepares that cache and the orchestrator store. Do not retain their SQLite/worker graph on the strength of a generic name. `avatars/cache.rs` is a separate image cache; its profile and repository-owner readers need their own retention audit.
- UI commands include New/Open/Remove Worktree, Worktree Notes, Review Changes, Git Menu, Open Code and Orchestrator (`crates/sheprd-commands/src/lib.rs`). CLI/control still exposes `code`, `review`, `orchestrator`, worktree/checkpoint/scripts, PR/security and related tab operations (`crates/sheprd-gpui/src/cli/control.rs`, `control/{client,server}/{tabs,work,hosts}.rs`, `crates/sheprd-control-protocol/src/protocol/{tabs,work,hosts}.rs`, `docs/cli.md`, `skills/herdr-gpui-browser/SKILL.md`). Settings, English/localized labels, icon assets and tests span `config.rs`, `settings_window`, `menu`, `sidebar`, `crates/sheprd-gpui/locales/*`, `crates/sheprd-gpui/src/icons.rs` and the GPUI README.
- Root `Cargo.toml` and GPUI/control/phone/orchestrator/VCS manifests keep all three crates in the build graph; `Cargo.lock` records their dependencies. Release workflow Git and GitHub operations, `git-cliff`, and the updater's GitHub release transport are separate engineering/update paths, not client GitHub account or issue-dispatch features. Removing product features by keyword must not remove those contracts.

## Retained boundary and cut order

`SPEC.md:17,37,65,69,73` keeps terminal/chat presentation, workspace navigation, plain source/diff text and explicit local file opening while excluding specialized source, review and Git tools. `chat/rich/{code,parse}.rs` and `chat/view/rich/code.rs` draw/copy bounded code text; `terminal/links.rs` and `chat/view/rich/links.rs` classify file links and reject remote paths. Those are retained primitives, whereas `sheprd-vcs::code_index` scans a Git checkout for a specialized IDE view. `chat/view/rich/links.rs` still offers `FileTarget::Editor`; its system-open route and safety checks must remain when that choice is removed.

The smallest complete negative unit found here still crosses another pending family: code search opens the code viewer, review/security/issue views share browser tab and activity state, and GitHub types enter VCS/orchestrator/control. T03 must first move retained view/CLI actions off legacy workspace/checkout assumptions; the browser cut must settle retained tab layout and file opening. Then remove each whole worker/UI/CLI/config/asset/dependency path, including startup and tick readers, without hiding it behind a flag. Negative tests should reject stale Git/IDE/account/dispatch commands and settings and show no GitHub/Beads/GitLab reader or code-server worker starts. Retained tests should cover plain `git diff`/source text in terminal/chat, bounded code copy, safe local file opening, and Core-backed workspace views. No product tests ran for this audit; package checks do not prove A29/A30.

## GitHub command route cut

This section follows the signed read-only audit above and needs a new
observation after integration. Starting from `ff8ff4ed`, the `sheprd pr`
and `sheprd security` CLI families, their host control operations and replies,
client text, server handlers, help, CLI docs and managed-skill claims are
removed. The control protocol's GitHub quota wire type and direct
`sheprd-github` dependency leave with their only route. A dependency-only
`Cargo.lock` change removes that one edge. Two Security helpers used only by
the removed control handlers leave as well; the retained alert parser tests
still check the original advisory ID and reject an unsafe one. Host listing,
host selection, port forwarding, usage and system commands retain their wire
and server paths. Product GitHub sign-in, PR and Security UI, readers,
credentials, and provider code remain for later T04 cuts.

New tests first failed because `HostOp::Pr` decoded and bare `pr` parsed.
They now reject PR status, comments, open, quota and Security list/open wire
operations and CLI commands. The wire test round-trips retained host list and
forward operations with the exact caller socket, window and workspace ID;
the CLI test keeps explicit window targeting on host and port commands.
`cargo +1.99.0 test --locked -p sheprd-control-protocol --lib` passed 54 tests,
and the focused GPUI `removed_github` test passed. The GPUI test binary
compiled after the orphan helper cleanup with no Rust code warnings; its link
reported the existing macOS compact-unwind-size warning and Cargo reported
the existing `block v0.1.6` future-compatibility notice.

The first default production `cargo +1.99.0 check --locked -p sheprd-gpui
--bin sheprd` passed but found those two newly unused Security helpers. They
were removed, then the focused GPUI test build passed. Free disk fell below
the agreed 8 GiB floor during that test link, so no post-cleanup production
check or further Cargo test ran locally. Run the merged production check and
retained host/port tests before treating this cut as warning-free. The isolated
branch sets control protocol 18→19; a concurrent T01 source-view cut also
claims 19, so the integrated version must become 20. T04 and RM-GITHUB stay
`in_progress`; A29 and A30 stay `not_run`. No personal account, credential,
Core state or source-view file was touched.

`just format-check`, `just check-file-size`, `just check-sorted`,
`git diff --check`, and
`uv run --with jsonschema==4.26.0 python tools/spec.py check` passed after
the last edit. The package check validates schema and links only.
