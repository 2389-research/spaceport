# SIFT Codebase Audit

**Status:** `INCOMPLETE`
**Repository:** `2389-research/spaceport`
**Revision:** `64d51b05befd80a7930e09f5ecf8fe555cf45c7f` on `main`
**Scope:** whole repository
**Audit mode:** read-only; no tests, builds or scripts were run
**Date:** 2026-11-13

Why `INCOMPLETE`: every subsystem row got a reviewer, and 42 findings survived an independent critic. But reviewers did not open every file they were assigned, even after a second gap-fill round, and one row of manifests and published contracts was never reviewed. Section 10 lists exactly what was not read. Nothing in this report claims coverage of those files.

## 1. Executive summary

The repository is about 380,000 lines of Rust, Python, shell and Swift. The GPUI app crate holds 304,000 of them.

The audit inventoried 46 rows: 34 from the first split, 8 gap-fill rows, and 4 rows for non-source material. 31 rows carry accepted recommendations, 14 are explicit skips, and 1 was not reviewed. There are 42 accepted findings. 12 more were drafted and then dropped after the critic's pass.

Five themes account for most of the value.

1. **Inherited code the fork can no longer reach.** The Herdr 1 SSH add-device wizard, with its relay helper, is unreachable in production: the only constructor of its request type is `#[cfg(test)]` (SIFT-S25-01). Three SSH entry points in the client crate have no caller (SIFT-S03-01). Five retired tab kinds still sit on the live tab type (SIFT-S31-01). The upstream release pipeline is guarded to another repository and another maintainer and cannot run here, yet its tests run on every pull request (SIFT-S32-01).
2. **A security gate that is off but looks on.** The supply-chain workflow and the Nix build still allow only the upstream repository and actor. In this fork cargo-deny, zizmor and the scheduled advisory scan are skipped on every run, and a skipped job shows green (SIFT-S32-02). The coordinator and the critic both confirmed the guards.
3. **One fact held in two places.** The selected host's live state exists as `SheprdWindow::live` and as `Endpoint::live`, with about two dozen hand-written "which copy" branches (SIFT-S16-02). Three reviewers reported this independently. The same shape appears for a Core tab's pane tree (SIFT-S17-01), for settings rows declared in a search catalog and again on the page (SIFT-S27-01), and for the stale-connection fence that eight structs each rebuild by hand (SIFT-S21-02).
4. **Lifecycles spread over booleans and options.** The ACP session (SIFT-S07-01), the endpoint link (SIFT-S16-01), the chat pane connection (SIFT-S23-02), notifications (SIFT-S30-02) and the tab drag (SIFT-S19-01) each encode a small state machine in independent fields. Several permit contradictory states that reviewers traced to wrong user-visible status. Those traces were made by reading code, not by running it.
5. **Lists declared several times.** A new catalog command touches up to seven places and only one is compiler-checked (SIFT-S09-01). CLI families, footer menu items and endpoint states are dispatched on strings (SIFT-S10-01, SIFT-S26-01, SIFT-S16-01).

**Highest-value first slice:** switch the guards in `.github/workflows/supply-chain.yml` to the fork's identity and watch one pull request run it. It is a few lines, and it turns a dead security gate back on.

**What is evidenced and what is inferred.** Dead-code and duplicate-declaration findings rest on grep and file evidence that the coordinator or the critic re-checked. Findings that describe a wrong runtime state (SIFT-S07-01, SIFT-S16-01, SIFT-S17-01, SIFT-S19-01, SIFT-G7-01) are inferences from reading; each names the test that would confirm it. The critic tested one concrete claim in each of 15 findings: 14 held, and one count in SIFT-S21-01 was wrong and is corrected below.

**Decisions that are yours, Doctor Biz.** Two findings are large deletions that need your explicit approval before anyone acts: SIFT-S25-01 (the add-device wizard) and SIFT-S32-01 (the upstream release route). One reviewer also raised a policy question that is not a simplification: session delete runs `herdr session stop` behind a confirmation dialog, and decision D58 says never stop a runtime. It is recorded in section 7.

## 2. Coverage contract

How to read the boundaries:

- `gpui/` means `crates/sheprd-gpui/src/`.
- A name ending in `*` means the module file, its directory and its tests.
- A row that names a directory owns every file under it, tests included, unless a more specific row names that file. The more specific row wins. Rows S02b and G1 to G8 are such carve-outs: they were split out after the first round so that files a reviewer had been assigned but did not open got their own review.
- Ownership is not the same as having been read. Section 10 lists what was assigned and not opened.

| ID | Subsystem | Exact ownership boundary | Key files/interfaces/tests | Status |
|---|---|---|---|---|
| S01 | Wire protocol | `crates/sheprd-protocol`, including `tests/fixtures/` (the frozen generation-1 contract) | `wire.rs`, `codec.rs`, `frame.rs`, `surface_*.rs`; `tests/compatibility.rs` | `skip` |
| S02 | Client session worker | `crates/sheprd-client` except what S02b and S03 name; includes `lib.rs`, `error.rs`, `src/tests*`, `tests/` | `run_connection`, `Barrier`, `Command`; `src/tests/focus_barrier.rs` | `recommend: SIFT-S02-01` |
| S02b | Client data paths | `crates/sheprd-client/src/{transport*,connect,catalog*,workspaces*,scrollback*,surface_images*,terminal_attach*,forward*,text*,method*,event,options,limits,compat,host_theme,app_name,control_dir,environment,process*}` | `scrollback::decode_response`, `Error::Endpoint` | `recommend: SIFT-S02B-01` |
| S03 | Client remote access | `crates/sheprd-client/src/{ssh*,wsl*,remote_host,private_endpoint*,login_env*,local_path*,local_network*,upload*,script*,settings_files*,discovery*}` | `ssh.rs`, `ssh/serve.rs`; samples `connect.rs` `Transport` (owned by S02b) | `recommend: SIFT-S03-01` |
| S04 | Backend traits and herdr-core client | `crates/sheprd-backend`, `crates/sheprd-core-client` | `Backend`, `Terminal`, `CoreBackend`; `just test-core` | `recommend: SIFT-S04-01` |
| S05 | Terminal emulator | `crates/sheprd-terminal` | `emulator.rs`, `cells.rs`, `input*.rs` | `skip` |
| S06 | ACP agent adapters | `crates/sheprd-acp/src/{claude*,codex*,pi.rs,pi/,agents*,harness*,observe*,history*,project*}` except what G4 names | the three `bridge.rs`, `harness/out.rs` | `recommend: SIFT-S06-01` |
| S07 | ACP session and in-pane view | the rest of `crates/sheprd-acp` except what S06 and G4 name | `session.rs`, `session/agent.rs`, `session/commands.rs` | `recommend: SIFT-S07-01` |
| S08 | Chat attach protocol | `crates/sheprd-chat-protocol` | `change.rs`, `wire.rs`, `model.rs` | `recommend: SIFT-S08-01` |
| S09 | Control protocol and command catalog | `crates/sheprd-control-protocol`, `crates/sheprd-commands` | `COMMANDS`, `Request`, `shortcuts/*` | `recommend: SIFT-S09-01, SIFT-S09-02` |
| S10 | Control server, client and CLI | `gpui/control*`, `gpui/cli*`, `crates/sheprd-gpui/tests/cli*` | `FAMILIES`, `control/client/help.rs`; `docs/cli.md` | `recommend: SIFT-S10-01, SIFT-S10-02` |
| S11 | Usage, remote helpers, ports | `crates/sheprd-usage`, `crates/sheprd-remote`, `gpui/{usage*,listening_ports*,port_forward*,processes*}` | `shell.rs`, `error.rs`, `tunnel.rs` | `recommend: SIFT-S11-01, SIFT-S11-02` |
| S12 | Diagnostics and logging | `crates/sheprd-diagnostics`, `gpui/{log_window*,log_targets*,config_diagnostic*}` | `kept_target`, `LOG_TARGETS` | `recommend: SIFT-S12-01` |
| S13 | Localisation | `crates/sheprd-i18n`, `gpui/{i18n*,direction*}`, `crates/sheprd-gpui/locales.rs`, the structure of `crates/sheprd-gpui/locales/`, `scripts/{check-i18n.py,test_check_i18n.py}` | `fl!`, `i18n/check.rs` | `recommend: SIFT-S13-01` |
| S14 | Build identity and test harnesses | `crates/{sheprd-launcher,embed-resource,test-support}`; `crates/sheprd-gpui/{Cargo.toml,build.rs,build_identity.rs,preview_identity.rs,release_version.rs,windows_resource.rs,windows.manifest,development-app-id,macos-preview-app-id,examples/}`; `crates/sheprd-gpui/tests/` except `cli*`; `gpui/{smoke*,performance*,constants*}` | `preview_identity::select`, `tests/live_gui.rs` | `recommend: SIFT-S14-01` |
| S15 | Configuration and persisted settings | `gpui/{config*,preferences*,state_file,window_state,lenient,storage_warning}` except what G7 names; `crates/sheprd-gpui/config-gpui.example.toml` | `Preference`, `edit_key_at`, `state_file::Writer` | `recommend: SIFT-S15-01` |
| S16 | Endpoints and connection | `gpui/{endpoint*,connection*}` except what G3 names | `Endpoint`, `ConnectionBridge`, `endpoint/lifecycle_tests` | `recommend: SIFT-S16-01, SIFT-S16-02` |
| S17 | Herdr Core terminals | `gpui/{core_terminals*,group_terminals*,core_feature}` except what G7 names | `TabPanes`, `Hub`, `Projection` | `recommend: SIFT-S17-01, SIFT-S17-02` |
| S18 | Terminal element and painting | `gpui/{terminal*,terminal_painter*,mark_layer*,caret,smooth_scroll*,contrast*,scrollback*,fonts,font_picker,font_sizes,presentation*}`, `gpui/window/regions*` | `paint_frame`, `Look`; `PERFORMANCE.md` | `recommend: SIFT-S18-01` |
| S19 | Tab strip and tab gestures | `gpui/window/{tab_strip*,tab_drag*,tab_tear*,tab_float*,tab_ghost*,tab_start*}`, `gpui/{tab_layout*,tab_menu*,reorder,panel_resize*,pane_move*}` | `TabDrag`, `render_tab_strip` | `recommend: SIFT-S19-01` |
| S20 | Window input, links and text fields | `gpui/window/{links*,selection*,pending_input*,input*,file_links*,select_hint*}`, `gpui/{input*,link*,links*,text_editing,multiline_input*,search_input*,dialog_input*,truncate}` | `Editing`, `PressedLink` | `recommend: SIFT-S20-01` |
| S21 | Window shell | `gpui/window.rs` and everything in `gpui/window/` that S18, S19, S20, G1 and G2 do not name (construct, render, lifecycle, commands, zen, layout_save, chrome_focus, flash, reconnecting, monitors, sound, and the window test modules) | `SheprdWindow`, `command()`, `local_error` | `recommend: SIFT-S21-01, SIFT-S21-02` |
| S22 | Rich chat view core | `gpui/chat/view.rs` and `gpui/chat/view/` except what G5 names | `render_chat_text`, `ChatPane.menu` | `recommend: SIFT-S22-01, SIFT-S22-02` |
| S23 | Chat lifecycle | `gpui/chat.rs` and `gpui/chat/` except what S22 and G5 name (includes `chat/tests*`, `preview*`, `selection.rs`, `glide`, `page`, `highlight`, `pasted`) | `Target`, `ChatPane`, `connect` | `recommend: SIFT-S23-01, SIFT-S23-02` |
| S24 | Sidebar | `gpui/sidebar.rs` and `gpui/sidebar/` except what G6 names; `gpui/state/` | `displayed_devices`, `WorkspaceDrag`, `DeviceDrag` | `recommend: SIFT-S24-01, SIFT-S24-02` |
| S25 | Devices UI | `gpui/menu/devices.rs` and `gpui/menu/devices/` except what G6 names; `gpui/setup_relay*` | `Step`, `setup::Request` | `recommend: SIFT-S25-01` |
| S26 | Menu shell and context menus | `gpui/menu.rs` and `gpui/menu/` except what S25 and G6 name; `gpui/{menus*,pane_menu*,group_menu*,settings_panel*}` | `Page`, `MenuState`, `menu_items` | `recommend: SIFT-S26-01, SIFT-S26-02` |
| S27 | Settings window | `gpui/settings_window.rs` and `gpui/settings_window/` except what G7 names; `gpui/toggles*` | `CATALOG`, `ui::row`, `save_with` | `recommend: SIFT-S27-01` |
| S28 | Palette, keymap, overview | `gpui/{palette*,keymap*,actions*,overview*,jump_list*,navigation}` | `Entry`, `Keymap`, `RunCommand` | `skip` |
| S29 | Application shell | `gpui/{main,app.rs,app/,error*,updates*,release_notes*,integrations,automations,agent_skill*,about*,bundled*,app_icon,icon_font_notice*}` | `run()`, `updates::available` | `skip` |
| S30 | Notifications, sound and small primitives | `gpui/{notifications*,notification_mutes*,sound*,bell,motion*,backdrop,icons,ago,caption*,accessibility,progress}` | `Notice`, `tick`, sound policy | `recommend: SIFT-S30-02` |
| S31 | Native tab store | `gpui/browser*` | `Location`, `Store::with_saved` | `recommend: SIFT-S31-01` |
| S32 | Release tooling and CI | `scripts/release/` (133 files), `.github/`, `homebrew/`, `cliff.toml`, `deny.toml` | `ci.yml`, `release.yml`, `macos-preview.yml` | `recommend: SIFT-S32-01, SIFT-S32-02` |
| S33 | Developer tooling and spec tools | `justfile`, files directly under `scripts/` except the two i18n ones, `scripts/git/`, `.githooks/`, `setup.sh`, `mise.toml`, `flake.nix`, `docs/spaceport/tools/`, `docs/spaceport/plan/*.json` | `spec.py`, `just ci` | `recommend: SIFT-S33-01` |
| G1 | Window media and notices | `gpui/window/{toasts*,system_notifications*,image_source*,images*,device_themes*,accessibility*,backdrop*,report_issue*,config_diagnostic*,status_bar*,announcement*}` | `click_toast`, `DrawnFrom`, `Posted` | `skip` |
| G2 | Terminal mouse, copy mode, find, clipboard, drops | `gpui/window/{mouse*,copy_mode*,find*,clipboard*,prefix*,transfers*,file_drop*,wsl_drop*,server_keys*,double_shift*,function_keys*}`, `gpui/{copy_mode*,find*,words,osc52}` | `Gesture`, `FileTransfer`, `WslPaste` | `skip` |
| G3 | Daemon start, sessions, network quality | `gpui/{daemon*,sessions*,network_quality*,restart_manager,worker_stop,herdr_settings*}`, `gpui/endpoint/{history,follow,config_reload}*`, `gpui/endpoint/wake/network*` | `connect_or_start`, `Sessions` | `skip` |
| G4 | ACP remainder | `crates/sheprd-acp/src/{herdr*,process*,pictures*,files*,cli*,relay*}`, `attach/downgrade*`, `tui/` except `tui/view*`, `claude/{settings,permission,tasks,tools}*`, `codex/{settings,approval,items}*`, `pi/tools*`, `project/todos*` | tool and permission mapping tables | `skip` |
| G5 | Rich chat rendering | `gpui/chat/{rich*,prepared*,visual*,widgets*,summary*,text*}`, `gpui/chat/selection/document*`, `gpui/chat/view/{rich*,widgets*,visuals*,embeds*,summary*}`, `gpui/chat/view/layout/` | `Block`, `Inline`, `ChatLayout::row` | `skip` |
| G6 | Sidebar rows, workspace and host menus | `gpui/menu/{workspace*,worktree_open*,worktree_note*,worktree_shortcut*,sessions*,shortcut_editor*,version_mismatch*,open_link*,mute*,colors*}`, `gpui/sidebar/{layout.rs,row*,agents*,activity*,tokens*,animated*}`, `gpui/menu/devices/{host_menu*,wsl*,core_catalog*,picker*,rail*,activity_card*,scope*}`, `gpui/{devices_overview*,close_modal*,theme_picker*,worktree*,device_tags*,state.rs}` | `Picker`, `MenuState.opening` | `skip` |
| G7 | Themes, fonts, Core terminal view | `gpui/config/{theme,fonts,cjk_fonts,bitmap_fonts,font_weight,ssh_hosts}*`, `gpui/settings_window/themes*`, `gpui/core_terminals/{view*,worker*,screen*}` | `ThemeName`, `resolve_fonts_with` | `recommend: SIFT-G7-01` |
| G8 | Platform integration | `gpui/{caffeine*,launch_at_login*,app_badge*,titlebar*,taskbar*}` | `Cup`, `Taskbar::launcher` | `skip` |
| N1 | Static web, design and assets | `site/`, `website/`, `design/`, `assets/`, `netlify.toml`, non-Markdown files under `docs/` (screenshots, evidence images) | root `netlify.toml` publishes `site/`; `website/` is the inherited sheprd.org site | `skip` |
| N2 | Prose and agent state | `docs/**/*.md`, `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `AGENTS.md`, `CLAUDE.md`, `LICENSE`, `NOTICE`, `*-NOTICE.md`, `gotchas.md`, `.pact/`, `.kata.toml`, `.claude/`, `.private-journal/` | not source | `skip` |
| N3 | Lockfiles and translation copy | `Cargo.lock`, `flake.lock`, the message text inside `crates/sheprd-gpui/locales/**/*.ftl` | sources are `Cargo.toml` and `flake.nix`; the `.ftl` files are hand-edited source whose structure S13 reviewed and whose wording is out of scope | `skip` |
| N4 | Workspace manifests and published contracts | root `Cargo.toml` (workspace dependencies and lints), `rust-toolchain.toml`, `clippy.toml`, `taplo.toml`, `.config/nextest.toml`, `.gitattributes`, `.gitignore`, `docs/spaceport/contracts/control.schema.json`, `skills/spaceport/SKILL.md` | the control-socket schema mirrors S09's Rust types; the skill mirrors the CLI contract in S10 | `incomplete: not reviewed` |

## 3. Prioritized recommendations

| Priority | Finding | Subsystem | Impact | Confidence | Effort | Blast radius | Prerequisites |
|---:|---|---|---|---|---|---|---|
| 1 | SIFT-S32-02: One repository and actor guard for CI workflows | S32 release tooling and CI | high | high | small for the slice | subsystem | none |
| 2 | SIFT-S07-01: One lifecycle phase for the ACP session | S07 sheprd-acp session and in-pane view | high | medium | medium | subsystem | none |
| 3 | SIFT-S25-01: Delete the unreachable Herdr 1 SSH add-device flow | S25 devices UI | high | high on unreachability | medium | cross-subsystem | owner approval for the deletion |
| 4 | SIFT-S03-01: Remove SSH entry points only removed features called | S03 sheprd-client remote access | medium-high | high for serve and Git-config | small | cross-subsystem | SIFT-S25-01 first |
| 5 | SIFT-S09-01: Declare each catalog command once | S09 control protocol and command catalog | high | high | medium | subsystem | none |
| 6 | SIFT-S16-02: One accessor for an endpoint's current live state | S16 endpoints and connection | medium-high | medium-high | small | cross-subsystem | none |
| 7 | SIFT-S16-01: One typed link phase for `Endpoint` | S16 endpoints and connection | high | medium-high | medium | cross-subsystem | easier after SIFT-S16-02 |
| 8 | SIFT-S27-01: Build each Settings row from its catalog entry | S27 settings window | medium-high | high | medium | subsystem plus locale files | none |
| 9 | SIFT-S33-01: Run the spec validator in the gate | S33 developer tooling and spec tools | medium | high | small | local | none |
| 10 | SIFT-S31-01: Separate the saved tab schema from the live tab model | S31 native tab store (`browser/`) | medium-high | medium-high | medium | cross-subsystem | none |
| 11 | SIFT-S26-01: Typed items for the footer menu | S26 menus and context menus | medium | high | small | local | none |
| 12 | SIFT-S10-01: Declare each CLI family once | S10 control server, client and CLI | medium | high | small | subsystem | none |
| 13 | SIFT-S11-01: Split the port-scan shell errors from the usage model | S11 usage, remote helpers, ports | medium | high | small | cross-subsystem | none |
| 14 | SIFT-S22-02: One value for the chat settings menu and its search field | S22 rich chat view | medium | medium-high | small | local | none |
| 15 | SIFT-S21-01: One owner for local operation failures | S21 window shell | medium-high | medium | medium | application-wide | none |
| 16 | SIFT-S04-01: Shrink the `Backend` abstraction to the seam that is substituted | S04 sheprd-backend and sheprd-core-client | medium | medium-high | small | cross-subsystem | none |
| 17 | SIFT-S17-01: The tab `Store` as sole owner of a Core tab's pane tree | S17 Herdr Core terminals | medium | medium | medium | cross-subsystem | SIFT-S31-01 first |
| 18 | SIFT-S14-01: One typed build channel | S14 build identity and test harnesses | medium | medium | small | subsystem | none |
| 19 | SIFT-S15-01: One checked writer for settings saves | S15 configuration and persisted settings | medium | medium-high | medium | subsystem | none |
| 20 | SIFT-S20-01: One key-to-edit mapping for the three text fields | S20 window input, selection and text fields | medium | high for the duplication | medium | subsystem | none |
| 21 | SIFT-S26-02: Nest popup pages under their family | S26 menus and context menus | medium | medium | small for the leak fix, large for the nesting | cross-subsystem | none |
| 22 | SIFT-S17-02: One definition of the exact-Core fence | S17 Herdr Core terminals | medium | medium-high | small | subsystem | none |
| 23 | SIFT-S02B-01: One endpoint response-envelope decoder | S02b sheprd-client data paths | medium | medium | medium | cross-subsystem | SIFT-S25-01 |
| 24 | SIFT-S24-01: One listing rule for the sidebar, and no duplicate fold set | S24 sidebar | medium | high for `collapsed_repos` | medium | cross-subsystem | SIFT-S16-02 |
| 25 | SIFT-S23-02: One connection phase for a chat pane | S23 chat lifecycle | medium-low | medium-high | small | local | none |
| 26 | SIFT-S19-01: One phase enum for the tab drag | S19 tab strip and tab gestures | medium | medium | small | local | none |
| 27 | SIFT-S24-02: One sidebar gesture | S24 sidebar | medium-low | medium | medium | cross-subsystem | none |
| 28 | SIFT-G7-01: One "follows Herdr" predicate for themes | G7 themes, fonts and config loaders | low-medium | medium | small | cross-subsystem | none |
| 29 | SIFT-S21-02: One captured connection fence for deferred actions | S21 window shell | medium | medium | medium | cross-subsystem | none hard |
| 30 | SIFT-S30-02: One stage enum for a notification | S30 notifications and sound | medium | medium-high | medium | cross-subsystem | none |
| 31 | SIFT-S22-01: Draw chat text from the prepared selection document | S22 rich chat view | medium | medium | medium | subsystem | none |
| 32 | SIFT-S32-01: Retire the inherited upstream release route | S32 release tooling and CI | medium-high maintenance | medium | large | subsystem | owner decision |
| 33 | SIFT-S09-02: Exhaustive caller and wait classification on `Request` | S09 control protocol and command catalog | medium-low | medium | small | cross-subsystem | none |
| 34 | SIFT-S02-01: One state type for the API request lease | S02 sheprd-client session worker | medium | medium | medium | local | none |
| 35 | SIFT-S10-02: Parse CLI families straight into protocol ops | S10 control server, client and CLI | medium-low | medium | medium | subsystem | easier after SIFT-S10-01 |
| 36 | SIFT-S08-01: A shared change-stream follower | S08 sheprd-chat-protocol | medium | medium | small | cross-subsystem | none |
| 37 | SIFT-S13-01: One validator for Fluent file shape | S13 localisation | medium-low | medium | small | subsystem | none |
| 38 | SIFT-S23-01: A chat target's source as one enum | S23 chat lifecycle | medium-low | medium | small | cross-subsystem | none |
| 39 | SIFT-S06-01: One typed turn-ended reply for the three bridges | S06 sheprd-acp agent adapters | medium-low | medium | small | subsystem | none |
| 40 | SIFT-S18-01: One cell-metrics value for paint, hit testing and IME | S18 terminal element and painting | medium | medium | medium | cross-subsystem | none |
| 41 | SIFT-S11-02: One SSH local-forward lifecycle | S11 usage, remote helpers, ports | medium | medium | large | cross-subsystem | none |
| 42 | SIFT-S12-01: One rule for kept log targets | S12 diagnostics and logging | low | medium | small | local | none |

### Finding details

Path shorthand: `gpui/` means `crates/sheprd-gpui/src/`. Other paths are relative to the repository root.
"Verified" in a confidence line means the coordinator re-opened the cited code. "Reviewer-read" means only the subsystem reviewer read it.

---

#### SIFT-S32-02 · One repository and actor guard for CI workflows

- **Finding ID:** SIFT-S32-02
- **Authoritative subsystem:** S32 release tooling and CI
- **Title:** Reduce each workflow guard to the fork's one identity, and decide the never-switched workflows on purpose
- **Verdict:** recommend
- **Primary evidence:** `.github/workflows/supply-chain.yml:21-29,46-52`; `.github/workflows/nix.yml:43-50`; `.github/workflows/ci.yml:35-44,62-71,112-122,223-234,279-290,341-352,402-412,462-471,521-525`; `scripts/release/check-ci.py:12-13,59`
- **Interfaces and call sites:** `cancel-merged-pr.yml:14`, `close-low-confidence-prs.yml:16`, `star-history.yml:14`; branch protection context `Format, lint, and test`
- **Tests and intent evidence:** `scripts/release/tests/test_ci_policy.py:20,81,94,116` and `scripts/release/test-release-security.py:48-54` pin the guard text
- **Current representation:** each job repeats a hand-written `if:` naming repositories and actors. `ci.yml` was extended for the fork; five other workflows were not.
- **Current complexity or invalid states:** in this fork, zizmor, cargo-deny, the scheduled advisory scan and the Nix build are skipped on every trigger. A skipped job reports green, so nothing shows that `deny.toml` is unenforced.
- **Why it is material:** a supply-chain gate that looks on but is off. Nine copies of one guard in `ci.yml` also drift.
- **Proposed representation:** one guard expression for `2389-research/spaceport` and its maintainer, with the upstream arms removed; one `WINDOWS_ENABLED` condition for the D66 pause; fork defaults in `check-ci.py`. Then an explicit choice per unswitched workflow: enable supply-chain (recommended), enable or delete nix and the three housekeeping workflows. This absorbs the former SIFT-S33-02: if the PR-closure workflow goes, its script and 373-line test go with it (`scripts/close-low-confidence-prs.py`, `scripts/test_close_low_confidence_prs.py`), and the orphan `scripts/test_mockup_recipe.py`, which runs a just recipe that no longer exists, is deleted.
- **Why it is simpler:** one identity instead of three repositories and two actors in nine places; no silently dead gates.
- **Implementation scope:** seven workflow files, `check-ci.py`, `test_ci_policy.py`, `test_check_ci.py`, `test_spaceport_release_transition.py`, `test-release-security.py`
- **Smallest credible slice:** switch `supply-chain.yml` guards to the fork identity and watch one PR run it
- **Regression risks:** enabling cargo-deny may surface existing advisory or licence failures; the required-check name must not change; fork-PR secret isolation must stay
- **Migration concerns:** land the supply-chain switch alone first so failures are attributable
- **Existing validation:** the policy tests above, actionlint, zizmor
- **Additional validation required:** one real PR and one main push showing the supply-chain jobs ran, not skipped
- **Impact:** high; a security gate is off
- **Confidence:** high; verified the guards in `supply-chain.yml` and `nix.yml`
- **Implementation effort:** small for the slice, medium for all seven files
- **Blast radius:** subsystem (CI)
- **Prerequisites:** none
- **Priority:** 1

---

#### SIFT-S07-01 · One lifecycle phase for the ACP session

- **Finding ID:** SIFT-S07-01
- **Authoritative subsystem:** S07 sheprd-acp session and in-pane view
- **Title:** Hold the agent lifecycle in one phase enum with one teardown shared by `fail` and `exited`
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-acp/src/session.rs:141,156,161,452-461`; `crates/sheprd-acp/src/session/agent.rs:93,162-175,356-388`; `crates/sheprd-acp/src/session/commands.rs:59-62,128,141-147,171,188`
- **Interfaces and call sites:** `fail` is reachable mid-turn from `session.rs:445`, `session/agent.rs:31,295`, `session/commands.rs:152`; consumers `crates/sheprd-acp/src/tui/view.rs:436-446`, `tui/render.rs:181`
- **Tests and intent evidence:** tests assert only the status after a failure (`session/tests/flow.rs:122,216`, `session/tests/resume.rs:22`); none covers a failure during a turn or a command after exit
- **Current representation:** `acp_session: Option<SessionId>`, `ready: bool`, `turn: Option<i64>`, plus the transcript's `status` and `turn`
- **Current complexity or invalid states:** `fail` sets `Failed` and clears the queue but leaves the turn, `ready`, `asked` and open permissions; `exited` clears each by hand. Traced by reading: a mid-turn failure leaves the pane reporting Blocked with a running timer; a late `session/new` reply after `fail` sets Idle again; `SetMode` after exit can overwrite `Exited` with `Failed`; `pending` is never cleared.
- **Why it is material:** user-visible wrong status after failures, on the path that reports to Herdr
- **Proposed representation:** private `enum Phase { Opening, Loading(SessionId), Ready { session, turn: Option<i64> }, Closed }`; commands match on it once; `fail` and `exited` call one `close(status)`
- **Why it is simpler:** five fields become one; teardown has one definition
- **Implementation scope:** `session.rs`, `session/agent.rs`, `session/commands.rs`, about 150 lines, plus a new topic file under `session/tests/`. No protocol change.
- **Smallest credible slice:** make `fail` call the same teardown as `exited`, before introducing the enum
- **Regression risks:** confirm `Failed` is terminal; decide whether `fail` withdraws or cancels open permissions; do not close on `Inbound::Closed(None)`
- **Migration concerns:** keep the order of notice and status changes that clients see
- **Existing validation:** `session/tests/{flow,permissions,auto_accept,commands,resume,settings,detach}.rs`
- **Additional validation required:** tests for a mid-turn failure with an open permission, `SetMode` or `Cancel` after exit, a late reply after failure, a prompt during `Loading`
- **Impact:** high
- **Confidence:** medium; verified that `fail` and `exited` differ as described; the downstream effects were traced by reading, not run
- **Implementation effort:** medium
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 2

