# Removal and retention reachability at `50b9063be`

This source snapshot covers all 19 entries in [the removal plan](../plan/removals.json) at
commit `50b9063be` (`Merge pull request #93 from 2389-research/docs/registry-reconciliation`,
committed 2026-11-17). It supersedes the [snapshot at `54cb41c2`](removal-reachability-54cb41c2.md)
as the current navigation record; prior records remain intact. That earlier snapshot is 518
commits behind this one and still records RM-GIT, RM-GITHUB and RM-DISPATCH as present, so it
understates how much of T04 has landed. **Present** means a live source entry point still reaches
the named feature. **Absent** means the named source route and its listed build entry points are
gone. **Partial** means some routes are gone while others remain, or a retained path still depends
on the old owner. These are reachability labels, not acceptance results. Unless fully qualified,
paths below start at `crates/sheprd-gpui/src/`.

Each label below comes from grepping its family's markers over `crates/` and `scripts/` at this
commit with test sources excluded, so a hit is a production route rather than a fixture. Where a
family is absent, the absence test that forbids its return is named as the boundary.

| Plan item | Source reachability | Current entry point or absence boundary |
| --- | --- | --- |
| RM-IDE | **Absent** | `browser/location.rs:8–21` keeps only the `Devices` and `CoreTerminal` tab kinds, and `browser/store/saved.rs:66–86,103–109` keeps `Px0`/`VsCode` only as parse-only `SavedPlace` names whose `restore` returns `None`. `Command::SplitEditor` (`crates/sheprd-commands/src/catalog.rs:234`) runs the Core pane split at `window/commands.rs:417`, not an editor. `scripts/release/tests/test_px0_source_absence.py` guards the route. [IDE route audit](t04-code-ui-dependency.md). |
| RM-GIT | **Absent** | `crates/sheprd-vcs` is gone from the workspace and no `update_git`, `menu/git.rs`, `git2` or `gix` route remains; `scripts/release/tests/test_git_mutation_absence.py` and `crates/sheprd-gpui/src/menu/git_read_removal_tests.rs` are the guards. `sidebar/tokens.rs:178–179` still paints a plain branch token and `worktree_banner.rs:9–41` renders a worktree build's branch and PR link, both retained text rather than a Git workflow. [Git dependency](t04-git-status-pr-dependency.md). |
| RM-GITHUB | **Absent** | `crates/sheprd-github` and `octocrab` are absent, as `scripts/release/tests/test_github_crate_absence.py` checks. The remaining `github` source hits are the refused old `[github]` table at `config.rs:584–589` (`Error::RemovedGitHub`), `config/tests/removed_github.rs`, its `error-config-removed-github` message, a chat agent-name alias and repository URLs. [Account dependency](t04-github-account-pr-dependency.md), [crate removal](t04-github-crate-removal.md). |
| RM-DISPATCH | **Absent** | `crates/sheprd-orchestrator` is gone, as `scripts/release/tests/test_orchestrator_crate_absence.py` checks. `group_menu.rs:27–44` offers only chat, port, close and split rows, and Orchestrator survives only as the parse-only `SavedLocation::Orchestrator` (`browser/store/saved.rs:50–52,120–127`) plus a comment at `preferences.rs:311`. [Crate removal](t04-orchestrator-crate-removal.md), [dispatch dependency](t04-git-ide-dispatch-dependency.md). |
| RM-BROWSER | **Absent** | No `wry`/webview dependency or `browser/{native,page,kit,markdown,…}` module remains, as `scripts/release/tests/test_webview_source_absence.py` checks. `browser/location.rs:8–21` has no page kind, `SavedLocation::Web` is parse-only (`browser/store/saved.rs:41–43,120–127`), and `browser/view.rs:192–205` `open_workspace_page` hands a `WebUrl` to `cx.open_url` — the retained explicit system-browser route. `browser/annotate.rs` is gone; `[keybindings]` refuses `annotate_selection` as an unknown key. [Browser dependency](t04-browser-dependency.md), [system open](t04-browser-system-open.md). |
| RM-CLOUD | **Absent** | `crates/sheprd-remote/src/lib.rs:1–9` exports only `agent`, `host_script`, `snapshot` and `error`; no provider, container, image or cloud-setup module remains. `scripts/release/tests/test_cloud_setup_absence.py`, `test_container_provisioning_absence.py`, `test_container_stop_absence.py` and the per-provider `test_*_source_absence.py` guards hold the cut. `endpoint/catalog.rs:1–5,53–77` loads SSH and WSL devices only, and the surviving `[container] engine` and `[cloud]` keys are documented as inert data at `config-gpui.example.toml:469–478`. [Container provisioning cut](t04-container-provisioning-removal.md), [cloud setup removal](t04-cloud-setup-removal.md). |
| RM-TELEPORT | **Absent** | No `teleport` module, symbol or host-script compatibility API remains: `scripts/release/tests/test_teleport_engine_absence.py:18–29` checks the modules and exports, `:34–49` the excluded host-reader symbols, and `test_teleport_native_absence.py:13–31` the native menu, page, mark-file and `poll_teleport` routes. Retained callers use the narrow `host_script::`, `agent::AgentKind` and `snapshot::ProcessInfo` seams at `chat/existing.rs`, `pane_move.rs`, `overview/reopen.rs` and `processes.rs`. [Engine removal](t04-teleport-engine-removal.md), [cut order](t04-teleport-current-cut-order.md). |
| RM-AUTOMATION | **Absent** | `sheprd-automate`, `sheprd-automation` and `sheprd-luau` are absent from the source tree and workspace manifest, as `scripts/release/tests/test_automation_absence.py` checks across build and package inputs. `automations.rs:1–40` renders an unavailable activity page and starts no scheduler. Passive observation stays under G-OBS. [Removal](t04-automation-removal.md). |
| RM-MODS | **Present**, one route | The client mod/Luau crates are absent and `config/tests/removed_mods.rs:4–14` checks the obsolete table, but the daemon-side plugin link activation is still live: `window/links.rs:454,464` gate on `supports_link_activate` (`state.rs:121,516`), `window/links.rs:513` sends `LinkRequest::Activate`, `links.rs:45` maps it to `Method::PaneLinkActivate` (`crates/sheprd-client/src/method.rs:51`), and a `plugin_link_failed` reply is read as a spent click (`links.rs:315–320`, `crates/sheprd-client/src/response.rs:50–51`). This is G-LINK's live path. [Legacy plugin link dependency](t04-legacy-plugin-link-dependency.md), [client mod cut](t04-client-mods.md). |
| RM-PHONE | **Absent** | Phone, push, nearby and iOS crates/helpers are absent from the workspace, the justfile and the Windows package inputs, as `scripts/release/tests/test_phone_absence.py:25–58` checks, with `test_ios_source_absence.py` beside it. `device_tags.rs` keeps local tags only, and `crates/sheprd-control-protocol/src/protocol.rs:18` records the removed phone control in the version log. Previously installed personal services were not inspected or removed. [Removal](t10-phone-removal.md). |
| RM-LEGACY | **Present** | `connection.rs:6–7,17` still connects with `sheprd_client` and the old daemon; `cli.rs:116,128,141,190` defaults to `ConnectTarget::Local`, and `crates/sheprd-client/src/lib.rs:36,37,39,43,45` still compiles `session`, `sessions`, `ssh`, `transport` and `workspaces`. D58 keeps this route. Core observation does not retire it. [Default bridge dependency](t04-default-bridge-dependency.md). |
| RM-PROVIDERS | **Absent** route | No client-owned provider probe, sign-in or credential-store route remains: `crates/sheprd-usage/src/lib.rs` is 151 lines of report and presentation models with no `poll`/`refresh`/`allow` worker, and `scripts/release/tests/test_provider_probe_absence.py` plus `test_credential_store_absence.py` guard startup, UI, package and dependency inputs. `usage/registry.rs:628` `all()` and the `[usage]` display config are retained metadata. Authoritative usage parity stays gated on G-OBS. [Implementation removal](t04-provider-implementation-removal.md), [credential tail](t04-credential-store-tail-removal.md). |
| RM-ANNOTATION | **Absent** terminal route | The terminal annotation command and composer are gone: `crates/sheprd-commands/src/tests/terminal_annotations.rs:4–10` and `visual_annotations.rs:5` refuse `annotate_selection` and `toggle_annotate`, and `config/tests/terminal_annotations.rs:4–6` checks the old binding is unknown. Normal selection/copy/paste remains. `browser/annotate.rs` is absent under RM-BROWSER and was never terminal annotation. [Terminal cut](t04-terminal-annotations.md). |
| KEEP-CORE | **Present** source | `main.rs:36` compiles `core_terminals`; `core_terminals.rs:17–32` includes panes, tabs, observation, view and the workspace document, and `crates/sheprd-core-client/src/lib.rs:18–24` exports the Core adapter, terminal, changes, invalidation and attachment. The old bridge still overlaps startup. [Core boundary](t02-core-boundary.md). |
| KEEP-TERM | **Present** source | `main.rs:105–106` compiles the terminal view and painter; `crates/sheprd-terminal` remains the Core raw-byte emulator and `crates/sheprd-backend` its shared terminal value types. This does not establish native terminal parity. [Parser evidence](t05-terminal-parser.md). |
| KEEP-SESSION | **Partial** | `core_terminals/workspace_document.rs:1–2` is a versioned presentation grouping stored as an opaque Core document and changes view metadata only, while `crates/sheprd-client/src/lib.rs:36,37,45` still supplies the old session and workspace paths. Core-only selection and layout parity remain open. [Workspace document](t03-workspace-document.md). |
| KEEP-CHAT | **Partial** | `main.rs:25` compiles chat UI, and `scripts/release/package-macos.sh:20–21`, `sign-macos.sh:37` and `package-windows.py:20` still ship `sheprd-acp` as a host companion. Local adapter behavior and remote Core duplex parity are separate T06/G-CHAT gates; source reachability is not chat qualification. [Adapter route](t06-local-adapter-route.md). |
| KEEP-CONTROL | **Present** source | `main.rs:26,33` compiles CLI and control; `crates/sheprd-control-protocol/src/protocol.rs:42` defines protocol version 37 and `crates/sheprd-commands/src/catalog.rs:61` the command catalog. Exact target and native behavior require their own tests. [Core catalog CLI](t09-core-catalog-status.md). |
| KEEP-SHELL | **Present** source | `main.rs:50,112` compiles locale and updates, `:142` diagnostics, and `crates/sheprd-launcher` starts the app without a console window. The inherited `sheprd-updater` crate is gone in favour of D60's app constants and D65's Sparkle route (`updates.rs:1–20`). Release identity and native packages are still unqualified. [Release identity audit](release-identity-audit.md). |

## Cross-check against the T04 unblock plan

The T04 unblock plan note (`notes/t04-unblock-plan.md`, on the unmerged
`feat/t04-unblock-plan` branch, written at this same commit) carries its own family table. Every
label agrees. RM-IDE, RM-GIT, RM-GITHUB, RM-DISPATCH,
RM-BROWSER, RM-CLOUD, RM-TELEPORT, RM-AUTOMATION, RM-PHONE, RM-PROVIDERS and RM-ANNOTATION have
no live excluded route; RM-MODS keeps exactly the one daemon-side link activation route it names,
at the lines it cites (`window/links.rs:513`, `links.rs:45`, `state.rs:516`,
`crates/sheprd-client/src/method.rs:51`, `crates/sheprd-client/src/response.rs:51`); RM-LEGACY and
the KEEP-* families are the retained D58 property, not T04 work.

The note's two corrections to the register also hold, verified here rather than copied:
`git rev-list --count 54cb41c2..50b9063be` is 518, and
[`removal-reachability-54cb41c2.md`](removal-reachability-54cb41c2.md) does record RM-GIT,
RM-GITHUB and RM-DISPATCH as present. A34 still has no evidence list in
[`plan/acceptance.json`](../plan/acceptance.json).

One correction in the other direction: the note's RM-PROVIDERS cell says "display metadata
retained under D12". The retained usage surface at this commit is the report and presentation
models plus `usage/registry.rs` display names, icons and external links; only
`[usage]` display configuration is still parsed. That is narrower than "display metadata" reads,
and G-OBS still owns the authoritative usage feed.