---

#### SIFT-S25-01 · Delete the unreachable Herdr 1 SSH add-device flow

- **Finding ID:** SIFT-S25-01
- **Authoritative subsystem:** S25 devices UI
- **Title:** Remove the ten unreachable `Step` variants, their host-key and local-network code, and `setup_relay`
- **Verdict:** recommend
- **Primary evidence:** `gpui/menu/devices/add_device.rs:41-74,414-416`; `gpui/menu/devices/setup.rs:264-266,304-305` (`Request::new` and `with_host_keys` are `#[cfg(test)]`); `gpui/menu/devices/host_key.rs:18-125`; `gpui/menu/devices/local_network.rs`; `gpui/menu/devices/host_key_choice.rs`; `gpui/menu/devices/add_device/offer.rs`
- **Interfaces and call sites:** `submit_device_setup` calls only `submit_core_device` (`add_device/core.rs:151-229`). `gpui/setup_relay*` (about 1,190 lines) is used in production only by `setup.rs:398,408,711` and the helper entry `gpui/app.rs:304`. Live users of `setup.rs` that must stay: `host_menu.rs:259,327-328,654`.
- **Tests and intent evidence:** tests enter the flow by injecting `Step::Checking` (`menu/devices/tests.rs:163`, `tests/host_key.rs:16`, `tests/key_login.rs:14,44`); `add_device.rs:401-412` is marked "retained for its transition tests"
- **Current representation:** `Step` has 16 variants. `Setup` carries `claim`, `task`, `host_keys` and `_edits` for a flow whose request type production cannot construct.
- **Current complexity or invalid states:** render, footer, key handling and the per-frame poll all match dead arms. The dead flow offers remote Herdr install, update and server restart, which decision D58 forbids, so it is a re-wiring hazard as well as weight.
- **Why it is material:** roughly 2,500 production lines plus tests and Fluent keys with no reachable path
- **Proposed representation:** `Step { Form, CoreConnecting, CoreAdding, CoreWaiting, CoreUnknown, Saved }` and `Setup { fields, step, error }`. `setup.rs` keeps `Claim`, `claim_saved`, `remove`, `rename`, `device_label`, `run_bounded`, `cli_environment`, `last_line`.
- **Why it is simpler:** the type states only what can happen; a forbidden capability stops existing
- **Implementation scope:** delete `host_key.rs`, `host_key_choice.rs`, `local_network.rs`, `add_device/offer.rs`, `setup_relay*`, `tests/cli/setup_relay.rs`; trim `add_device.rs`, `setup.rs`, `saved_warning.rs`, `menu/devices/error.rs`, root `Error::SetupRelay`, matching tests and `add-device-*` Fluent keys
- **Smallest credible slice:** delete the ten variants and the four files; leave `setup_relay` for a second change
- **Regression risks:** `Error::Adding` stays live through `claim_saved`; i18n exhaustiveness tests must shrink with the variants; `sheprd-client` helpers (`probe_host_with`, `scan_host_key`, `trust_host_key`, `wait_for_local_network`, `store_saved_host`) and `Config::save_host_keys` may become orphans and need a caller check
- **Migration concerns:** no decision records this flow as parked; confirm with the owner that Herdr 1 SSH add is not planned to return. Large deletion, so it needs explicit approval.
- **Existing validation:** `add_device/core/tests.rs`, `tests/core_flag.rs`, `tests/core_catalog.rs`, host-menu remove and rename tests, `devices_overview/tests/add_device.rs`
- **Additional validation required:** `just check-sorted`, i18n catalog checks, a native Add Device pass with Core on (success, refusal, unknown)
- **Impact:** high; largest removable surface found, and it removes a forbidden capability
- **Confidence:** high on unreachability (verified: constructor is test-only, WSL has its own `Step`); medium on intent
- **Implementation effort:** medium; wide but mechanical
- **Blast radius:** cross-subsystem (devices menu, app entry, i18n, possibly sheprd-client)
- **Prerequisites:** owner approval for the deletion
- **Priority:** 3

---

#### SIFT-S03-01 · Remove SSH entry points only removed features called

- **Finding ID:** SIFT-S03-01
- **Authoritative subsystem:** S03 sheprd-client remote access
- **Title:** Delete the unix-socket server forward, the remote Git config read and (on decision) the application-built bridge
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-client/src/ssh/serve.rs:1-50`; `crates/sheprd-client/src/ssh.rs:255-310,358-380,652-672`; `crates/sheprd-client/src/connect.rs:48-65,115`; `crates/sheprd-client/src/lib.rs:91-99`
- **Interfaces and call sites:** no non-test caller of `serve_command`, `remote_origin_url`, `remote_config_value` or `connect_command` anywhere under `crates/` or `scripts/` (coordinator grep). The one production connector, `gpui/connection.rs:185-232`, yields only a `Stream`.
- **Tests and intent evidence:** `ssh/tests/serve.rs` uses fixture `sheprd-vscode-ab12` (RM-IDE); `scripts/release/tests/test_boat_source_absence.py:35` and `test_coder_source_absence.py:38` assert `pub fn connect_command` still exists
- **Current representation:** three extra ways to spawn `ssh`, each with its own policy variation; a two-variant `Transport` whose `Bridge` arm nothing constructs
- **Current complexity or invalid states:** every change to SSH policy, host keys or the mux must reason about unreachable paths. `Error::InvalidGitDir` and `ssh::Error::InvalidSocket` describe failures that cannot occur.
- **Why it is material:** about 150 production and 200 test lines of dead transport surface in security-sensitive code
- **Proposed representation:** delete `ssh/serve.rs`, the Git-config functions with `run_remote`, and the two error variants. If the bridge is not reserved, delete `connect_command` and `Bridge` and collapse `Transport` to `Stream`.
- **Why it is simpler:** every remaining SSH entry point has a live caller
- **Implementation scope:** `ssh.rs`, `ssh/serve.rs`, `ssh/error.rs`, `error.rs`, `lib.rs`, `connect.rs`, the crate README, `gpui/i18n/errors/client.rs:107`, `gpui/i18n/errors/client/ssh.rs:14`, two Fluent keys
- **Smallest credible slice:** the serve and Git-config removal, which nothing pins
- **Regression risks:** unused-import fallout on Windows and Linux cfg paths; the two release tests pin `connect_command`
- **Migration concerns:** check decision D06 ("generic Core machines") before removing the bridge
- **Existing validation:** `ssh/tests`, `tests/errors.rs`, i18n error exhaustiveness tests
- **Additional validation required:** default and all-feature builds on each platform through CI; grep for each removed symbol and Fluent key
- **Impact:** medium-high
- **Confidence:** high for serve and Git-config (verified no callers); medium for the bridge
- **Implementation effort:** small
- **Blast radius:** cross-subsystem (client, gpui i18n, release tests)
- **Prerequisites:** SIFT-S25-01 first, then recompute the no-caller list once (that deletion orphans more client helpers); owner decision on the bridge
- **Priority:** 4

---

#### SIFT-S09-01 · Declare each catalog command once

- **Finding ID:** SIFT-S09-01
- **Authoritative subsystem:** S09 control protocol and command catalog
- **Title:** One `macro_rules!` table generating `Command`, `COMMANDS` and the macOS and Linux default shortcuts
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-commands/src/lib.rs:15-112,136-672,808-864`; `crates/sheprd-commands/src/shortcuts/macos.rs:5-61`; `crates/sheprd-commands/src/shortcuts/linux.rs:8-69`
- **Interfaces and call sites:** `gpui/keymap.rs:134,157,178,239-243`; `gpui/control/server/app.rs:358`; `gpui/palette.rs:670-694`
- **Tests and intent evidence:** `crates/sheprd-commands/src/tests.rs:23-158` mirrors the whole catalog and asserts `COMMANDS.len() == expected.len()` (line 132); `tests.rs:179-226` lists the window commands again
- **Current representation:** enum, a 107-row table, an exhaustive `request` arm, two per-platform matches ending in `_ => &[]`, and a positional test mirror
- **Current complexity or invalid states:** a new command touches five to seven places and only `request` is compiler-checked. A variant with no `COMMANDS` row compiles and is unbindable (`keymap.rs:241` continues). A variant missing from a shortcut match silently gets no default. `TabNumber(u8)` admits 0 and 10-255.
- **Why it is material:** every command is declared in up to seven places, and the positional test mirror is the kind of append-at-the-same-spot list CLAUDE.md's "Parallel Pull Requests" section warns about (the assert compares against a list, not a literal total, so it is the spirit of that rule, not its letter)
- **Proposed representation:** one table row per command (`Variant, "name", "message", mac: [...], linux: [...]`) and one numbered form that expands 1..=9. `windows.rs` and `request` stay hand-written. Table order is user-visible, so no keep-sorted markers.
- **Why it is simpler:** every variant has a row and defaults by construction; tests shrink to invariants
- **Implementation scope:** `lib.rs`, `shortcuts.rs`, `shortcuts/macos.rs`, `shortcuts/linux.rs`, `tests.rs`. Public surface unchanged; no gpui edits needed.
- **Smallest credible slice:** the table and generated `COMMANDS`, keeping the old test mirror for one commit as an equality check
- **Regression risks:** catalog order; `[keybindings]` names are a user config contract (including `check-for-updates`); macro readability
- **Migration concerns:** compare generated `(command, name, message, mac, linux)` tuples with the current ones before deleting the mirror
- **Existing validation:** `sheprd-commands` tests, `shortcuts/tests.rs`, `gpui/keymap/tests/platform_defaults.rs`, `gpui/config/tests/keybindings.rs`
- **Additional validation required:** unique names and unique base shortcuts as invariant tests; Linux and Windows CI legs
- **Impact:** high; recurring cost on every new command
- **Confidence:** high; verified the length assert and file size (881 lines)
- **Implementation effort:** medium
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 5