## Proof limits

This pass inspected current source, manifests, absence tests, and linked evidence. It did not
build or launch the app, Core, a remote host, or a release package, and the cited absence tests
were read, not run in this documentation cut. The [native platform record](native-platform-results.md)
covers earlier private macOS fixtures but no final Spaceport desktop qualification; Windows and
Linux native runs remain absent. A28–A34 and A40–A42 remain `not_run`, and G-LINK, G-OBS,
G-PLATFORM and G-RELEASE remain open. Greps show absence of named routes, not absence of
behaviour. The plan's `audit_status`, task statuses, gaps, and
[acceptance results](../plan/acceptance.json) remain authoritative registers and are unchanged by
these source labels.

Two residues sit outside every family above and are recorded here rather than fixed by a
read-only pass: `main.rs:143–144` still carries the dangling `Git status, without GPUI.` comment
from the removed `sheprd-vcs` import, and `config-gpui.example.toml:273–274` still lists
`git_menu` and `toggle_annotate` among the unbound commands although neither is in the catalog at
`crates/sheprd-commands/src/catalog.rs`.

## Snapshot checks

The 19 table IDs match the 19 `plan/removals.json` IDs exactly, with no duplicates.
`tools/spec.py check` passed schema validation and all 328 local links. It does not require every
`evidence/` file to be listed in a register, so no register was touched: `plan/tasks.json` keeps
T04 `blocked` with its existing evidence list, and no status anywhere changed. `tools/test_spec.py`
passed. `git diff --check` passed. These are package and source checks, not application tests.