---

#### SIFT-S16-02 · One accessor for an endpoint's current live state

- **Finding ID:** SIFT-S16-02
- **Authoritative subsystem:** S16 endpoints and connection
- **Title:** Replace the hand-written `index == self.selected_endpoint` branch with `live_of(index)` and `hosts()`
- **Verdict:** recommend
- **Primary evidence:** `gpui/endpoint.rs:152,584-592` (the selected endpoint's state is cloned into `SheprdWindow::live`); `gpui/overview.rs:108-112,130-134,161-165,188-192,300-304`; `gpui/palette.rs:571-575,632-636`; `gpui/palette/activate.rs:194-199`; `gpui/palette/project_open.rs:64-68,240-244`; `gpui/sidebar/render.rs:148-221`; `gpui/sidebar/rail.rs:144-156`; `gpui/sidebar/order.rs:15-27`
- **Interfaces and call sites:** 40 non-test lines match `== self.selected_endpoint` (coordinator grep); about two dozen are this branch. `gpui/endpoint.rs:733-740` reads `self.endpoints[self.selected_endpoint].live` with no branch.
- **Tests and intent evidence:** fixtures copy one state into the other (`gpui/palette.rs:843`, `gpui/chat/preview.rs:403`), which hides the two-copy case; no test pins it
- **Current representation:** two `LiveState` values exist for the selected host. `Endpoint::live` is authoritative only for unselected endpoints.
- **Current complexity or invalid states:** a reader that iterates `self.endpoints` and uses `endpoint.live` compiles, passes single-host tests, and shows stale data for the selected host
- **Why it is material:** three independent reviewers (sidebar, palette and overview, endpoints) hit the same rule; it is restated at every multi-host reader
- **Proposed representation:** `fn live_of(&self, index) -> &LiveState` and `fn hosts(&self) -> impl Iterator<Item = (usize, &Endpoint, &LiveState)>` beside `sync_live` in `endpoint.rs`
- **Why it is simpler:** the "which copy is current" rule lives in one place. This is step one, not the end state: the two `LiveState` copies remain, and the accessor is what makes a later move to single ownership a two-method change
- **Implementation scope:** `endpoint.rs`, the 12 palette and overview sites, the sidebar walkers; about 80 changed lines
- **Smallest credible slice:** add the two methods and convert `overview.rs`
- **Regression risks:** `palette_sources` lists the selected endpoint first; `overview.rs:166-170` and `:193-197` filter `enabled` differently and must keep doing so; loops that mutate `self` need an index loop
- **Migration concerns:** none; no persisted or wire change
- **Existing validation:** `overview/tests/{poll,page,boot_fence,closed}.rs`, `palette/tests.rs`, `palette/project_open/tests.rs`
- **Additional validation required:** a two-endpoint headless test where `endpoints[selected].live` differs from `self.live`, asserting rows come from `self.live`; check whether `endpoint.rs:733-740` is a stale read
- **Impact:** medium-high; removes a stale-read hazard at about two dozen sites
- **Confidence:** medium-high; verified the copy and the grep count, not every site
- **Implementation effort:** small
- **Blast radius:** cross-subsystem (endpoint, sidebar, palette, overview)
- **Prerequisites:** none
- **Priority:** 6

---

#### SIFT-S16-01 · One typed link phase for `Endpoint`

- **Finding ID:** SIFT-S16-01
- **Authoritative subsystem:** S16 endpoints and connection
- **Title:** Replace string-dispatched endpoint state with `enum Phase`
- **Verdict:** recommend
- **Primary evidence:** `gpui/endpoint.rs:140-185,470-505`; `gpui/control/server/app.rs:36-45`; `gpui/devices_overview/model.rs:123-137`; `gpui/endpoint/polling.rs:137-140`; `gpui/endpoint/catalog.rs:219-224,304-311`
- **Interfaces and call sites:** `devices_overview/chart.rs:189-196`, `devices_overview/table.rs:137`, `gpui/group_terminals.rs:60-63`; the `endpoint-status` Fluent selector
- **Tests and intent evidence:** tests assert the strings (`endpoint/lifecycle_tests/remote_drop.rs:13-24,160`, `endpoint/tests.rs:317`, `endpoint/tests/version_mismatch.rs:42`); comments at `endpoint.rs:345-346,496` record that a stopped handle precedes its disconnect state
- **Current representation:** lifecycle spread over `enabled`, `detached`, `connection.handle`, `outage`, `version_mismatch`, `retry_at`, `live.status`, `live.error`. `state()` folds them into `&'static str`; `status()` and `control_hosts` match those strings again; the devices overview derives its own `Link` enum and ignores `outage` and `version_mismatch`.
- **Current complexity or invalid states:** `detached && outage.is_some()`, `!enabled` with a live handle, and `detached` with `live.status != Detached` are representable. The overview can show a dropped host as Online for a poll (inferred, not reproduced). String dispatch on internal state breaks the repo's Rust conventions.
- **Why it is material:** three derivations of one state already disagree
- **Proposed representation:** `enum Phase { Disabled, Detached, Outdated(VersionMismatch), Reconnecting, Connecting, Online }`, `Endpoint::phase()` as the only derivation, `as_str()` for Fluent and CLI words, predicates `is_online()` and `may_dial()`. Second step: fold `detached` and `outage` into one stored `Hold` enum.
- **Why it is simpler:** one derivation, exhaustive matches, no string dispatch
- **Implementation scope:** `endpoint.rs`, `endpoint/polling.rs`, `endpoint/catalog.rs`, `control/server/app.rs`, `devices_overview/model.rs` and its two matchers; about 150 lines. Wire strings and Fluent keys unchanged.
- **Smallest credible slice:** add `Phase` and `phase()`, make `state()` call it
- **Regression risks:** precedence in `state()` (mismatch, then outage, then live) must be kept; `control_hosts` deliberately prefers `self.live` for the selected host; the overview's behaviour for outage and mismatch hosts changes visibly
- **Migration concerns:** none persisted
- **Existing validation:** `endpoint/tests`, `endpoint/lifecycle_tests/{remote_drop,reconnect_backoff,wake_recovery,endpoint_switch}`, `devices_overview/tests/model.rs`
- **Additional validation required:** a table test of `phase()` over flag combinations; a test of the overview link during the handle-stopped window
- **Impact:** high
- **Confidence:** medium-high; verified `state()` and `status()`; the stale-Online window is inferred
- **Implementation effort:** medium
- **Blast radius:** cross-subsystem
- **Prerequisites:** easier after SIFT-S16-02
- **Priority:** 7

---

#### SIFT-S27-01 · Build each Settings row from its catalog entry

- **Finding ID:** SIFT-S27-01
- **Authoritative subsystem:** S27 settings window
- **Title:** Make the catalog entry the row's only declaration so search and page cannot disagree
- **Verdict:** recommend
- **Primary evidence:** `gpui/settings_window/catalog.rs:24-32,80-478`; `gpui/settings_window/pages/tabs.rs:36-60`; `gpui/settings_window/pages/status_bar.rs:92-94`; `gpui/settings_window/pages/notifications.rs:78,165`; `gpui/settings_window/pages/theme.rs:121`; `gpui/settings_window/pages/window.rs:83-85`; `gpui/settings_window/pages/general.rs:87-102,139-202,358-411`; `gpui/settings_window/pages/sidebar.rs:176`
- **Interfaces and call sites:** `gpui/settings_window/search.rs:48-57`; `gpui/settings_window/ui.rs` row builder
- **Tests and intent evidence:** `catalog.rs:5-6` claims the catalog matches the pages; `settings_window/tests/search.rs:64-89` checks only catalog id to rendered row
- **Current representation:** id, section, title and description are declared in the catalog and again in each page's `ui::row(...)`
- **Current complexity or invalid states:** five rows show a different description in search than on the page (`status-usage`, `notif-delay`, `notif-finished`, `theme-mode`, `theme-padding-left`). Seven rendered rows have no catalog entry, so settings search cannot find them. Availability is stated twice.
- **Why it is material:** drift and unsearchable settings exist in the tree today
- **Proposed representation:** pages call one builder taking `&Entry`; a `macro_rules!` table emits named consts and `CATALOG`; pages gate through `self.available(entry.availability, cx)`; dynamic rows keep an explicit uncatalogued builder
- **Why it is simpler:** one declaration per row; search coverage holds by construction
- **Implementation scope:** `catalog.rs`, `ui.rs`, ten `pages/*.rs`, `search.rs`, duplicate description messages in `locales/*/settings*.ftl`
- **Smallest credible slice:** add catalog entries for the seven missing rows and a both-directions test, before changing the builder
- **Regression risks:** someone must choose the winning description per row; hints with arguments need a page-side override; debug selectors must stay stable
- **Migration concerns:** translations drop the losing message ids in every locale
- **Existing validation:** `settings_window/tests/search.rs`, `tests/pages_behaviour.rs`, `tests/en_xa.rs`
- **Additional validation required:** a test that every rendered static row has a catalog entry
- **Impact:** medium-high; user-visible today
- **Confidence:** high; verified the `status-usage` drift
- **Implementation effort:** medium
- **Blast radius:** subsystem plus locale files
- **Prerequisites:** none
- **Priority:** 8

---

#### SIFT-S33-01 · Run the spec validator in the gate

- **Finding ID:** SIFT-S33-01
- **Authoritative subsystem:** S33 developer tooling and spec tools
- **Title:** Add `spec.py check` to `just ci` and CI, and stop committing its stale snapshot
- **Verdict:** recommend
- **Primary evidence:** `docs/spaceport/tools/spec.py:222-306`; `docs/spaceport/evidence/validation.json:5`; `docs/spaceport/plan/tasks.json:3-8`
- **Interfaces and call sites:** no caller in `justfile`, `.github/` or `.githooks/` (coordinator grep)
- **Tests and intent evidence:** `docs/spaceport/tools/test_spec.py`; `tools/README.md:45-52` describes the check as a manual step
- **Current representation:** cross-file ID checks exist but run only by hand; a committed report embeds input hashes
- **Current complexity or invalid states:** `validation.json` says 61 decisions while `plan/decisions.json` holds 69 (verified). Status enums are hard-coded in `spec.py:241,255` beside `status_values` in the data. Evidence paths are checked for non-emptiness only.
- **Why it is material:** this plan is the fork's authoritative scope; its integrity depends on memory
- **Proposed representation:** a `check-spec` recipe in `just ci` and a CI step in the always-run job; stop tracking `validation.json` or regenerate it in the gate; add evidence-path existence and enum checks that read `status_values`
- **Why it is simpler:** one enforced validator instead of a validator plus a stale attestation
- **Implementation scope:** `spec.py` (about 20 lines), `test_spec.py`, `justfile`, one `ci.yml` step, `tools/README.md`, `START_HERE.md:37`
- **Smallest credible slice:** the recipe and CI step with `--skip-schema`
- **Regression risks:** the first gated run may fail on existing drift (the tool was not run in this audit); `scripts/ci-changes.sh:37-44` would skip a docs-only PR unless the step sits in the always-run job
- **Migration concerns:** `GENERATED` in `spec.py:20-23` names `validation.json`
- **Existing validation:** `test_spec.py`
- **Additional validation required:** fixtures for a missing evidence path and an invalid enum value
- **Impact:** medium
- **Confidence:** high; verified the count mismatch and the absent callers
- **Implementation effort:** small
- **Blast radius:** local
- **Prerequisites:** none
- **Priority:** 9

---

#### SIFT-S31-01 · Separate the saved tab schema from the live tab model

- **Finding ID:** SIFT-S31-01
- **Authoritative subsystem:** S31 native tab store (`browser/`)
- **Title:** Keep retired tab kinds in a load-only type so the live `Location` has two variants
- **Verdict:** recommend
- **Primary evidence:** `gpui/browser/location.rs:10-44,46-158,192-275`; `gpui/browser/store.rs:170-214,228-235,372-380`
- **Interfaces and call sites:** dead arms at `browser/store.rs:249,268`, `browser/closed.rs:261`, `browser/view/render.rs:45`, `gpui/control/server/tabs.rs:42-44,56-60`, `control/server/tabs/arrive.rs:132`, `gpui/window/render.rs:685-697`; root errors `InvalidCodeFile`, `InvalidLocalPage`, `InvalidReviewCheckout` (`gpui/error.rs:95-105`)
- **Tests and intent evidence:** decision D05 and `RM-BROWSER` (`docs/spaceport/plan/removals.json:75-90`) removed the embedded browser; `window/render.rs` comment "Retired tabs are dropped on load and never opened"; tests build the impossible states by hand (`control/server/tabs/tests.rs:18-38`, `browser/store/tests/retired_*.rs`)
- **Current representation:** one struct is both the tolerant schema for old files and the live model. Five of `Location`'s seven variants (`Web`, `Local`, `RetiredReview`, `RetiredOrchestrator`, `RetiredSource`) never survive `Store::with_saved`.
- **Current complexity or invalid states:** the type permits a live web, file, review, orchestrator or editor tab; every consumer carries a defensive arm
- **Why it is material:** each new `Location` match must list five dead variants; three root errors and their strings exist for unconstructible states
- **Proposed representation:** private `SavedTab`, `SavedLocation`, `SavedPlace` used only by `parse_saved`, converted with `SavedTab -> Option<Tab>` at the existing filter; live `Location` becomes `Devices | CoreTerminal { panes, observe }`; drop `Tab.place`
- **Why it is simpler:** impossible states become unrepresentable; consumers lose their dead arms
- **Implementation scope:** `browser/store.rs`, `browser/location.rs`, `browser/closed.rs`, `browser/view/render.rs`, `control/server/tabs.rs`, `control/server/tabs/arrive.rs`, `window/render.rs`, optionally `error.rs` and `i18n/errors/root.rs`
- **Smallest credible slice:** introduce the saved types and conversion; remove consumer arms in the same change
- **Regression risks:** a malformed retired payload currently rejects the whole file; unknown `kind` and `place` strings must keep their outcome; discarded IDs must stay reserved
- **Migration concerns:** `browser-tabs.json` format for retained tabs must not change
- **Existing validation:** `browser/store/tests/*`, `browser/location/tests.rs`, `control/server/tabs/tests*`, `browser/closed/tests.rs`
- **Additional validation required:** a round-trip test loading an old file with every retired kind
- **Impact:** medium-high
- **Confidence:** medium-high; verified the seven variants; did not trace every test helper
- **Implementation effort:** medium
- **Blast radius:** cross-subsystem (browser store, control server, window render)
- **Prerequisites:** none; do it before SIFT-S17-01
- **Priority:** 10

---

#### SIFT-S26-01 · Typed items for the footer menu

- **Finding ID:** SIFT-S26-01
- **Authoritative subsystem:** S26 menus and context menus
- **Title:** Replace the footer menu's string-keyed items with one `enum Item`
- **Verdict:** recommend
- **Primary evidence:** `gpui/menu/chrome.rs:97-126,128-171,803-823`; `gpui/menu/whats_new.rs:16-25`
- **Interfaces and call sites:** `gpui/menu/chrome.rs:567-600`; `gpui/menu/chrome/keys.rs:236-248,282-291`
- **Tests and intent evidence:** tests call by string (`menu/announcement_tests.rs:190`, `menu/font_size_tests.rs:21-42`, `menu/workspace_tests.rs:124-126`); debug selectors `menu-{item}` are used by `about/tests.rs:119`
- **Current representation:** `menu_items() -> Vec<&'static str>`, `activate_menu(item: &str)` with `_ => {}`, and a label table falling back to the raw key
- **Current complexity or invalid states:** a typo or a new item missing from one table compiles and silently does nothing or shows its English key. CLAUDE.md forbids internal string dispatch.
- **Why it is material:** four string tables for one item set
- **Proposed representation:** private `enum Item` with `label()`, `selector()` and one exhaustive `match`; order stays hand-written
- **Why it is simpler:** the compiler checks every table
- **Implementation scope:** `menu/chrome.rs`, `menu/chrome/keys.rs`, `menu/whats_new.rs`, three test files
- **Smallest credible slice:** the whole change; it is local
- **Regression risks:** `selector()` must return the same strings; the update-ready versus what's-new distinction must survive
- **Migration concerns:** none
- **Existing validation:** `menu/font_size_tests.rs`, `announcement_tests.rs`, `account_route_tests.rs`, `workspace_tests.rs`, `en_xa_tests.rs`
- **Additional validation required:** a test that every `Item` has a distinct selector
- **Impact:** medium
- **Confidence:** high; verified the string match
- **Implementation effort:** small
- **Blast radius:** local
- **Prerequisites:** none
- **Priority:** 11

---

#### SIFT-S10-01 · Declare each CLI family once

- **Finding ID:** SIFT-S10-01
- **Authoritative subsystem:** S10 control server, client and CLI
- **Title:** One `Family { name, summary, usage, parse }` table
- **Verdict:** recommend
- **Primary evidence:** `gpui/cli/control.rs:93-141,188-192,212-291`; `gpui/control/client/help.rs:13-98,106`
- **Interfaces and call sites:** `gpui/app.rs:370`
- **Tests and intent evidence:** the only help test is `gpui/control/client/tests/tab_help.rs:3-8`; nothing checks that every family has usage text
- **Current representation:** `FAMILIES` holds name and summary; `command()` matches the names as strings with `_ => UnknownOption`; `help::family()` matches them a third time ending `_ => ""`
- **Current complexity or invalid states:** a family without a help arm prints empty usage and exits 0; one without a `command()` arm is recognised then rejected as an unknown option
- **Why it is material:** three string-keyed lists per family, and CLAUDE.md requires `--help` text for every command
- **Proposed representation:** a `Family` struct in one ordered slice; `parse()` calls `family.parse`; usage text sits beside its parser in `cli/control/<family>.rs`
- **Why it is simpler:** a family without a parser or usage cannot compile; both fallback arms go
- **Implementation scope:** `cli/control.rs`, six `cli/control/*.rs` files, `control/client/help.rs`, one line in `app.rs`
- **Smallest credible slice:** move usage into the table first
- **Regression risks:** help text must stay byte-identical; `theme` and `font` share a parser that reads the family name
- **Migration concerns:** none; CLI surface unchanged
- **Existing validation:** `gpui/cli/tests/*`, `control/client/tests/tab_help.rs`
- **Additional validation required:** a test that every entry's usage starts `spaceport <name>` and names are unique
- **Impact:** medium
- **Confidence:** high; verified both fallback arms
- **Implementation effort:** small
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 12

---

#### SIFT-S11-01 · Split the port-scan shell errors from the usage model

- **Finding ID:** SIFT-S11-01
- **Authoritative subsystem:** S11 usage, remote helpers, ports
- **Title:** Give the host shell its own `shell::Error` and delete the provider-era variants
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-usage/src/shell.rs:1-223`; `crates/sheprd-usage/src/error.rs:8-38`; `crates/sheprd-usage/src/lib.rs:108-131`
- **Interfaces and call sites:** sole consumer `gpui/listening_ports.rs:117-146,219-243`; `gpui/i18n/errors/usage.rs:8-38`; `crates/sheprd-gpui/locales/en-US/error-usage.ftl:13-17`; `gpui/control/server/hosts.rs:163-186`
- **Tests and intent evidence:** `Connect`, `NoPlan`, `RateLimited`, `Status`, `Unavailable` are constructed only in tests (coordinator grep found no production construction in the usage, listening-ports or control paths); RM-PROVIDERS keeps "passive metadata/report models"
- **Current representation:** one error enum serves a live host-shell transport and a provider-fetch surface with no producer
- **Current complexity or invalid states:** a failed port scan reports "Usage check timed out." and "Could not reach this host over SSH to read usage."; every matcher covers five unconstructible variants
- **Why it is material:** wrong diagnostics on a live path
- **Proposed representation:** `shell::Error { Process, Size, Timeout, Unreachable, WslHostUnsupported, Client }` with neutral wording and `Shell(#[from] shell::Error)` at the crate root; delete the five provider variants and their Fluent keys
- **Why it is simpler:** each enum describes one operation
- **Implementation scope:** `sheprd-usage/src/{error,shell,lib}.rs`, `gpui/i18n/errors/usage.rs`, `locales/*/error-usage.ftl`, test fixtures
- **Smallest credible slice:** the whole change
- **Regression risks:** Fluent key removal must stay in step with `fl!` checks; `process_step` matches the operation string "start the remote usage shell"
- **Migration concerns:** touches only errors, not `Report` or the registry that RM-PROVIDERS keeps
- **Existing validation:** `sheprd-usage` `shell/tests.rs`, `tests/remote_hosts.rs`; `gpui/listening_ports/tests.rs:192`
- **Additional validation required:** a test that a scan timeout's localized cause chain carries no usage wording
- **Impact:** medium
- **Confidence:** high
- **Implementation effort:** small
- **Blast radius:** cross-subsystem (usage crate, i18n)
- **Prerequisites:** none
- **Priority:** 13

---

#### SIFT-S22-02 · One value for the chat settings menu and its search field

- **Finding ID:** SIFT-S22-02
- **Authoritative subsystem:** S22 rich chat view
- **Title:** `menu: Option<OpenMenu>` carrying its search field
- **Verdict:** recommend
- **Primary evidence:** `gpui/chat/chats.rs:140-143,326-334`; `gpui/chat/view/settings.rs:198-214,613-618`; `gpui/chat/view.rs:522-527`
- **Interfaces and call sites:** `gpui/chat/window.rs:319,470-478,551-557`; `gpui/chat/view/composer.rs:293-295`
- **Tests and intent evidence:** `chat/tests/menus.rs`
- **Current representation:** `menu: Option<Menu>` and `menu_search: Option<(Entity<SearchInput>, Subscription)>` written as a pair
- **Current complexity or invalid states:** `fail` clears `menu` only (verified). A search field can exist with no menu; if it still holds focus, keys route to a menu that is not drawn (read, not run).
- **Why it is material:** a reachable contradictory state after a connection failure
- **Proposed representation:** `OpenMenu { kind: Menu, search: Option<(Entity<SearchInput>, Subscription)> }`
- **Why it is simpler:** closing drops both
- **Implementation scope:** `chats.rs`, `view/settings.rs`, `view.rs`, `view/composer.rs`, `chat/window.rs`
- **Smallest credible slice:** the whole change
- **Regression risks:** focus must return to the composer on every close path; `fail` has no window handle
- **Migration concerns:** none
- **Existing validation:** `chat/tests/menus.rs`
- **Additional validation required:** a connection failure with search open leaves no search field and keys reach the composer
- **Impact:** medium
- **Confidence:** medium-high; verified the asymmetric clear
- **Implementation effort:** small
- **Blast radius:** local
- **Prerequisites:** none
- **Priority:** 14

---

#### SIFT-S21-01 · One owner for local operation failures

- **Finding ID:** SIFT-S21-01
- **Authoritative subsystem:** S21 window shell
- **Title:** Replace the sticky `local_error: Option<String>` with one reporting entry point and a defined lifetime
- **Verdict:** recommend
- **Primary evidence:** `gpui/window.rs:208`; `gpui/window/status_bar.rs:38-41`; `gpui/state.rs:490-504`; `gpui/window/flash.rs:106-151`; `gpui/window/commands.rs:262,296,493,530-536,577-583,666-670`
- **Interfaces and call sites:** 61 production writes in 30 files (coordinator grep). Four plain clears, none tied to the failed operation: `gpui/endpoint.rs:603`, `gpui/endpoint/polling.rs:181`, `gpui/menu/settings.rs:249`, `gpui/menu/workspace/requests.rs:210`. `gpui/window/pending_input.rs:409,415` also `take()` the error and restore it conditionally (found by the findings critic). `gpui/control/server/hosts.rs:143-151` diffs the string to recover a failure reason.
- **Tests and intent evidence:** `endpoint/lifecycle_tests/held_paste.rs:152,191` assert some messages remain; about 90 test assertions read the field
- **Current representation:** a last-write-wins string beside a second channel, the self-retiring flash, with about 62 call sites
- **Current complexity or invalid states:** no cause and no expiry: "clear needs a newer daemon" stays beside "Connected" until an unrelated reconnect. The same class of failure flashes at one site and sticks at another. The control server cannot tell an identical repeat failure from no failure.
- **Why it is material:** user-visible stale errors, and a display string used as a return value
- **Proposed representation:** `fn report(&mut self, failure: LocalFailure, cx)` with `Kind { Declined, Condition }`. `Declined` retires through the flash path; `Condition` stays until its producer clears it. Endpoint selection returns `Result<(), crate::Error>`.
- **Why it is simpler:** the lifetime rule lives in one file instead of 30
- **Implementation scope:** a reporting module beside `window/flash.rs`, the 61 writers converted mechanically, `state.rs:490`, the selection return type
- **Smallest credible slice:** typed selection result for `control/server/hosts.rs`, removing the string diff
- **Regression risks:** some sticky messages are deliberate; classify per site. The status bar message is an accessibility label; a flash is announced differently. The disconnection-reason rule in CLAUDE.md must keep holding.
- **Migration concerns:** tests need an accessor
- **Existing validation:** `endpoint/lifecycle_tests/*`, `window/tests/report_issue.rs`, `window/flash/tests.rs`, `control/server/hosts/tests/info.rs`
- **Additional validation required:** a `Declined` failure retires without a reconnect; a `Condition` clears when its producer recovers; `host select` returns the typed reason on a repeat failure
- **Impact:** medium-high
- **Confidence:** medium; writer count verified, clear sites corrected once by the critic, the per-site split is unverified
- **Implementation effort:** medium
- **Blast radius:** application-wide (30 files, one line each)
- **Prerequisites:** none
- **Priority:** 15

---

#### SIFT-S04-01 · Shrink the `Backend` abstraction to the seam that is substituted

- **Finding ID:** SIFT-S04-01
- **Authoritative subsystem:** S04 sheprd-backend and sheprd-core-client
- **Title:** Keep `Terminal` and attach; delete `Generation`, `Changes` and the unused `Backend` methods
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-backend/src/lib.rs:22-62,80-86`; `crates/sheprd-backend/README.md:9-14`; `crates/sheprd-core-client/src/backend.rs:40-99,126-184`
- **Interfaces and call sites:** the only generic consumer is `gpui/core_terminals/worker.rs:92-149` (attach plus four `Terminal` methods). Others name `CoreBackend` and call `backend.core()`: `gpui/core_terminals/hub.rs:27-28,44,88`, `observation.rs:551-643`, `workspace_ui.rs:248,377,773`.
- **Tests and intent evidence:** the test double `gpui/core_terminals/tests/worker.rs:29-70` must stub `NoChanges`, `generation`, `terminals`, `watch`, `spawn`, `kill` to test attach alone
- **Current representation:** a six-method trait with three associated types, promising generation independence
- **Current complexity or invalid states:** one production implementation (verified). `sheprd-client` does not depend on `sheprd-backend`, so the README states a false architecture fact. `generation()` and `Changes::next` have no production caller; `watch` has two contracts.
- **Why it is material:** the abstraction misdescribes the architecture and forces test stubs
- **Proposed representation:** keep the shared value types and `Terminal`; reduce `Backend` to `type Terminal`, `type Error`, `attach`, or pass an attach closure; make `terminals`, `watch`, `spawn`, `kill` inherent on `CoreBackend`
- **Why it is simpler:** the trait states only what is substituted
- **Implementation scope:** `sheprd-backend/src/lib.rs` and README, `sheprd-core-client/src/backend.rs`, `sheprd-core-client/src/tests/backend.rs`, four `Backend as _` imports in gpui, the test double, the CLAUDE.md architecture row; check `crates/sheprd-terminal` too, which also depends on `sheprd-backend` (`crates/sheprd-terminal/Cargo.toml:14`)
- **Smallest credible slice:** delete `Generation` and `Changes::next`
- **Regression risks:** parallel PRs importing `Backend as _` conflict; upstream merges may reintroduce the wider trait
- **Migration concerns:** confirm no Herdr 1 implementation is planned in `docs/spaceport`
- **Existing validation:** `sheprd-core-client` tests, `gpui/core_terminals/tests/worker.rs`, `just test-core`
- **Additional validation required:** default and all-feature builds
- **Impact:** medium
- **Confidence:** medium-high; verified the single implementation and dependency list
- **Implementation effort:** small
- **Blast radius:** cross-subsystem (two crates and gpui imports)
- **Prerequisites:** none
- **Priority:** 16

---

#### SIFT-S17-01 · The tab `Store` as sole owner of a Core tab's pane tree

- **Finding ID:** SIFT-S17-01
- **Authoritative subsystem:** S17 Herdr Core terminals
- **Title:** Remove the per-window copy of the pane tree and its one-way dirty flag
- **Verdict:** recommend
- **Primary evidence:** `gpui/browser/store.rs:285-293`; `gpui/core_terminals.rs:238-259,583-585`; `gpui/core_terminals/tabs.rs:47-51,177-182,220-230,258-276`; `gpui/core_terminals/commands.rs:81-85`; `gpui/core_terminals/split.rs:319-326`
- **Interfaces and call sites:** Store readers `gpui/core_terminals/sidebar.rs:26-34`, `core_terminals/workspace_ui.rs:422-431`, `gpui/control/server/tabs.rs:62`; window-copy readers `split.rs:49-65`, `commands.rs:34`
- **Tests and intent evidence:** `core_terminals/tests/panes.rs`, `workspace_ui/tests`; no two-window test
- **Current representation:** an app-global saved tree plus a per-window `TabPanes { panes, observe, unsaved }` seeded once and written back when `unsaved` is set (verified)
- **Current complexity or invalid states:** sidebar rows and control output can disagree with what is drawn until the next save. Across two windows on one workspace, a split in window A never reaches window B, and B's next resize writes its stale tree over A's (inferred from code, not run).
- **Why it is material:** possible loss of a pane from the saved layout while it still runs in Core
- **Proposed representation:** `TabPanes` keeps only view state (`active`, `zoom`, `zoomed`); structural edits go through `Store::update`; an in-flight drag share stays local as an overlay committed on drop
- **Why it is simpler:** one tree; no dirty flag, no seed-once clone, no `save_core_panes`
- **Implementation scope:** `core_terminals.rs`, `core_terminals/{tabs,commands,split,sidebar}.rs`, one `Store` method
- **Smallest credible slice:** the two-window test first, to prove or dismiss the overwrite
- **Regression risks:** each structural edit now saves; confirm the state-file writer coalesces; `active` and `zoomed` need re-validation when another window removes a pane
- **Migration concerns:** saved format unchanged
- **Existing validation:** `core_terminals/tests/panes.rs`, `tests/worker.rs`, `workspace_ui/tests`
- **Additional validation required:** two windows, split in one, resize in the other, saved tree keeps both panes
- **Impact:** medium; Core is an opt-in experiment
- **Confidence:** medium; verified the `unsaved` flag sites; the overwrite is inferred
- **Implementation effort:** medium
- **Blast radius:** cross-subsystem (core terminals and tab store)
- **Prerequisites:** SIFT-S31-01 first; both change `browser/store.rs`
- **Priority:** 17

---

#### SIFT-S14-01 · One typed build channel

- **Finding ID:** SIFT-S14-01
- **Authoritative subsystem:** S14 build identity and test harnesses
- **Title:** Emit a build channel from the build script and stop re-deriving "preview build" from identity strings
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-gpui/preview_identity.rs:4-20`; `crates/sheprd-gpui/build.rs:79-85`; `gpui/updates.rs:11-16`; `gpui/updates/policy.rs:24-27`; `gpui/constants.rs:16-19,35`; `crates/sheprd-gpui/windows_resource.rs:14-19,73-90`; `crates/sheprd-gpui/release_version.rs:7-19`
- **Interfaces and call sites:** `RELEASE_BUILD` gates `gpui/agent_skill.rs:210-211` and `gpui/window/report_issue.rs:28,54`
- **Tests and intent evidence:** `gpui/constants/tests.rs:9-16`; `crates/sheprd-gpui/tests/preview_identity.rs`, `tests/build_identity.rs`, `tests/removed_updater.rs`; decision D60
- **Current representation:** the channel is implicit in `(app_id, version)`; consumers compare against the literal `"ai.2389.spaceport"` and `"dev"`
- **Current complexity or invalid states:** two code sites and one test each rebuild the decision (verified literals at `updates.rs:14` and `policy.rs:25`). Editing `macos-preview-app-id` would silently disable updates. `RELEASE_BUILD` is always false because the build refuses `HERDR_RELEASE_VERSION`, yet it still gates code. Two version parsers differ on leading zeros.
- **Why it is material:** identity decided in several places on the update path
- **Proposed representation:** `preview_identity::select` returns a channel; `build.rs` emits `SPACEPORT_BUILD_CHANNEL`; `constants.rs` parses it into `enum Channel { Development, MacosPreview }`
- **Why it is simpler:** one decision, one enum, no string comparison
- **Implementation scope:** `preview_identity.rs`, `build.rs`, `constants.rs` and its tests, `updates.rs`, `updates/policy.rs` and tests. This absorbs the former SIFT-S29-02: delete `updates::supported_build()` so `policy::eligible` plus the installed global is the only answer to "can this build update" (`gpui/menus.rs:51,126`, `gpui/control/server/app.rs:369-401`)
- **Smallest credible slice:** emit and parse the channel; convert `updates.rs`
- **Regression risks:** `build.rs` must stay dependency-free; D60's release refusal must remain; `policy::eligible` must still reject a bundle whose id differs from the built id
- **Migration concerns:** removing `RELEASE_BUILD` consumers belongs to their owners
- **Existing validation:** the four test files above, `updates/tests/policy.rs`, `tests/sparkle_lifecycle.rs`
- **Additional validation required:** a test that a development channel is ineligible whatever the id string
- **Impact:** medium
- **Confidence:** medium; verified the literals and `RELEASE_BUILD` uses
- **Implementation effort:** small
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 18

---

#### SIFT-S15-01 · One checked writer for settings saves

- **Finding ID:** SIFT-S15-01
- **Authoritative subsystem:** S15 configuration and persisted settings
- **Title:** Route Settings-window saves through the dotted-key editor that re-parses before writing
- **Verdict:** recommend
- **Primary evidence:** `gpui/config/preferences.rs:8-74,103-342`; `gpui/config/edit.rs:79-186`; `gpui/config/notifications.rs:81-88,210-219`; `gpui/config/sidebar_style.rs:39-52,85-91`; `gpui/config/status_bar.rs:27-67,110-121`
- **Interfaces and call sites:** 120 `Preference::` uses under `gpui/settings_window/`; `Config::set_key` and `unset_key` are already used by `gpui/control/client/settings/config.rs:29-35` and `gpui/settings_window/controls/chat_env.rs:70-76`
- **Tests and intent evidence:** `config/tests/sidebar_settings.rs:428` expects `InvalidSidebarMetric`; `config/preferences/tests.rs`
- **Current representation:** 45 `Preference` variants each mapped by hand to table, key and TOML value, beside a generic set/unset that proves the file still loads
- **Current complexity or invalid states:** bounds and enum spellings are restated beside the parser's (verified for the 3600-second delay). A variant without a hand check writes unvalidated. A renamed serde variant drifts silently.
- **Why it is material:** two writers for one file with different guarantees
- **Proposed representation:** each `Preference` reduces to a key path and `Option<toml_edit::Value>` and calls the body of `edit_key_at`; derive `Serialize` on config enums so the written spelling is the parsed one
- **Why it is simpler:** one writer, validation by re-parse, no restated bounds
- **Implementation scope:** `config/preferences.rs`, `config/edit.rs`, small derives in seven `config/*.rs` files
- **Smallest credible slice:** a round-trip test that every `Preference` value writes text `Config::parse` accepts, before changing the writer
- **Regression risks:** host names containing `.` must travel as path segments; `edit_key_at` refuses when the file already fails to load; typed error variants change for delay and volume
- **Migration concerns:** keep `TerminalPadding`'s `layout.sidebar_gap` removal as an explicit pre-step
- **Existing validation:** `config/preferences/tests.rs`, `config/tests/{preferences,edit,sound_volume,notification_settings,sidebar_settings,terminal_padding}.rs`
- **Additional validation required:** the round-trip test above
- **Impact:** medium
- **Confidence:** medium-high; verified both writers exist and one restated bound
- **Implementation effort:** medium
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 19

---

#### SIFT-S20-01 · One key-to-edit mapping for the three text fields

- **Finding ID:** SIFT-S20-01
- **Authoritative subsystem:** S20 window input, selection and text fields
- **Title:** A pure `EditCommand` mapper in `text_editing.rs`, with `DialogInput` on the shared `Editing` model
- **Verdict:** recommend
- **Primary evidence:** `gpui/text_editing.rs:1-4,53-62,114-144`; `gpui/search_input.rs:249-375`; `gpui/search_input/control_keys.rs:13-77`; `gpui/multiline_input/keys.rs:89-299`; `gpui/dialog_input.rs:11-15,44-84,91-204,225-234`
- **Interfaces and call sites:** `gpui/window/input.rs:86-124,144-183,202-229`; `gpui/menu/chrome/keys.rs:26,56`
- **Tests and intent evidence:** `search_input/tests/{word_keys,control_keys,composition,rtl_caret}`, `dialog_input/tests*`; `MultilineInput` has no key tests
- **Current representation:** `text_editing.rs` calls itself the shared editing model, but `DialogInput` keeps its own (`selection` plus `reversed`), and three near-identical key tables exist
- **Current complexity or invalid states:** the tables have drifted: `DialogInput` lacks the Ctrl-A/E/B/F/H/D/K/Y keys and resolves modifiers in a different order; it strips all control characters while `Editing` strips only CR and LF. `selection` plus `reversed` can express a reversed empty range.
- **Why it is material:** the same keystroke behaves differently in different fields
- **Proposed representation:** a pure function mapping key, modifiers and `&Editing` to `Option<EditCommand>` (`Move`, `Delete`, `SelectAll`, `Copy`, `Cut`, `Paste`, `Kill`, `Yank`); each widget applies it; `DialogInput` holds an `Editing`
- **Why it is simpler:** one table; the shared model becomes actually shared
- **Implementation scope:** `text_editing.rs`, `search_input.rs`, `search_input/control_keys.rs`, `multiline_input/keys.rs`, `dialog_input.rs`
- **Smallest credible slice:** the mapper plus `SearchInput` and `MultilineInput`; `DialogInput` second
- **Regression risks:** modifier precedence; a masked field must never fill the kill buffer or clipboard; composing fields must keep returning early; dialogs gaining Ctrl keys is a behaviour change
- **Migration concerns:** none
- **Existing validation:** the search and dialog input tests above
- **Additional validation required:** table tests for the mapper; key tests for `MultilineInput`
- **Impact:** medium
- **Confidence:** high for the duplication (reviewer-read in full); `from_utf16` duplicate not confirmed by coordinator grep, which found only the `text_editing.rs` definition
- **Implementation effort:** medium
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 20

---

#### SIFT-S26-02 · Nest popup pages under their family

- **Finding ID:** SIFT-S26-02
- **Authoritative subsystem:** S26 menus and context menus
- **Title:** `Page::Pane(PanePage)`, `Page::Tab(TabPage)`, `Page::Host(HostPage)` with exhaustive render and key routing
- **Verdict:** recommend
- **Primary evidence:** `gpui/menu/page.rs:7-65,97-127`; `gpui/menu/chrome.rs:401-416,567-667`; `gpui/menu/chrome/keys.rs:68-152`; `gpui/menu/page_title.rs:15-25`; `gpui/menu/state.rs:38-114,184-218`
- **Interfaces and call sites:** `gpui/pane_menu.rs:343-349,589,658-660`; `gpui/group_menu.rs:185,237-243`; `gpui/tab_menu.rs:256`; `gpui/menu/settings.rs:112-119`
- **Tests and intent evidence:** `pane_menu/tests`, `tab_menu/tests`, `group_menu/tests`, `menu/appear/tests.rs`
- **Current representation:** a flat `Page` of about 38 variants; family membership restated in five places; rendering is an `if / else if` chain
- **Current complexity or invalid states:** a new page missing from the render chain draws release notes. `page == Pane` with `menu.pane == None` is representable and about 37 accesses guard against it. `reset()` omits `keybinds_search` and its subscription, so they outlive the page.
- **Why it is material:** silent fallthrough and a leaked subscription
- **Proposed representation:** nested family enums, `Page` stays `Copy`; payloads stay beside it
- **Why it is simpler:** one exhaustive match for render and one for keys
- **Implementation scope:** `menu/page.rs`, `menu/page_title.rs`, `menu/chrome.rs`, `menu/chrome/keys.rs`, `pane_menu*`, `tab_menu.rs`, `menu/devices/host_menu*`, about 66 test references
- **Smallest credible slice:** clear the two keybinds fields in `reset()`; then replace the render chain's final `else` with an explicit arm
- **Regression risks:** input isolation order in `menu_key_down`; open PRs adding pages will conflict
- **Migration concerns:** nested enums stay keep-sorted
- **Existing validation:** the menu tests above
- **Additional validation required:** every page renders its own content
- **Impact:** medium
- **Confidence:** medium; wide change
- **Implementation effort:** small for the `reset()` fix, large for the nesting
- **Blast radius:** cross-subsystem
- **Prerequisites:** none. The `reset()` leak fix is a standalone slice and should ship before the nesting.
- **Priority:** 21

---

#### SIFT-S17-02 · One definition of the exact-Core fence

- **Finding ID:** SIFT-S17-02
- **Authoritative subsystem:** S17 Herdr Core terminals
- **Title:** `Hub::backend_for(core_id)` and two `Projection` helpers
- **Verdict:** recommend
- **Primary evidence:** `gpui/core_terminals.rs:64-69,435-438,722-727`; `gpui/core_terminals/workspace_ui.rs:211,247-249,331-341,374-379,392-402,415,474-485,505-515,533-543,601-617,727-740`; `gpui/core_terminals/tabs.rs:81-83`
- **Interfaces and call sites:** `gpui/chat/core_pane.rs:296`
- **Tests and intent evidence:** `hub/tests/target.rs`, `workspace_ui/tests/hub_target.rs`, `tests/spawn_identity.rs`; decision D58 ("keep exact runtime targets")
- **Current representation:** "backend is this Core and alive" written four ways
- **Current complexity or invalid states:** one site uses `is_none_or` (missing backend refuses), one `is_some_and` (missing backend passes), one omits the lost check. A new entry point can pick the wrong variant unnoticed.
- **Why it is material:** the fence enforces a safety decision
- **Proposed representation:** `backend_for(core_id) -> Result<&CoreBackend, Mismatch { Absent, Lost, Other }>`, `Projection::group_key(id)`, `Projection::observes(core_id, terminal_id)`, `Access::row(target)`
- **Why it is simpler:** "absent is fine for reading" becomes a visible choice
- **Implementation scope:** `core_terminals.rs`, `core_terminals/{hub,observation,workspace_ui,tabs}.rs`
- **Smallest credible slice:** `backend_for` and its four callers
- **Regression risks:** the differing variants may be deliberate; record each site's behaviour first
- **Migration concerns:** none
- **Existing validation:** the three test areas above
- **Additional validation required:** a table test over absent, lost, other and match
- **Impact:** medium
- **Confidence:** medium-high
- **Implementation effort:** small
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 22

---

#### SIFT-S02B-01 · One endpoint response-envelope decoder

- **Finding ID:** SIFT-S02B-01
- **Authoritative subsystem:** S02b sheprd-client data paths
- **Title:** Move the envelope decoder out of scrollback and stop hand-parsing `error` and `result` at each call site
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-client/src/scrollback.rs:152-186,242-262`; `crates/sheprd-client/src/error.rs:262-275`
- **Interfaces and call sites:** null-tolerant copies: `gpui/integrations.rs:56-68`, `gpui/links.rs:255-260,316-325`, `gpui/window/tab_start.rs:10-16`, `gpui/state.rs:653-683`, `crates/sheprd-client/src/session.rs:804`. Any-key-fails copies: `gpui/menu/devices/add_device.rs:828`, `gpui/menu/workspace/requests.rs:71`, `gpui/menu/worktree_open.rs:141`, `gpui/palette/project_open.rs:282`, `gpui/smoke.rs:224`. Error-body readers: `gpui/menu.rs:96-101`, `gpui/daemon/error.rs:13,25-33`.
- **Tests and intent evidence:** `crates/sheprd-client/src/scrollback/tests.rs`
- **Current representation:** one typed decoder used only for scrollback; about 14 hand-rolled copies elsewhere
- **Current complexity or invalid states:** `"error": null` is success on eight paths and failure on five. Each site re-checks `result["type"]` by string. Bounding and control-stripping of the daemon's message happens at one site.
- **Why it is material:** one wire contract decoded in two incompatible conventions
- **Proposed representation:** `sheprd-client/src/response.rs` with `response::result(&Value) -> Result<&Value>` and `response::typed::<T>(&Value, kind) -> Result<T>`; unknown codes stay `Other(String)` so the envelope stays open-ended
- **Why it is simpler:** one decoder, one null rule, one error shape
- **Implementation scope:** new `response.rs` and tests; `scrollback.rs` loses about 40 lines; gpui sites migrate one at a time
- **Smallest credible slice:** the module plus the five any-key-fails sites
- **Regression risks:** display text changes at unfiltered sites; `daemon/error.rs:29` tolerates a bare-string error, so keep a lenient fallback; `links.rs:255-262` deliberately swallows failures
- **Migration concerns:** each site is independent
- **Existing validation:** scrollback tests; gpui tests for integrations, links, project_open, tab_start
- **Additional validation required:** a table test over absent, null, object and string errors, missing result and wrong type tag
- **Impact:** medium
- **Confidence:** medium; the divergence is plain, but Herdr emitting `"error": null` is unconfirmed
- **Implementation effort:** medium
- **Blast radius:** cross-subsystem
- **Prerequisites:** SIFT-S25-01, which deletes one of the call sites (`add_device.rs:828`)
- **Priority:** 23

---

#### SIFT-S24-01 · One listing rule for the sidebar, and no duplicate fold set

- **Finding ID:** SIFT-S24-01
- **Authoritative subsystem:** S24 sidebar
- **Title:** Drop the window's duplicate `collapsed_repos` and settle one listing rule for painting and stepping
- **Verdict:** recommend
- **Primary evidence:** `gpui/sidebar/render.rs:134,148-221,263-266`; `gpui/sidebar/rail.rs:144-156,180-270,387`; `gpui/sidebar/order.rs:15-70`; `gpui/sidebar/search.rs:265-273`; `gpui/sidebar/device_drag.rs:164-173`; `gpui/window.rs:139`; `gpui/endpoint.rs:151`
- **Interfaces and call sites:** `gpui/menu/workspace.rs:506-520`; `gpui/sidebar/host_row.rs:309-313`; `gpui/sidebar/render/agent_rows.rs:62`
- **Tests and intent evidence:** `sidebar/order.rs:57-58` says painting and stepping read one list; `rail/tests.rs`, `layout_tests`
- **Current representation:** three walkers pair `displayed_devices()` with visibility, the live-state pick and a collapsed-repos pick
- **Current complexity or invalid states:** filters have drifted: `order.rs:18` requires `enabled`, the painters do not; tag-folded hosts are skipped under different conditions; stepping can land on rows that are not painted. The Local fold set lives on the window and on `Endpoint` (verified both fields), chosen by three different predicates.
- **Why it is material:** keyboard stepping and painting disagree
- **Proposed representation:** delete `SheprdWindow::collapsed_repos`. Do not add a second iterator: the walkers consume `hosts()` from SIFT-S16-02 and share one `folded` and agent-visibility predicate in `sidebar/order.rs`
- **Why it is simpler:** one definition of what is listed
- **Implementation scope:** `sidebar/{order,render,rail,search,device_drag,host_row}.rs`, `render/agent_rows.rs`, `menu/workspace.rs`, `window.rs`, `window/construct.rs`
- **Smallest credible slice:** the `collapsed_repos` removal alone
- **Regression risks:** unifying filters changes stepping in rail mode and for disabled or tag-folded hosts; decide the intended rule first
- **Migration concerns:** none persisted
- **Existing validation:** `sidebar/order.rs` tests, `rail/tests.rs`, `layout_tests`, `smoke_hosts`
- **Additional validation required:** stepping lands only on painted rows, with a collapsed host, a tag-folded section and a disabled endpoint
- **Impact:** medium
- **Confidence:** high for `collapsed_repos`; medium for the shared walk
- **Implementation effort:** medium
- **Blast radius:** cross-subsystem (sidebar, menu, window)
- **Prerequisites:** SIFT-S16-02, which supplies the one host iterator
- **Priority:** 24

---

#### SIFT-S23-02 · One connection phase for a chat pane

- **Finding ID:** SIFT-S23-02
- **Authoritative subsystem:** S23 chat lifecycle
- **Title:** `enum Phase { Due, Linked, Backoff, Ended }` and removal of the dead generation tag
- **Verdict:** recommend
- **Primary evidence:** `gpui/chat/chats.rs:129-153,280-290,313-323,351-360,662-663`
- **Interfaces and call sites:** `gpui/chat/terminal/resize.rs:9-19` is the one real reader of `generation()`; `gpui/chat/link.rs:232-236`
- **Tests and intent evidence:** `chat/tests/chats.rs` covers resume, backoff, give-up and version refusal
- **Current representation:** `failures`, `generation`, `link: Option<(u64, Link)>`, `permanent`, `preview`, `retry_at`
- **Current complexity or invalid states:** a link with a pending retry, a permanent pane with a retry time and a preview with a link are representable. The generation tag on the link is written straight after the only increment, so the mismatch branch in `poll` cannot run (verified).
- **Why it is material:** a fence that looks load-bearing and is dead
- **Proposed representation:** `enum Phase { Due, Linked(Link), Backoff(Instant), Ended }` with a test-only `Preview` variant; `generation` stays a plain counter
- **Why it is simpler:** exclusive states are exclusive; three predicates become matches
- **Implementation scope:** `chat/chats.rs`, about 60 lines
- **Smallest credible slice:** delete the dead branch and the tag
- **Regression risks:** `fail` must still close menus; `available()` must keep showing the transcript during backoff
- **Migration concerns:** none
- **Existing validation:** `chat/tests/chats.rs`, `chat/tests/session.rs`, `chat/tests/terminal_input/resize.rs`
- **Additional validation required:** a test that a pane in backoff still shows its transcript and reconnects exactly once when the retry time passes
- **Impact:** medium-low
- **Confidence:** medium-high
- **Implementation effort:** small
- **Blast radius:** local
- **Prerequisites:** none
- **Priority:** 25

---

#### SIFT-S19-01 · One phase enum for the tab drag

- **Finding ID:** SIFT-S19-01
- **Authoritative subsystem:** S19 tab strip and tab gestures
- **Title:** `enum Phase { Pressed, Carried, Settling }` in `TabDrag`
- **Verdict:** recommend
- **Primary evidence:** `gpui/window/tab_drag.rs:47-78,161-180,251-254,294,310,323,340-348,410-416,430,461-479,532-546`
- **Interfaces and call sites:** `gpui/window/tab_ghost.rs:96-103,137-143`; `gpui/window/tab_float.rs:148`
- **Tests and intent evidence:** `window/tab_strip/tests/tear_off.rs`, `tests/group_drop.rs`
- **Current representation:** `lifted: Option<Instant>`, `dropped: Option<Vec<String>>`, `outside: bool`, `target`, `float`, one reused timer
- **Current complexity or invalid states:** `dropped` without `lifted`, `outside` while settling and `target` before lift are representable. Guards disagree: `move_tab_drag` ignores a settling drag but `release_tab_drag` takes and drops it, so a mouse-up during the two-second settle appears to end the preview early (read, not run).
- **Why it is material:** three phases spread over flags, with a likely early-cancel defect
- **Proposed representation:** the phase enum with per-phase fields; each transition is one assignment
- **Why it is simpler:** meaningless combinations cannot be built
- **Implementation scope:** `window/tab_drag.rs` and three reads in `tab_float.rs` and `tab_ghost.rs`
- **Smallest credible slice:** the whole change
- **Regression risks:** settling still needs `target`; the macOS cfg gate on `float`
- **Migration concerns:** none
- **Existing validation:** the tab strip tests above
- **Additional validation required:** a mouse-up during settling keeps the preview, or pins current behaviour if intended
- **Impact:** medium
- **Confidence:** medium
- **Implementation effort:** small
- **Blast radius:** local
- **Prerequisites:** none
- **Priority:** 26

---

#### SIFT-S24-02 · One sidebar gesture

- **Finding ID:** SIFT-S24-02
- **Authoritative subsystem:** S24 sidebar
- **Title:** `sidebar_gesture: Option<Gesture>` in place of three drag fields and twin lift code
- **Verdict:** recommend
- **Primary evidence:** `gpui/window.rs:156,265,332`; `gpui/sidebar/reorder.rs:30-96,285-415`; `gpui/sidebar/device_drag.rs:18-36,126,134-159,280-346`
- **Interfaces and call sites:** `gpui/window/double_shift.rs:85-88`; `gpui/window/input.rs:383-384`; `gpui/sidebar/render.rs:871-891,935-939`
- **Tests and intent evidence:** `sidebar/reorder/tests.rs`, `sidebar/device_drag/tests.rs`
- **Current representation:** `device_drag`, `sidebar_drag`, `workspace_drag` as independent fields; press, lift and move are near-copies
- **Current complexity or invalid states:** two drags can be armed at once. The copies drifted: cancelling a device drag forgets an unlifted press so its timer cannot lift it later; cancelling a workspace drag leaves it armed.
- **Why it is material:** exclusivity enforced by scattered guards
- **Proposed representation:** `Gesture { Workspace, Device, Resize }` with a shared `Lift { origin, pointer, phase }`
- **Why it is simpler:** one gesture at a time by type
- **Implementation scope:** `sidebar/{reorder,device_drag,hover,host_row,render}.rs`, `window.rs`, `window/{construct,double_shift,input}.rs`
- **Smallest credible slice:** fix the workspace cancel drift with a test
- **Regression risks:** the click-versus-drag return value; the dropped preview must last until the daemon's reorder arrives
- **Migration concerns:** none
- **Existing validation:** the two test files above, `layout_tests` drag cases
- **Additional validation required:** Escape during an unlifted workspace press must not lift later
- **Impact:** medium-low
- **Confidence:** medium
- **Implementation effort:** medium
- **Blast radius:** cross-subsystem (sidebar, window)
- **Prerequisites:** none; the cancel-drift fix stands alone
- **Priority:** 27

---

#### SIFT-G7-01 · One "follows Herdr" predicate for themes

- **Finding ID:** SIFT-G7-01
- **Authoritative subsystem:** G7 themes, fonts and config loaders
- **Title:** One exported constant and `ThemeName::follows_herdr(value, light)`
- **Verdict:** recommend
- **Primary evidence:** `gpui/config/theme.rs:14-18,219-227`; `gpui/settings_window/themes.rs:18,356,417,452,512`; `gpui/settings_panel.rs:11,70-103`; `gpui/menu/settings.rs:149,223`; `gpui/preferences.rs:37`; `gpui/settings_window/persistence.rs:213,221`
- **Interfaces and call sites:** `gpui/settings_window.rs:424-426`; `gpui/settings_window/pages/theme.rs:173-186`; `gpui/settings_window/themes/system.rs:51`
- **Tests and intent evidence:** `config/theme.rs:16-18` says either side of a light and dark pair may itself be `Follow Herdr`; `settings_window/themes/tests*`
- **Current representation:** the sentinel string is defined three times and written as a raw literal at five more sites; some sites test one side, others the whole string
- **Current complexity or invalid states:** for `light:Follow Herdr,dark:X`, the theme browser treats the light side as following while the shared-theme re-apply path does not, so a changed daemon theme would not be re-applied for that side (read from code, not run)
- **Why it is material:** two answers to one question, with a probable mixed-pair defect
- **Proposed representation:** one constant in `config/theme.rs` and one predicate beside `side` and `follows_system`
- **Why it is simpler:** one definition, about 20 call sites reading it
- **Implementation scope:** `config/theme.rs`, `settings_panel.rs`, `settings_window.rs`, `settings_window/{themes.rs,themes/system.rs,pages/theme.rs,persistence.rs}`, `menu/settings.rs`, `preferences.rs`
- **Smallest credible slice:** export the constant and replace the literals, with no behaviour change
- **Regression risks:** whole-string sites change behaviour for mixed pairs; the Follow toggle and the "already shows the daemon's theme" shortcut each need a per-site decision
- **Migration concerns:** no config format change
- **Existing validation:** settings-window theme tests, config theme tests
- **Additional validation required:** a mixed-pair config picks up a changed shared theme on the light side only
- **Impact:** low-medium
- **Confidence:** medium
- **Implementation effort:** small
- **Blast radius:** cross-subsystem (config, settings window, menu)
- **Prerequisites:** none
- **Priority:** 28

---

#### SIFT-S21-02 · One captured connection fence for deferred actions

- **Finding ID:** SIFT-S21-02
- **Authoritative subsystem:** S21 window shell
- **Title:** A `ConnectionFence` value and one `fence_holds` predicate for work that outlives the connection it started on
- **Verdict:** recommend
- **Primary evidence:** `gpui/window/mouse.rs:28-40,118-127,284-302,378-384`; `gpui/window/transfers.rs:18-30,104-112,334-340`; `gpui/window/wsl_drop.rs:35-47,70-78,139-147,171-184`; `gpui/window/images.rs:162-176,482-488`; `gpui/window/toasts.rs:30-36`; `gpui/window/toasts/card.rs:183-187`; `gpui/window/system_notifications.rs:40-51,145-155,363-369`; `gpui/window/config_diagnostic.rs:100-107,345-389`
- **Interfaces and call sites:** `gpui/endpoint.rs:584-588` bumps `selection_epoch` and copies `selected_generation`; `gpui/chat/start.rs:169-179,331-341` and `gpui/chat/existing.rs:43-50` build the same fence in two more shapes; `gpui/window/commands.rs:381`
- **Tests and intent evidence:** CLAUDE.md: "Reconnect and detach must isolate old inboxes so delayed events and paint acknowledgements cannot affect the new connection"; `endpoint/lifecycle_tests/toast_handoff.rs:27,151,252,364`; `window/config_diagnostic/tests.rs:149-159`; `window/transfers/tests/{cancellation,completion}.rs`; `window/wsl_drop/tests.rs`
- **Current representation:** each deferred action (held mouse gesture, file transfer, WSL paste, pending image, toast click, system notification click, config diagnostic) copies its own subset of endpoint id, generation, selection epoch, boot id and inbox pointer, and re-compares them by hand
- **Current complexity or invalid states:** about eleven hand-written comparisons in eight structs. The copies differ: `Gesture` has no endpoint id and compares `selected_generation`; the others compare `endpoints[selected].generation` plus the id; the notice paths compare inbox pointer identity instead of epoch and boot. A copy that drops one term still compiles and passes happy-path tests. Loose `(&str, u64, &Arc<..>, u64)` signatures let a caller pair one endpoint's generation with another's inbox.
- **Why it is material:** this is the safety rule that keeps a delayed action off a new connection, and it has no single owner. Three reviewers reported it independently.
- **Proposed representation:** one small value captured from the window (endpoint id, generation, epoch, boot, and a weak inbox for click handlers) and one resolver on `SheprdWindow`. Each site becomes `self.fence_holds(&x.fence) && <its own readiness terms>`.
- **Why it is simpler:** identity comparison has one definition; each site's own readiness conditions become visible
- **Implementation scope:** `window/{mouse,transfers,wsl_drop,images,toasts,system_notifications,config_diagnostic}.rs`, `toasts/{card,motion}.rs`, the helper beside `selection_epoch`; chat start and existing-chat scopes optionally
- **Smallest credible slice:** the input-side fence (`transfers`, `wsl_drop`, `images`), which already use identical fields
- **Regression risks:** `Gesture` would gain the endpoint-id term; confirm `selected_generation` and `endpoint.generation` cannot differ mid-gesture or a release could be skipped. `motion::Key` must stay `Hash + Eq` without the inbox. `Posted` must keep a weak inbox. Epoch-only scopes (`integrations.rs:194-200`, `chat/start.rs:275`) stay as they are.
- **Migration concerns:** whether the input-side fence (epoch and boot) and the click-side fence (inbox identity) are one value or two is a design choice to settle first
- **Existing validation:** the toast, transfer, WSL-drop and config-diagnostic tests above
- **Additional validation required:** one test of the predicate across reselect, reconnect and a new boot id; one test that a held gesture's release is not sent after each
- **Impact:** medium; drift prevention on a safety invariant, no defect evidenced
- **Confidence:** medium; duplication reviewer-read in full across two rows, not re-opened by the coordinator
- **Implementation effort:** medium
- **Blast radius:** cross-subsystem (window, chat)
- **Prerequisites:** none hard; land before or after SIFT-S30-02, not alongside, since both edit `toast_target`. The epoch and generation live in `endpoint.rs`, so the helper belongs there even though the consumers are window code.
- **Priority:** 29

---

#### SIFT-S30-02 · One stage enum for a notification

- **Finding ID:** SIFT-S30-02
- **Authoritative subsystem:** S30 notifications and sound
- **Title:** Replace `Notice`'s `ready`, `visible`, `posted` booleans with `Stage`
- **Verdict:** recommend
- **Primary evidence:** `gpui/notifications.rs:30,40-47,105-115,149,296-300,370-447,464,521-553`
- **Interfaces and call sites:** `gpui/window/toasts.rs:93,187-193`; `gpui/window/system_notifications.rs` (about 26 field references)
- **Tests and intent evidence:** `notifications/policy_tests*`, `window/toasts/tests*`, `endpoint/lifecycle_tests/toast_*`
- **Current representation:** three booleans plus `expires`, `checked`, `displayed`
- **Current complexity or invalid states:** four exclusive stages are encoded; `posted && visible` and `visible && !ready` are representable; correctness rests on guard order inside a 100-line closure; eviction can remove a posted entry
- **Why it is material:** stage is re-derived with compound tests at each consumer
- **Proposed representation:** `Stage { Waiting { checked }, Ready, Visible { expires, displayed }, Posted }`; `promote` and `take_system` are the only transitions
- **Why it is simpler:** fields exist only in the stage where they mean something
- **Implementation scope:** `notifications.rs`, `window/toasts.rs`, `window/system_notifications.rs` and its freedesktop outbox, test accessors
- **Smallest credible slice:** the enum inside `notifications.rs` with accessor methods for consumers
- **Regression risks:** pause-extension of `expires` while hidden; the QA preview that promotes before `receive`
- **Migration concerns:** tests set the booleans directly
- **Existing validation:** the three test areas above
- **Additional validation required:** a test that eviction never removes a posted notice, and one that a notice cannot be both visible and posted
- **Impact:** medium
- **Confidence:** medium-high; `system_notifications.rs` only sampled
- **Implementation effort:** medium
- **Blast radius:** cross-subsystem (notifications, window)
- **Prerequisites:** none
- **Priority:** 30

---

#### SIFT-S22-01 · Draw chat text from the prepared selection document

- **Finding ID:** SIFT-S22-01
- **Authoritative subsystem:** S22 rich chat view
- **Title:** Make the selection document the text authority and stop re-deriving and comparing per frame
- **Verdict:** recommend
- **Primary evidence:** `gpui/chat/selection/document.rs:250-334`; `gpui/chat/view/entries.rs:165-172,185-192,270-297,379-386,433-474,626-654,685-691`; `gpui/chat/view/text.rs:29-50`; `gpui/chat/prepared.rs:33-37,114-126`
- **Interfaces and call sites:** `gpui/chat/view/entries/questions.rs:50`
- **Tests and intent evidence:** `chat/tests/selection.rs`, `chat/tests/relocale.rs`, `view/selection/tests.rs`
- **Current representation:** the document maps kind, body and expansion to `(Region, Cell, text)`; the view repeats the mapping and reconciles by string comparison per visible fragment per frame
- **Current complexity or invalid states:** a mismatch draws plain, unselectable text with no signal. Clip drift exists: the document clips at 256 KiB, the view passes unclipped text, so a larger message is drawn in full and cannot be selected. `tool_text` clones whole tool output per content block per frame.
- **Why it is material:** two authorities that must agree byte for byte, with per-frame cost
- **Proposed representation:** `render_chat_text` takes `(entry, region, cell)` and draws the document's segment; tool details iterate the document's cells
- **Why it is simpler:** no comparison, no per-frame clones, no clip drift
- **Implementation scope:** `view/text.rs`, `view/entries.rs`, `view/entries/questions.rs`, possibly one accessor on `Document`
- **Smallest credible slice:** convert one entry kind (notices)
- **Regression risks:** the document must be projected before a row first renders; expansion and document version must update together; locale switches must re-project
- **Migration concerns:** debug selector indices must stay stable
- **Existing validation:** the three test areas above
- **Additional validation required:** a test that text over 256 KiB is clipped and selectable; the native selection probe
- **Impact:** medium
- **Confidence:** medium; reviewer-read; rebuild timing in `prepared.rs` not read
- **Implementation effort:** medium
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 31

---

#### SIFT-S32-01 · Retire the inherited upstream release route

- **Finding ID:** SIFT-S32-01
- **Authoritative subsystem:** S32 release tooling and CI
- **Title:** Remove `release.yml`, `promote.yml`, `dispatch.sh` and the scripts only they reach
- **Verdict:** recommend
- **Primary evidence:** `.github/workflows/release.yml:39-43,60-64,121-123,145-147`; `.github/workflows/promote.yml:36,57,117,145,188`; `scripts/release/dispatch.sh:30-37`; `justfile:286-295`
- **Interfaces and call sites:** scripts reached only from those workflows: `winget.py`, `windows-signing.py`, `windows-sign-file.ps1`, `package-linux.sh`, `package-linux-distro.py`, `smoke-linux-packages.sh`, `update-homebrew.sh`, `render-cask.sh`, `next-version.py`, `generate-sbom.py`, `cargo-locked.sh`, `artifact-manifest.py`, `scripts/verify-release.sh`, `homebrew/Casks/*.rb`
- **Tests and intent evidence:** `ci.yml:162-176` runs the whole `scripts/release/tests` discovery on three runners; `tests/test_updater_absence.py:54-61` asserts `release.yml` contains `github.actor == 'penso'`; D53 says the fork is maintained independently
- **Current representation:** two release systems share one directory with different identities and trust roots
- **Current complexity or invalid states:** every job requires repository `sheprd-labs/sheprd` and actor `penso`; `dispatch.sh` refuses any other origin (verified). Only the macOS preview route can run, yet the dead route's string contracts are enforced on every PR.
- **Why it is material:** editing shared scripts means keeping an unreachable pipeline green; about 13 scripts and about 20 test files
- **Proposed representation:** one release route, the macOS preview. Recover the rest from git history when gate G-RELEASE opens. Keep `package-windows.py`, `windows-installer.py`, `sheprd.iss`, which `ci.yml:340-396` still calls.
- **Why it is simpler:** one pipeline, one identity
- **Implementation scope:** two workflows, about 13 scripts, about 20 tests, `homebrew/`, three just recipes, README sections
- **Smallest credible slice:** delete `dispatch.sh` and the three just recipes, which cannot succeed here
- **Regression risks:** `package-macos.sh`, `sign-macos.sh`, `generate-notices.py`, `smoke-cli.py`, `check-ci.py`, `generate-changelog.sh` are shared with the live route; `test_release.py` and `test-release-security.py` mix live and dead cases and must be split; `flake.nix` reads `org.sheprd.Sheprd.desktop`
- **Migration concerns:** owner decision: keep as a template for T12, or delete and restore later. Large deletion, so it needs explicit approval.
- **Existing validation:** remaining `unittest discover`, `test-release-security.py`, actionlint, zizmor
- **Additional validation required:** grep for zero remaining callers of each deleted script
- **Impact:** medium-high maintenance; no runtime effect
- **Confidence:** medium; dead-or-live evidence is direct, keeping it is a policy call
- **Implementation effort:** large
- **Blast radius:** subsystem
- **Prerequisites:** owner decision; SIFT-S32-02 first, because both rewrite `test_ci_policy.py` and `test-release-security.py`
- **Priority:** 32

---

#### SIFT-S09-02 · Exhaustive caller and wait classification on `Request`

- **Finding ID:** SIFT-S09-02
- **Authoritative subsystem:** S09 control protocol and command catalog
- **Title:** `Request::caller()` and wildcard-free `wait_seconds`
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-control-protocol/src/protocol.rs:51-76,97-106`; `crates/sheprd-control-protocol/src/protocol/hosts.rs:47-52`; `crates/sheprd-control-protocol/src/protocol/settings.rs:55-61`; `crates/sheprd-control-protocol/src/socket.rs:68`
- **Interfaces and call sites:** `gpui/control/server.rs:222-247`
- **Tests and intent evidence:** protocol tests, `socket/tests`
- **Current representation:** `wait_seconds` ends in `_ => 0`; the server keeps a second hand-kept `may_park` list and a 12-arm match to extract the caller
- **Current complexity or invalid states:** the two lists already disagree on `Work`. A new waiting op that misses the wildcard gets a short client timeout while the server parks it.
- **Why it is material:** a spurious-timeout class no compiler check catches
- **Proposed representation:** `caller()` in the protocol crate; exhaustive matches; the server derives `may_park` from the protocol
- **Why it is simpler:** one classification
- **Implementation scope:** `protocol.rs`, `protocol/hosts.rs`, `protocol/settings.rs`, `server.rs:222-247`
- **Smallest credible slice:** `caller()` and the exhaustive matches
- **Regression risks:** if `Work` truly parks, deriving `may_park` from `wait_seconds` drops its full-queue refusal
- **Migration concerns:** no version bump needed
- **Existing validation:** protocol and control server tests
- **Additional validation required:** every parking family reports a nonzero wait
- **Impact:** medium-low
- **Confidence:** medium; whether `work::serve` parks was not traced
- **Implementation effort:** small
- **Blast radius:** cross-subsystem (protocol, control server)
- **Prerequisites:** none
- **Priority:** 33

---

#### SIFT-S02-01 · One state type for the API request lease

- **Finding ID:** SIFT-S02-01
- **Authoritative subsystem:** S02 sheprd-client session worker
- **Title:** A barrier enum that owns response assembly
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-client/src/session.rs:142-144,170-176,416-419,459-465,514-532,761-845`; `crates/sheprd-client/src/session/barrier.rs:9-23,41-68`
- **Interfaces and call sites:** crate-private; no public API or wire change
- **Tests and intent evidence:** `crates/sheprd-client/src/tests/focus_barrier.rs:309-318` builds the struct by hand; `tests/commands.rs`, `tests/errors.rs`
- **Current representation:** `Option<Pending>`, `Option<Barrier>`, and `held: Option<Value>` plus `refused: bool` inside `Barrier`
- **Current complexity or invalid states:** `held.is_some() && refused` is representable. The response arm looks up the barrier three times and carries an `ok_or` that cannot fire and an `unwrap_or_default()` request id. `session.rs` is 910 lines against the 1,000-line limit.
- **Why it is material:** lease transitions are written from two places in the protocol core
- **Proposed representation:** `enum { Awaiting, Held(Value), Refused }` on `Barrier`, and a method in the barrier module that takes the response and returns what to emit
- **Why it is simpler:** one owner for transitions; dead error paths disappear
- **Implementation scope:** `session.rs`, `session/barrier.rs`, test literals
- **Smallest credible slice:** the enum alone
- **Regression risks:** the shared `MAX_RESPONSE_BYTES` bound, focus-before-activation order, `pipeline_barriers = false` on `endpoint_busy`, resend once only
- **Migration concerns:** none
- **Existing validation:** `tests/focus_barrier.rs`, `tests/commands.rs`, `tests/errors.rs`, `tests/session_state.rs`
- **Additional validation required:** an activation response arriving after the focus response
- **Impact:** medium
- **Confidence:** medium; ordering rules are subtle and nothing was run
- **Implementation effort:** medium
- **Blast radius:** local
- **Prerequisites:** none
- **Priority:** 34

---

#### SIFT-S10-02 · Parse CLI families straight into protocol ops

- **Finding ID:** SIFT-S10-02
- **Authoritative subsystem:** S10 control server, client and CLI
- **Title:** Drop the nine mirror `*Command` enums
- **Verdict:** recommend
- **Primary evidence:** `gpui/cli/control/app.rs:4-32`; `gpui/cli/control/hosts.rs:4-26`; `gpui/cli/control/tabs.rs:4-9`; `gpui/cli/control/chat.rs:4-20`; `gpui/cli/control/work.rs:4-19`; `gpui/control/client/app.rs:110-138,162-167`; `gpui/control/client/hosts.rs:19-40`
- **Interfaces and call sites:** `gpui/cli/control.rs:14-24` already re-exports protocol types
- **Tests and intent evidence:** `gpui/cli/tests/control_commands.rs`, `chat.rs`, `ui_panel.rs`; CLAUDE.md forbids mirror enums
- **Current representation:** CLI enums restating `AppOp`, `WindowOp`, `UiOp`, `TabOp`, `HostOp`, `ChatOp`, `WorkOp` nearly variant for variant, with translation arms
- **Current complexity or invalid states:** every new verb adds a variant in two enums plus a translation arm
- **Why it is material:** recurring per-verb cost
- **Proposed representation:** the parser returns the op; non-identity cases (`window focus ID`, `notes clear`, `chat new`) move to parse time; settings families keep their enums
- **Why it is simpler:** one enum per family
- **Implementation scope:** five `cli/control` files, `cli/control.rs`, five client files, parser tests
- **Smallest credible slice:** one family (`tab`)
- **Regression risks:** `window focus ID --window N` precedence; the 16 KiB note bound stays in the parser
- **Migration concerns:** none; no wire change
- **Existing validation:** parser tests, client tests
- **Additional validation required:** a `window focus` precedence test
- **Impact:** medium-low
- **Confidence:** medium
- **Implementation effort:** medium
- **Blast radius:** subsystem
- **Prerequisites:** easier after SIFT-S10-01
- **Priority:** 35

---

#### SIFT-S08-01 · A shared change-stream follower

- **Finding ID:** SIFT-S08-01
- **Authoritative subsystem:** S08 sheprd-chat-protocol
- **Title:** Put the cursor gate, reducer call and touched-entry mapping in a `Replica` beside the reducer
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-chat-protocol/src/change.rs:83-162`; `crates/sheprd-chat-protocol/src/wire.rs:19-37`; `crates/sheprd-chat-protocol/README.md:59-71`
- **Interfaces and call sites:** `crates/sheprd-acp/src/tui/view.rs:159-173,186-196`; `gpui/chat/session.rs:59-65,203-225,240-254`
- **Tests and intent evidence:** `crates/sheprd-acp/src/tui/view/tests.rs`; `gpui/chat/tests/session.rs`
- **Current representation:** the "a client takes a change only right after the one before it" rule is prose in the README and hand-written in both clients
- **Current complexity or invalid states:** the copies differ (verified): the TUI computes `known.seq + 1`, GPUI uses `checked_add`; the TUI clears the cursor inline on failure, GPUI returns an error and relies on a later `lost()`. Both re-derive the touched entry with a wildcard arm, so a new `Change` variant is not flagged.
- **Why it is material:** the cursor and transcript invariant holds by convention in two places
- **Proposed representation:** `Replica { cursor, transcript }` with `reset`, `follow(&change, cursor) -> Result<Touched>` and a closed `Touched` enum from one exhaustive match
- **Why it is simpler:** one authority and compiler-checked variant coverage
- **Implementation scope:** a new module and tests in `sheprd-chat-protocol`; `sheprd-acp/src/tui/view.rs`; `gpui/chat/session.rs`. No wire change.
- **Smallest credible slice:** both adopters together; one adopter alone adds a third copy
- **Regression risks:** GPUI keeps the stale transcript on screen after a desync, so `Replica` must not clear it; `Error::Stream { after, got }` feeds an i18n string
- **Migration concerns:** README paragraph
- **Existing validation:** the two test files above
- **Additional validation required:** `seq` at `u64::MAX`; a failed apply leaves the transcript unchanged; `Touched` for every `Change` variant
- **Impact:** medium
- **Confidence:** medium; verified both copies and their differences; each is only about 25 lines
- **Implementation effort:** small
- **Blast radius:** cross-subsystem (three crates)
- **Prerequisites:** none
- **Priority:** 36

---

#### SIFT-S13-01 · One validator for Fluent file shape

- **Finding ID:** SIFT-S13-01
- **Authoritative subsystem:** S13 localisation
- **Title:** The build script owns file shape; the Python checker keeps only whole-repository rules
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-gpui/locales.rs:88-118`; `scripts/check-i18n.py:107-136,263-300,305-311,315-319,326-328`; `gpui/i18n/check.rs:8-30`
- **Interfaces and call sites:** `justfile:190-191`; `.github/workflows/ci.yml:196-200`
- **Tests and intent evidence:** `scripts/test_check_i18n.py:26-84` exists to keep the second parser in step with Fluent grammar; the script's docstring admits the overlap
- **Current representation:** `fluent_syntax` in the build and a hand-written line and brace scanner in Python enforce the same three rules
- **Current complexity or invalid states:** two grammars define "valid file"; valid Fluent can fail CI. The "translation has no ID English lacks" rule lives only in Python.
- **Why it is material:** an approximate second parser to maintain
- **Proposed representation:** `locales.rs` gains the stale-translation rule; the script keeps unused messages, migrated literals and platform files, reading IDs only
- **Why it is simpler:** one grammar
- **Implementation scope:** `locales.rs`, `check-i18n.py`, `test_check_i18n.py`, `docs/i18n.md`
- **Smallest credible slice:** add the stale-translation rule to the build
- **Regression risks:** `just check-i18n` alone no longer reports malformed files; `build.rs` must stay dependency-free
- **Migration concerns:** none
- **Existing validation:** `gpui/i18n/tests/messages.rs`, remaining Python tests
- **Additional validation required:** a build-helper test for a translation ID English lacks
- **Impact:** medium-low
- **Confidence:** medium
- **Implementation effort:** small
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 37

---

#### SIFT-S23-01 · A chat target's source as one enum

- **Finding ID:** SIFT-S23-01
- **Authoritative subsystem:** S23 chat lifecycle
- **Title:** `enum Source { Program(EndpointId), Observed(Binding) }` on `Target`
- **Verdict:** recommend
- **Primary evidence:** `gpui/chat.rs:149-159`; `gpui/chat/discovery.rs:87-93`; `gpui/chat/chats.rs:42-77,332,448-456`; `gpui/chat/existing.rs:55-63`
- **Interfaces and call sites:** `gpui/chat/preview.rs:609`; `gpui/control/server/chat.rs:121,135-141`
- **Tests and intent evidence:** `chat/tests/discovery.rs`, `chat/tests/chats/observed_identity.rs`; comment at `chats.rs:521-522` that two panes may observe one session
- **Current representation:** `Target { endpoint, existing: Option<Binding>, pane_id }`
- **Current complexity or invalid states:** an observed target holds an endpoint made from its session UUID that addresses nothing. `existing.is_some()` decides behaviour at five places; the control server re-derives the input mode `ChatPane::input` already computes.
- **Why it is material:** two fields encode one choice
- **Proposed representation:** the `Source` enum; `connect` matches `(source, route)`; the control server maps `ChatInput` to `InputMode`
- **Why it is simpler:** no placeholder endpoint; one input-mode derivation
- **Implementation scope:** `chat.rs`, `chat/discovery.rs`, `chat/core_pane.rs:45-51`, `chat/chats.rs`, `chat/preview.rs`, `control/server/chat.rs`, test fixtures
- **Smallest credible slice:** the control server reuses `ChatPane::input`
- **Regression risks:** `Target` is a `Hash` and `Eq` key; `pane_id` must stay in it
- **Migration concerns:** none
- **Existing validation:** the chat discovery, chats, existing and observer tests; control chat tests
- **Additional validation required:** `connect` builds the observe command for each route
- **Impact:** medium-low
- **Confidence:** medium; `preview.rs` not read
- **Implementation effort:** small
- **Blast radius:** cross-subsystem (chat, control server)
- **Prerequisites:** none
- **Priority:** 38

---

#### SIFT-S06-01 · One typed turn-ended reply for the three bridges

- **Finding ID:** SIFT-S06-01
- **Authoritative subsystem:** S06 sheprd-acp agent adapters
- **Title:** A shared `turn_ended` builder and a serializable `PromptUsage`
- **Verdict:** recommend
- **Primary evidence:** `crates/sheprd-acp/src/claude/bridge.rs:193-195,570-574,689-698,768-785`; `crates/sheprd-acp/src/codex/bridge.rs:183-188,476-493,528-537`; `crates/sheprd-acp/src/pi/bridge.rs:249-254,513-525`; `crates/sheprd-acp/src/project.rs:612-637`
- **Interfaces and call sites:** `crates/sheprd-acp/src/session/agent.rs:315`
- **Tests and intent evidence:** `claude/bridge/tests/turns.rs:299`, `codex/bridge/tests.rs:276-285`, `pi/bridge/tests.rs:178`
- **Current representation:** the `usage` object is hand-written `json!` in three writers and parsed strictly by one reader
- **Current complexity or invalid states:** a misspelt key silently drops the turn's tokens. "Total" means different things: Claude and Pi compute it, Codex passes its own.
- **Why it is material:** an untyped contract with silent loss
- **Proposed representation:** `PromptUsage` derives Serialize and Deserialize; `out::turn_ended(id, result, usage)` in `harness/out.rs`; a shared busy constant. No shared bridge base, no trait.
- **Why it is simpler:** one struct for writers and reader
- **Implementation scope:** `harness/out.rs`, three `bridge.rs` files, `project.rs`
- **Smallest credible slice:** the shared struct
- **Regression risks:** serialized output must stay byte-compatible; Pi omits `thoughtTokens` today
- **Migration concerns:** do not normalise the Codex total without deciding it
- **Existing validation:** the bridge tests above
- **Additional validation required:** a round-trip from `turn_ended` to `project::tokens`
- **Impact:** medium-low
- **Confidence:** medium
- **Implementation effort:** small
- **Blast radius:** subsystem
- **Prerequisites:** none
- **Priority:** 39

---

#### SIFT-S18-01 · One cell-metrics value for paint, hit testing and IME

- **Finding ID:** SIFT-S18-01
- **Authoritative subsystem:** S18 terminal element and painting
- **Title:** `CellMetrics { width, height }` with `origin`, `bounds` and `cell_at`
- **Verdict:** recommend
- **Primary evidence:** `gpui/terminal_painter.rs:125-134,294-302,344-350,399-409,448-462,475-481`; `gpui/terminal.rs:96-102,120-147,355-363,385-413,440-446,680-689`; `gpui/window/regions.rs:26-33,85,223-229,272-283`; `gpui/window/input.rs:135-145`
- **Interfaces and call sites:** `gpui/group_terminals.rs:616,641`, `gpui/core_terminals/view.rs:331`, `gpui/input.rs:210`, `gpui/palette/preview.rs:250`, `gpui/terminal/links.rs:244-245,274-275`, `gpui/terminal/selection.rs:104,120`, `gpui/terminal/splits.rs:23-24`
- **Tests and intent evidence:** CLAUDE.md requires the same geometry for painting, hit testing and IME; `terminal_painter/tests/*`, `terminal/tests/*`
- **Current representation:** geometry carried three ways: painter state, call arguments and `Look`; 37 `cell_width: f32` parameters
- **Current complexity or invalid states:** a caller can pass a width or font that disagrees with the painter's height; a differing font silently clears the glyph cache on every paint. Rect-to-pixel bounds are hand-built in eight places. One visible drift: the slide paints at a rounded offset while hit testing uses the unrounded one.
- **Why it is material:** the copies agree only because of call order
- **Proposed representation:** a small `Copy` value produced once per appearance and taken by `paint_frame`, `popup_origin`, `input_area`, `pane_at` and peers
- **Why it is simpler:** one conversion each way
- **Implementation scope:** `terminal.rs`, `terminal_painter.rs`, `terminal_painter/images.rs`, `window/regions.rs`, mechanical caller updates
- **Smallest credible slice:** the type plus helpers, before changing `paint_frame`
- **Regression risks:** float evaluation order must stay identical; `splits.rs` truncates where others floor; `palette/preview.rs` owns a separate painter
- **Migration concerns:** none
- **Existing validation:** the terminal and painter tests, `window/regions` tests
- **Additional validation required:** a `bounds` and `cell_at` round-trip; `just test-perf` budgets unchanged; IME placement by hand
- **Impact:** medium
- **Confidence:** medium; no live defect found
- **Implementation effort:** medium
- **Blast radius:** cross-subsystem
- **Prerequisites:** none
- **Priority:** 40

---

#### SIFT-S11-02 · One SSH local-forward lifecycle

- **Finding ID:** SIFT-S11-02
- **Authoritative subsystem:** S11 usage, remote helpers, ports
- **Title:** One per-window forward table in place of `Tunnels` and `PortForwards`
- **Verdict:** recommend
- **Primary evidence:** `gpui/listening_ports/tunnel.rs:1-393`; `gpui/port_forward.rs:1-219`; `crates/sheprd-client/src/forward.rs:22-150`
- **Interfaces and call sites:** `gpui/window.rs:235,321`; `gpui/control/server/hosts/ports.rs:20-56`; `gpui/listening_ports/render.rs:190-275`
- **Tests and intent evidence:** `gpui/listening_ports/tests.rs:710`, `port_forward/tests.rs`
- **Current representation:** two tables for "remote port P on target T is forwarded to local L", each with its own limit of 32 and teardown
- **Current complexity or invalid states:** the same port can be forwarded twice. The control server reconciles the tables by hand. `Ended` with a live handle is representable.
- **Why it is material:** duplicated process lifecycle code
- **Proposed representation:** one table keyed by target, bind address and port, backed by `sheprd_client::PortForward` extended with the optional remote bind address
- **Why it is simpler:** one lifecycle, one limit
- **Implementation scope:** `listening_ports/tunnel.rs`, `listening_ports/render.rs`, `port_forward.rs`, `window/monitors.rs`, `control/server/hosts/ports.rs`, `sheprd-client/src/forward.rs`
- **Smallest credible slice:** a port click reuses an existing host-menu forward
- **Regression risks:** the two paths prove readiness differently; the "never open another process's page" guarantee must hold; `spaceport ports` JSON shape must not change
- **Migration concerns:** none persisted
- **Existing validation:** tunnel and forward tests
- **Additional validation required:** a reuse test; a native remote-port click
- **Impact:** medium
- **Confidence:** medium; `forward.rs:241-371` not read closely
- **Implementation effort:** large
- **Blast radius:** cross-subsystem
- **Prerequisites:** none
- **Priority:** 41

---

#### SIFT-S12-01 · One rule for kept log targets

- **Finding ID:** SIFT-S12-01
- **Authoritative subsystem:** S12 diagnostics and logging
- **Title:** This binary's crate plus the `sheprd_` prefix, instead of a hand-kept list
- **Verdict:** recommend
- **Primary evidence:** `gpui/log_targets.rs:6-22`; `gpui/log_targets/tests.rs:6-19`; `crates/sheprd-diagnostics/src/lib.rs:435-448`
- **Interfaces and call sites:** `gpui/app.rs:391`
- **Tests and intent evidence:** branding audit finding BR-01 (`docs/audits/AUDIT_REPORT_2026-11-12.md`) measured a silent drop from this list; BR-01 itself is fixed
- **Current representation:** twelve literal crate names plus a test that re-derives part of the graph from TOML
- **Current complexity or invalid states:** a workspace crate reached only transitively can start logging and be dropped silently with the test green; no loss today
- **Why it is material:** modest; the drift class is proven, the exposure is latent
- **Proposed representation:** the collector admits targets whose first path segment starts with `sheprd_`
- **Why it is simpler:** the naming convention is the single authority
- **Implementation scope:** `sheprd-diagnostics/src/lib.rs` and tests, `log_targets.rs`, one call site
- **Smallest credible slice:** the whole change
- **Regression risks:** match on the first segment only; a future crate without the prefix needs an explicit root
- **Migration concerns:** none
- **Existing validation:** `kept_target` tests
- **Additional validation required:** an end-to-end capture under the compiled crate name
- **Impact:** low
- **Confidence:** medium
- **Implementation effort:** small
- **Blast radius:** local
- **Prerequisites:** none
- **Priority:** 42

## 4. Smallest credible implementation slices

These are the first slices of the top findings. None is implemented.

**Slice A, from SIFT-S32-02: turn the supply-chain gate on.**
1. Boundary: `.github/workflows/supply-chain.yml` only.
2. Changed: the two job guards name `2389-research/spaceport` and its maintainer.
3. Compatibility: none. The required-check name `Format, lint, and test` is in another workflow.
4. Validation: actionlint and zizmor locally; one pull request and one push to `main` showing the jobs ran.
5. Deferred: the nine guards in `ci.yml`, `nix.yml`, the three housekeeping workflows, `check-ci.py` defaults.

**Slice B, from SIFT-S07-01: make `fail` tear down like `exited`.**
1. Boundary: `crates/sheprd-acp/src/session.rs` and `session/agent.rs`.
2. Changed: one private teardown function called by both; no protocol change.
3. Compatibility: decide first whether a failure withdraws or cancels open permissions.
4. Validation: existing `session/tests/*`; new tests for a mid-turn failure with an open permission and for a late reply after failure.
5. Deferred: the `Phase` enum and the command gating.

**Slice C, from SIFT-S25-01: delete the ten unreachable steps.**
1. Boundary: `gpui/menu/devices/{add_device,setup,saved_warning,error}.rs`; delete `host_key.rs`, `host_key_choice.rs`, `local_network.rs`, `add_device/offer.rs`.
2. Changed: `Step` shrinks to six variants; `Setup` loses four fields.
3. Compatibility: needs your approval first. Remove matching Fluent keys and i18n error arms in the same change.
4. Validation: Core add-device tests, host-menu remove and rename tests, `just check-sorted`, the i18n checks, one native Add Device pass.
5. Deferred: `setup_relay*`, then the client-crate sweep in SIFT-S03-01.

**Slice D, from SIFT-S16-02: add `live_of` and `hosts`, convert the overview.**
1. Boundary: `gpui/endpoint.rs` and `gpui/overview.rs`.
2. Changed: two methods on `SheprdWindow`; five branch sites become iterator use.
3. Compatibility: none.
4. Validation: overview tests, plus a new two-endpoint test where the two copies differ.
5. Deferred: palette and sidebar sites, single ownership of `LiveState`.

**Slice E, from SIFT-S09-01: generate `COMMANDS` from one table.**
1. Boundary: `crates/sheprd-commands/src/lib.rs`.
2. Changed: a `macro_rules!` table emits the enum and `COMMANDS`; public surface unchanged.
3. Compatibility: keep the old test mirror for one commit as an equality check on order, names and messages.
4. Validation: `sheprd-commands` tests, `gpui/keymap/tests/platform_defaults.rs`.
5. Deferred: generating the per-platform shortcut defaults; shrinking the test mirror.

**Slice F, from SIFT-S33-01: gate the spec validator.**
1. Boundary: `justfile` and one step in the always-run CI job.
2. Changed: a `check-spec` recipe running `spec.py check --skip-schema`.
3. Compatibility: the first run may fail on existing drift; fix or regenerate `validation.json` in the same change.
4. Validation: `test_spec.py`.
5. Deferred: evidence-path and enum checks.

**Slice G, from SIFT-S27-01: catalogue the seven unsearchable rows.**
1. Boundary: `gpui/settings_window/catalog.rs` and `tests/search.rs`.
2. Changed: seven catalog entries and a test that every rendered static row has one.
3. Compatibility: none.
4. Validation: settings search tests.
5. Deferred: the single row builder and choosing between the five drifted descriptions.

## 5. Explicit skips

- **S01, wire protocol.** Read every production file and the client's dispatch. One validation authority already exists. Candidate deduplications were about 15 lines each with differing rules, and the layout is a frozen wire contract.
- **S05, terminal emulator.** Read every production file and its one consumer. Input encoding has one entry point, modes are enums, the scanner is an explicit state machine. Note for S18: the cell modifier bits and colour encoding are restated by number in `crates/sheprd-terminal/src/lib.rs:19-28` and `gpui/terminal.rs:31-37,452-476`; see section 7.
- **S28, palette, keymap, overview.** The app keeps no duplicated per-command table, palette sources share one `Entry` model and one ranker, and navigation targets are typed. Its one candidate was the live-state accessor, which became SIFT-S16-02.
- **S29, application shell.** Two candidates were drafted and dropped: a `PersonalState` struct for seven adjacent load-or-default lines, and an update-availability cleanup that folded into SIFT-S14-01. `release_notes.rs` is the shared Markdown line model, not updater residue.
- **G1, window media and notices.** Read every production file. Its one candidate, a shared identity for the connection a notice was drawn from, merged into SIFT-S21-02.
- **G2, terminal mouse, copy mode, find, clipboard, drops.** Read every production file. Its one candidate, a connection fence for deferred input, merged into SIFT-S21-02. `prefix_armed` and `resize_mode` are already mutually exclusive in practice.
- **G3, daemon start, sessions, network quality.** Read every production file. `connect_or_start` is one linear function with typed exits. The session-scan flags would save about 20 lines as an enum. Policy note in section 7.
- **G4, ACP remainder.** Read every production file. The per-agent tool, permission and settings tables look alike but map different vocabularies; a shared registry would cost more than it removes.
- **G5, rich chat rendering.** The same-named layers (`chat/rich` and `chat/view/rich`, and so on) are one model and one renderer. Layout variants share one row model. `preview` is behind the `integration-test` feature. Three per-frame scans were noted and judged micro-optimisation without measured cost.
- **G6, sidebar rows, workspace and host menus.** One candidate, a request phase for the open-worktree dialog, was dropped as modest. The automations panel is a deliberate "unavailable" placeholder under RM-AUTOMATION and gap G-OBS, not removable residue. Much of this row was not opened; see section 10.
- **G8, platform integration.** There are no per-platform triplets restating one decision; caffeine delegates to one crate behind one state machine, and the icon scripts are a pipeline, not copies. The preview app identity literals in release scripts are fail-closed equality checks.
- **N1, static web, design and assets.** Inspected file lists and the two `netlify.toml` files only. Not source for this audit. Note for the docs owner: `website/` is the inherited sheprd.org site and `site/` is the one the root `netlify.toml` publishes.
- **N2, prose and agent state.** Not source.
- **N3, lockfiles and translation copy.** Lockfiles derive from manifests. The structure of the locale files was reviewed in S13; their wording is out of scope.

## 6. Cross-cutting patterns

- **Code left behind by features the fork removed** (S03, S11, S25, S31, S32, and notes from S13 and S14). This is one shared source-of-truth problem: `docs/spaceport/plan/removals.json` records what was removed, but nothing checks that the code no longer carries it. Several removals are recorded as done or in progress while tests still pin the old strings in place. It implies a sequencing dependency: do SIFT-S25-01 before SIFT-S03-01 and SIFT-S02B-01, and sweep for orphans once.
- **Two copies of per-endpoint state, reconciled by hand** (S16, S24, S28, S17, G1, G2, S23). One shared problem with two faces: which `LiveState` is current (SIFT-S16-02), and whether a deferred action still belongs to this connection (SIFT-S21-02). Both helpers belong in `endpoint.rs`.
- **State machines encoded as flags** (S07, S16, S19, S23, S24, S30, S02). This is repeated but appropriately local code. Each needs its own enum; a shared "phase" abstraction across them would be speculative. The critic rejected coupling the tab drag and the sidebar drags for this reason.
- **Lists declared several times with a string or wildcard fallback** (S09, S10, S26, S27, S16). Repeated but local. The repository's own "Parallel Pull Requests" rules already prescribe the fix, one table per list.
- **Hand-parsed endpoint responses** (S02b, G3, G6, S21). One shared problem, owned by `sheprd-client` (SIFT-S02B-01).
- **Test bodies inside production files**, against the repository rule: `gpui/config_diagnostic.rs:138-218`, `gpui/integrations.rs:508-686`, `gpui/group_terminals.rs:690-734`, `gpui/browser.rs:97-132`, `gpui/browser/hidden.rs:97-133`, `gpui/browser/group_motion.rs:202-252`, `crates/sheprd-client/src/upload.rs`, `crates/sheprd-client/src/remote_host.rs:64-84`. This is hygiene, not a structural finding, so it has no finding ID.

## 7. Rejected, merged, and superseded candidates

| Candidate | Disposition | Reason | Authoritative finding/subsystem, if any |
|---|---|---|---|
| Explicit `Kind` enum for queued client commands (drafted as SIFT-S02-02) | rejected | No defect; medium effort in the ordering-sensitive protocol core | S02 |
| `Message` newtype for run-time message IDs (SIFT-S13-02) | rejected | Application-wide mechanical diff for a modest gain | S13 |
| One selector and runner for native fixtures (SIFT-S14-02) | rejected | Test-harness tidying that needs three native desktops to verify | S14 |
| Persist sidebar chrome through `state_file` (SIFT-S15-02) | rejected | Modest, and dropping the fsync weakens crash durability | S15 |
| Build each tab strip's list once (SIFT-S19-02) | rejected | No divergence proven; medium effort in a 926-line file | S19 |
| Tagged `PressedLink` target (SIFT-S20-02) | rejected | The constructor already keeps the three options exclusive | S20 |
| Tagged settings-save value (SIFT-S27-02) | rejected | No path reaches the invalid combination today | S27 |
| `PersonalState` for launch-versus-fixture loading (SIFT-S29-01) | rejected | Seven adjacent lines; the hazard is a hypothetical future store | S29 |
| One update-availability authority (SIFT-S29-02) | merged | Same lines as the build-channel change | SIFT-S14-01 |
| Shared agent-evidence verdict for toasts and sound (SIFT-S30-01) | rejected | The copies agree today; deduplication against a hypothetical status | S30 |
| One list of script checks (SIFT-S33-02) | merged | Its real content is deleting orphans, which follows the housekeeping decision | SIFT-S32-02 |
| Request phase for the open-worktree dialog (SIFT-G6-01) | rejected | Five guards on one small dialog | G6 |
| "Listed hosts" iterator in the sidebar | superseded | A second iterator beside `hosts()` | SIFT-S16-02; SIFT-S24-01 narrowed |
| Live-state accessor reported by S24 and S28 | merged | Same branch at the same sites | SIFT-S16-02 |
| Stale-connection identity (G1) and connection fence (G2) | merged | One invariant in two shapes | SIFT-S21-02 |
| Resize-notification flag held twice in the core client (S04) | demoted | About 20 lines; benefit modest | S04 |
| `Native` enum for natively launched agents (S06) | demoted | The invalid path is unreachable today | S06 |
| Typed endpoint ID key instead of prefix strings (S16) | demoted | No defect evidenced; marginal | S16 |
| Shared constants for cell modifier bits and colour encoding (S05 note) | demoted | Real duplication of a wire value across two crates, but no drift found and G7 judged the modifier byte trivial | `sheprd-protocol` owner, as an investigation |
| Unify transcript state between ACP session and mirror | rejected | Not duplicated: the session is the single writer and clients are replicas fed by the shared reducer | S07 |
| Unify the three bounded child-process runners in the client | rejected | Their stream, limit and cancellation contracts differ | S03 |
| Remove the automations panel and setting | rejected | A deliberate placeholder the spec keeps (RM-AUTOMATION, gap G-OBS) | spec owner |
| Remove the usage panel and registry (about 1,700 lines with no producer) | rejected | RM-PROVIDERS deliberately keeps the passive models | spec owner |
| Regroup `SheprdWindow`'s roughly 165 fields into sub-structs | rejected | A reshuffle that removes no invalid state | S21 |
| Session delete runs `herdr session stop` (`crates/sheprd-client/src/sessions.rs:30-43`, called from `gpui/menu/sessions/management.rs:266-272`) | not a finding | A policy question about decision D58, raised by the G3 reviewer; yours to decide | owner |
| Private security reporting, branding text | not a finding | Covered by the 2026-11-12 branding audit | `docs/audits/AUDIT_REPORT_2026-11-12.md` |

## 8. Audit-of-audit results

Two fresh reviewers who took no part in the audit checked the draft.

**Coverage and missing boundaries.** The coverage auditor compared the draft ledger with `git ls-files` and judged it "not yet honest". It found about 50 modules, roughly 28,000 lines and over half of them tests, that no row's boundary named, plus top-level manifests and several published contracts with no owner. Resolution: the ledger in section 2 was rewritten so that every tracked path has exactly one owning row, a new row N4 holds the manifests and contracts, and the status of this report changed from complete to `INCOMPLETE`. Most of the unnamed modules had been assigned to a reviewer who did not open them; they are now listed in section 10.

**Duplication and ownership overlap.** The coverage auditor found twelve boundary overlaps between the first-round rows and the gap-fill rows. Resolution: the "more specific row wins" rule at the top of section 2, and narrowed globs (for example `pi.rs, pi/` so it no longer matches `pictures`). The critic found overlapping findings; resolutions are in section 7 (two merges, one superseded iterator) and in corrected prerequisites: SIFT-S25-01 before SIFT-S03-01 and SIFT-S02B-01, SIFT-S31-01 before SIFT-S17-01, SIFT-S32-02 before SIFT-S32-01. The critic also judged that SIFT-S21-02's helper belongs in `endpoint.rs`; S16 already has two findings, so the finding stays with S21 and says where the code goes.

**Materiality and over-abstraction.** The critic argued twelve of 54 drafted findings below the gate. All twelve were dropped or merged. It marked four more as weak but worth keeping low: SIFT-S16-02 (a wrapper until single ownership follows, now stated in the finding), SIFT-S08-01, SIFT-S18-01 and SIFT-S12-01. They rank 6, 36, 40 and 42.

**Finding-schema completeness.** No field was missing. Three unranked findings were ranked. Two "none beyond existing" validation fields were filled. One limit remains: for brevity, several low-ranked findings give Impact, Confidence or Effort as a bare level without a rationale clause.

**Dependency-aware priority ranking.** The critic's order was adopted. Main moves from the first draft: SIFT-S07-01 up to 2, SIFT-S11-01 and SIFT-S22-02 up, SIFT-S32-01 down to 32 because it is large and policy-gated, SIFT-S04-01 down.

**Evidence spot-checks.** The critic tested one concrete claim in each of 15 findings. Fourteen were confirmed. One was wrong: SIFT-S21-01 counted four clears of `local_error` and missed two conditional take-and-restore sites in `gpui/window/pending_input.rs:409,415`. The finding is corrected. Two confirmations came with caveats that are now in the text: SIFT-S09-01 overstated which CLAUDE.md rule the test mirror breaks, and SIFT-S04-01's scope omitted `sheprd-terminal`.

**Deviations from the skill's procedure.** Reviewers were told to read the brief file themselves; its text was not pasted into each prompt. One reviewer (S33) ran a single `python3 -I -c` that only read JSON key shapes and wrote two temporary lists to the session scratchpad. The coordinator ran one similar read-only Python line to count decisions. Nothing was written inside the repository.

## 9. Repository integrity

- **Baseline status:** `git status --porcelain=v1 --untracked-files=all` at revision `64d51b05b` on `main`: 11 untracked paths (eight files under `.private-journal/`, `docs/Spaceport-Specification.md`, `docs/spaceport-spec-v1.zip`, `symbols-audit.md`) and no tracked changes.
- **Final status:** the same command, captured after all reviews and before this report was written: 11 lines.
- **Comparison:** unchanged. `cmp` reported the two captures identical, and `HEAD` is still `64d51b05b`.
- **Integrity verdict:** pass
- **Post-audit writes:** `docs/sift-audit-2026-11-13.md`, this report, is the only write.

## 10. Incomplete-work ledger

Every row below had a reviewer. What is listed is what that reviewer, by their own account, did not open. A "skip" or a finding for the row does not cover these files. Test bodies were sampled for intent throughout and are not listed again per row unless the row's tests were not looked at at all.

| Row | Not read | Why |
|---|---|---|
| N4 | All of it: root `Cargo.toml`, `rust-toolchain.toml`, `clippy.toml`, `taplo.toml`, `.config/nextest.toml`, `.gitattributes`, `.gitignore`, `docs/spaceport/contracts/control.schema.json`, `skills/spaceport/SKILL.md` | The row was created from the coverage auditor's findings after the review rounds ended |
| S01 | `crates/sheprd-protocol/tests/fixtures/`; tests sampled by grep only | Reviewer read production files |
| S02 | `crates/sheprd-client/src/tests*` (about 5,300 lines) beyond grep for field use; `crates/sheprd-client/tests/live*`; `lib.rs`, `error.rs` | Reviewer concentrated on the session worker |
| S02b | `transport/windows.rs`, `transport/overlapped.rs`, all tests in scope | Stated by reviewer |
| S03 | `upload_tests.rs`, `script_tests*`; Windows and Linux `cfg` paths checked by symbol grep only | Stated by reviewer |
| S06 | Tests grepped only | Remaining production files were covered by G4 |
| S07 | `attach/downgrade.rs` was covered by G4; `mirror/transcript.rs` beyond line 330 except `parse` | Stated by reviewer |
| S11 | `crates/sheprd-client/src/forward.rs:241-371`; `crates/sheprd-remote` beyond `host_script.rs`; `gpui/usage/` panel code beyond `details.rs` | Stated by reviewer; lowers confidence in SIFT-S11-02 |
| S13 | The per-crate mappings under `gpui/i18n/errors/*.rs` (listed, not read) | Stated by reviewer |
| S14 | `examples/sparkle_fixture.rs`, `tests/sparkle_lifecycle.rs`, `test-support/pe.rs`, `crates/embed-resource`, `src/updates/native.rs`; `smoke_sidebar.rs`, `smoke_hosts.rs`, `performance.rs` read in part | Stated by reviewer |
| S15 | `gpui/config/{watch,files,sidebar,chat*,keybindings,layout,devices,backdrop,padding,accent,zen,omarchy,status_icons,tab_layout,option_as_alt,favorite_themes,worktrees,agents_placement,agent_skill,tab_labels,error}.rs`; `config/tests` (6,800 lines); `window_state.rs` beyond line 200; `storage_warning.rs`; `config-gpui.example.toml` | Neither S15 nor G7 opened them |
| S16 | `endpoint/lifecycle_tests` (9,500 lines) beyond assertions on status strings | Sampled for intent only |
| S17 | `core_terminals/{workspace_document*,workspace_ui/render,fallback,infos,observation/remote}`; all test bodies | Neither S17 nor G7 opened them |
| S18 | `gpui/{scrollback*,contrast*,fonts,font_picker,presentation*}`, `terminal/{links,selection,accessibility,splits}` bodies, `terminal_painter/images.rs` beyond line 200 | Neither S18 nor G8 opened them |
| S19 | `window/tab_strip/` submodules, `tab_layout/` variants, `tab_layout.rs` beyond line 200, `tab_menu.rs` in part, `panel_resize/grip.rs`, `window/tab_start*` | Stated by reviewer |
| S20 | `gpui/{truncate,input/weak,link/follow}`, `window/select_hint*` | Neither S20 nor G8 opened them |
| S21 | `window/{reconnecting,monitors,sound}` and the window test modules | Not named in either round |
| S22 | `chat/view/{rail,panels,viewer}`; `pinned`, `jump`, `disclosure`, `entries/steps` by outline only | Neither S22 nor G5 read them in full |
| S23 | `chat/preview*` (1,300 lines, test-only feature), `chat/{glide,highlight,page,pasted}` by outline only, `chat/selection.rs` beyond line 120, `chat/start/native.rs`, `chat/tests` (9,600 lines) | Neither S23 nor G5 read them in full |
| S24 | `sidebar/{accessibility,slide,wash,metrics,note_line,sticky}`, `sidebar/layouts/{minimal,orca,superset,parts}` bodies, `sidebar/tests`, `layout_tests` (8,900 lines) | Not opened in either round |
| S25 | `menu/devices/{error}` in part, `menu/devices/tests` | Stated by reviewer |
| S26 | `menus/in_window/render.rs`, `pane_menu/{moves,processes}.rs`, `menu/{page_title,whats_new,announcement}` tests | Stated by reviewer |
| S27 | `settings_window/{ui/*,controls/fonts,controls/backdrop*,remote_history,navigation,scrollbar,launch_at_login,themes/grid}`; `settings_window/tests` (4,400 lines); `gpui/toggles*` | Neither S27 nor G7 opened them |
| S28 | `overview/{view,marks,hosts,presets}`, `palette/{render,preview,projects,project_create,keys,canonical}`, `keymap/{host,error}` | Neither S28 nor G7 nor G8 opened them |
| S29 | `gpui/{about*,bundled*,app_icon,icon_font_notice*}`, `agent_skill.rs` outside lines 185-240 | Neither S29 nor G8 opened them |
| S30 | `sound/{config,playback}`, `motion/slow`, `gpui/{backdrop,icons,ago,caption*,accessibility,progress,bell}`, `window/system_notifications.rs` sampled only | Neither S30 nor G8 opened them |
| S31 | `browser/{groups_view,layouts,tab_body,tab_scroll,tab_move,view/agent_tab,view/requests}`, `view_tests` | Stated by reviewer |
| S32 | Most of `scripts/release/` (133 files): only `README.md`, `check-ci.py`, `build-macos.sh`, `verify-macos-preview.sh`, `preview-identity.py`, `common.sh`, `sparkle.json`, `package-macos.sh`, `build-icon.py` and four tests were read in full; `release.yml` beyond line 215; `cliff.toml`, `deny.toml`, `homebrew/`, `sheprd.iss` | The dead-or-live question was answered from workflow guards and callers, not from reading each script |
| S33 | `scripts/{check-sorted.py,check-icons.swift,lint-linux.sh,stress-linux.sh,rich_chat_*.py,check-rich-chat-native.py,capture-core-contract.py,ubuntu-ci-sources.py,install-herdr-windows.py,install-linux-deps.sh,verify-release.sh,gpg-sign-release.sh}`, `scripts/git/*` | Neither S33 nor G8 opened them |
| G5 | `chat/rich/{math,math_input}.rs`, `chat/visual/remote.rs`, `view/widgets/components.rs`, `view/rich/rhythm.rs`; five of eight layout variants by outline only | Stated by reviewer |
| G6 | Most of the row: `gpui/state.rs`, `sidebar/{layout.rs,row*,agents*,tokens*,animated*}`, `gpui/{worktree_identity*,worktree_notes*,worktree_banner,device_tags*,devices_overview*,theme_picker*,close_modal*}`, `menu/{sessions/management,worktree_note*,worktree_shortcut*,version_mismatch*,open_link*,mute*,colors*}`, `menu/devices/{wsl,core_catalog,picker,rail,activity_card,scope}*`; `host_menu.rs`, `sessions.rs`, `shortcut_editor.rs` by struct heads only | The reviewer read the workspace and worktree-open flow in full and little else. This is the weakest row in the audit. |
| G7 | `settings_window/themes*` and `config/theme.rs` read in part only | Stated by reviewer |
| G8 | `titlebar/{navigation,status,tabs}`, `caffeine/{error,work}`, `taskbar/{native,button*}` | Stated by reviewer |

To close this ledger, the rows worth a further pass, largest first, are G6, S32, S15, S27 and S24.
