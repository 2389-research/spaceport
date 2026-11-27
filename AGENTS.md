# AGENTS.md

Engineering guidance for agents working in this repository. Read this alongside
`README.md`, the relevant crate documentation, and the code before making changes.

## Spaceport fork specification

The authoritative Spaceport scope and task graph live in
[`docs/spaceport/`](docs/spaceport/START_HERE.md). Read that package's
`AGENTS.md`, `SPEC.md`, and `SYSTEM.md`; run its tools from `docs/spaceport/`.
Keep the upstream engineering rules below where they apply to retained code.
Where they describe excluded Sheprd behavior, the Spaceport specification
governs the fork.

The standalone iOS and Android clients share a mobile workstream under [mobile/](mobile/README.md).
It connects directly to an installed Herdr TUI over SSH. Keep its build outside the desktop release pipeline.
The desktop fork specification's mobile deferral still applies to the retired phone-control service.

Decision D60 removes the inherited custom updater. App version reporting and
the unconfigured-production-release refusal live in the app's constants;
verified platform/package update replacements remain a T12 release gate.
Decision D61 adds an explicit macOS signed preview route as `ai.2389.spaceport`;
ordinary development remains `spaceport.development`. D64 separately authorizes
tagged preview publishing and D65 selects Sparkle for installed macOS previews;
development/worktree builds remain excluded. Neither enables the inherited
production release pipeline. See
[`t12-macos-signed-preview.md`](docs/spaceport/evidence/t12-macos-signed-preview.md)
for actual artifact qualification and remaining gates.

## Priorities

- Keep changes small, explicit, and focused on the root cause.
- Preserve user and other agents' changes. Never revert unrelated edits.
- Prefer existing standard traits and generics over unnecessarily concrete APIs.
- Protect protocol compatibility, bounded resource use, and UI responsiveness.
- Add regression tests for changed behavior and report what was actually verified.
- Never modify or patch Herdr itself. It is owned by an unrelated external team;
  sibling Herdr checkouts are read-only references. Propose an upstream PR only
  when genuinely justified, never make incidental upstream edits for this client.

## Architecture

Spaceport is a native client of Herdr 1 and herdr-core, not a TUI wrapper.
Herdr 1 supplies terminal cells; herdr-core supplies raw output emulated by
`spaceport-terminal`. The daemons own terminal processes and session state.
Closing or detaching the GUI must leave the daemons and their terminals running.

| Location | Responsibility |
| --- | --- |
| `crates/spaceport-protocol` | Generation-1 wire types, framing, validation, and atomic surface patches |
| `crates/spaceport-acp` | `sheprd-acp` binary run inside a Herdr pane: the only ACP client of an agent, its in-pane chat view, the attach socket Spaceport connects to, and Herdr status reports |
| `crates/spaceport-chat-protocol` | Attach wire between `sheprd-acp` and its clients: transcript model, the shared change reducer, framing, and endpoint discovery; no ACP or GPUI types |
| `crates/spaceport-release-notes` | The daemon's release notes and product announcements as bounded, display-safe lines: `parse` says what each line is — heading, bullet, numbered, quote, rule, table, paragraph, fenced code — and which byte ranges of it are code, strong, emphasized or links, admits only `http`/`https` links, and drops the control and direction-override characters an untrusted body carries; the app paints every line and words the note saying what a body's bounds left out, and a chat's rich text and compaction summary read the same model; no GPUI |
| `crates/spaceport-chat-core` | The chat model below the app's views: which existing agent conversations a window may attach to, which daemon's chat programs it may reach and how, the `[chat.env]` table a chat's tab is created with, how the measured entry list follows a transcript, the attach side — the transcript a session holds and the blocking worker that reads it — the block model a message's Markdown is parsed into with the parse that reports the diagrams and pictures it admits as wants, the identity, bounds and diagram theme of the pictures a transcript shows, the entries a view draws prepared from them, with the words `prepared` and `text` read from `spaceport-i18n` in the language shown, the catalog agent JSON is read into as values that can name no code, URL or command, the syntax spans of the code a transcript shows, how long a step or turn took, which of an agent's answers is a compaction summary, and the selectable text of a transcript with the `Region`, `Caret`, `Document` and `Segment` selection over it, copied within `spaceport-client::text`'s 4 MiB bound; no GPUI (the app keeps the views, the pictures, the retry policy and every message) |
| `crates/spaceport-client` | Discovery, socket worker, session transitions, ordered commands, client-local activity projection, and the navigation target a focus request aims at with the rules a snapshot answers it by |
| `crates/spaceport-backend` | The terminal value types herdr-core terminals share with the app, and the `Backend` (attach) and `Terminal` traits `spaceport-core-client` implements, as `spaceport-protocol` cells and semantic input; no GPUI or transport |
| `crates/spaceport-browser` | Native browser tabs and a window's groups of them: what a tab shows, its saved file, the split layouts, the strip's growth and scroll, and recently closed tabs; it depends on GPUI for `WindowId`, `ScrollHandle` and `Pixels`, and the app owns every message and draws every strip |
| `crates/spaceport-web-url` | The bounded http or https address with a host that browser tabs, terminal links and chat share: `WebUrl`, checked once where an address is read from a page, a saved tab or a transcript, and saved as the string it came from; opening one is the app's, and no message lives here |
| `crates/spaceport-panes` | The panes one herdr-core tab splits into: a binary tree of terminals with each split's share and the rects and neighbours they lay out as, saved with the tab, and the cell map a daemon's reported frame rectangles make only when they tile it exactly once; no GPUI |
| `crates/spaceport-core-client` | herdr-core (`hc`, Herdr 2): endpoint discovery, the framed JSON-RPC connection worker, credit-flowed terminal and event streams, starting a core, and `CoreBackend`, which lists, watches, starts, ends, and attaches terminals in `spaceport-backend` types; no GPUI |
| `crates/spaceport-terminal` | The terminal emulator for herdr-core's raw byte streams (alacritty_terminal): bytes in, `spaceport-protocol` cells and damage out, and semantic input encoded for the program's modes; no GPUI or I/O |
| `crates/spaceport-textfield` | The app's text fields: the single-line field a search box is built from — its own caret, selection, scroll, IME plumbing and paint, with its appearance, placeholder, label and element name given by the caller — and the editing model the app's own `DialogInput` and `MultilineInput` share: a grapheme- and bidi-aware caret and selection, the UTF-16 mapping the platform input API speaks, newline normalization, which edit a key asks for, and the platform input handler GPUI is handed; it declares Cut, Copy, Paste and Select All, the keystroke an Edit menu item replays and the `LocaleChanged` notice a field formats its placeholder again on, and `test-support` opens the model and the key-table driver to the app's tests; no window, no message and no word |
| `crates/spaceport-icons` | The artwork the app draws — each agent's mark, each status set and the UI glyphs, plus each agent's full-colour brand mark with the theme's monochrome fallback (`AgentIcon::sidebar_element`) — embedded at build time from `assets/icons` at the repository root, and the one `gpui::AssetSource` that serves them by asset path; no app state and no messages |
| `crates/spaceport-sidebar` | The sidebar's core: the geometry its rows are measured with, the density and style a layout name pairs, the four row layouts and the mode that picks one, the rows the daemon's configured tokens resolve to, the order its workspaces and agents list and step in, which devices it lists and where each one runs, the host headers pinned above a scrolling list, the colour wash a host lays over its rows, which workspace rows a folded group shows, what a row is — its kind, its place in a worktree group, and how its two lines are weighted — the status sets drawn rather than loaded and the props their marks are drawn with, the brand mark each agent's row opens with, taken from `spaceport-icons`' `sidebar_element`, the local state it draws from — the notes on checkouts and the tags on devices, with their bounds and saved shape — and the interaction it answers: the two drags that reorder workspaces and devices, what a resting pointer claims, and how far the sidebar slides; it measures and paints in GPUI's values, mirrors a row when the shown language reads right to left, words the one message it shows (`sidebar-primary`) itself, takes the app's label painting and moving marks as `paint::Painters`, and no piece of it holds a window: the app owns every message, the copy every window reads and the file it saves them in, and words every status |
| `crates/spaceport-widgets` | The chrome's shared widget kit: the logical start and end sides every styled element is given, with the arrow and icon that point along the line, the selection marks dialogs, menus and settings draw (checkbox, radio and switch), the configured face an element that draws in a font is given, the popover-appearing motion, and the AccessKit helpers GPUI's own setters cannot express; a GPUI crate that holds no window, no words and no message, and that leaves the mirror rules where they are — `spaceport-sidebar`'s `direction`, keyed on the `bool` a row's painter can read — while the app publishes the shown locale's direction as `i18n::current::replace` loads the catalog (its own thread's under the `test-support` feature, so one test's pseudo-locale mirrors only its layout, and a test build sees a popover in place), words every label, and keeps each module at the path the chrome has always named it by |
| `crates/spaceport-gpui` | Native UI, connection bridge, presentation state, semantic input, geometry, and painting |
| `crates/spaceport-control-protocol` | The control socket between `spaceport <command>` and the running app: request and response types, bounds, and the Unix socket or named pipe; no GPUI |
| `crates/spaceport-commands` | The command catalog: each command's name, default shortcuts per platform, and the daemon request it becomes against a snapshot; no GPUI (the app binds the keys and runs window commands) |
| `crates/spaceport-remote` | The bounded host shell that listening-port inspection runs its steps through, existing bounded Herdr host scripts, agent-session validation/arguments and pane/process snapshot readers retained by chat and pane movement, and the saved device catalog's claim, remove, rename and label through the installed `herdr` CLI; no migration, provisioning, dispatch, credential store or GPUI |
| `crates/spaceport-diagnostics` | The app's log file and crash reports: the bounded capture of the crates the app names, the writer thread and rotation, the reader the log window tails, the panic hook, and Windows minidumps; no GPUI |
| `crates/spaceport-endpoint` | One host as the app drives it, and the endpoint program's policy, below the app and GPUI: `Host`, the link machine that is one attempt after another with the failure that ended it and the backoff it earns, the live state its connection last reported, and the typed `Action` a transition comes back with (detach, retire for another target, prove the transport live) for the window to perform on the socket it keeps; the phase and the predicates a command, picker or row reads off that link; the values that name the connection a deferred action started on, with the comparison that says whether one still names another; the rule that the selected host reads the window's stamped copy of its state; one host as a sidebar row paints it, its `HostView` of identity, label, phase, status and folds carrying no target or bridge; the saved-device catalog its two-second loads and one-at-a-time selection writes keep, the device list those loads describe and the entry comparison that retires an edited one, the failed client storage step a retry names once, the Back and Forward trail through the panes a host has focused, told from the daemon's snapshots and dropped at a new boot, when the machine slept or its network came back or changed (from the two clocks and, on macOS, the system's path monitor), where a dragged device lands in the user's own order, which of a background host's moves is someone asking for a focus, and the IDs saved hosts are keyed by; no window, socket, message or word: the app keeps the views, the connections, the endpoint list it reconciles and every `fl!` site |
| `crates/spaceport-i18n` | The languages the app shows (`docs/i18n.md`): `Locale`, `Direction`, the `language` setting and its negotiation with an `en-US` fallback, the Fluent catalog, the `fl!` lookup with its compile-time `check`, the `Localize` trait each crate words its own errors through, and the `en-XA`/`en-XB` pseudo-locales; no GPUI (the crate embeds its own `locales/`, the app keeps the current locale as a GPUI global, publishes the shown direction, and reaches the lookup through a facade) |
| `crates/spaceport-keymap` | The keymap below the app, and the one answer it resolves: the keystrokes bound to each catalog command — the catalog's own defaults, Herdr's `[keys]` table as this client reads it (its prefixes, each action's entries and Herdr's default for it, the `[keys.indexed]` combos over the digits 1–9, and the keys that move the workspace picker) and the GUI's `[keybindings]` layered over it, with the `[pane_keys]` a pane takes — the queries the window reads off that answer (`shortcuts`, `primary`, `bindings`, `prefix_label`, `is_prefix`, `chord`, `triggers`, `navigates_workspace`, `pane_key`) and the rule that two spellings name one keystroke; the daemon's `[[keys.command]]` shortcuts matched against it when typed, with the `Reach` that says whether one runs here; the keystrokes the app reserves for its own Edit menu and macOS system keys, which no custom command takes; whether a keystroke reaches a pane as a key event, and Herdr's `+`-separated spelling turned into GPUI's, with the host OS the text is parsed and printed for; a GPUI crate naming `Keystroke` (`parse_with_os`, `unparse`, `should_match`), `Modifiers`, `OperatingSystem`, `KeybindingKeystroke` and the parse failure it returns, and none of GPUI's runtime, holding no window, element, entity, message or word — the app keeps prefix mode, the GPUI bindings, the pane input encoder and words every failure |
| `crates/spaceport-ago` | How long ago something happened, in the largest whole unit ("3 min ago", "yesterday") from seconds elapsed; it names the unit and its count and the app writes the words; no GPUI or messages |
| `crates/spaceport-motion` | The clock animations are measured by, and how long each takes: `now`, `duration`, `progress`, `ENTER`, `LEAVE`, and the process-wide QA > Slow Animations switch that runs every animation at a tenth speed; no GPUI |
| `crates/spaceport-lenient` | Reads Herdr's `config.toml` leniently: a value this build cannot read (a variant added since, a key whose type changed) falls back to its own default instead of failing the whole file, while malformed TOML still fails; no GPUI |
| `crates/spaceport-activation` | The activation fence both clients drive one Herdr daemon with, below either of them: `Fence`'s lifecycle for one surface activation and `Transition` as its only writer, the `Failure` causes that distinguish a barrier the ordered queue refused from one that was submitted and never answered, `TIMEOUT` for the one barrier, and `ROUTE_BUDGET` for a walk that reaches a pane one level per request as `Route` and `Step`, refusing to replay a step; it also owns the `NavigationTarget` a focus request is aimed at and the `SurfaceActivation` one barrier was asked for, and the client keeps its own projection, its own events and every word it says about a failure — the phone takes it instead of `spaceport-live`, which would drag `rodio` onto iOS; no GPUI |
| `crates/spaceport-live` | A connection's live state below the window that drives it: what one Herdr daemon last said — its snapshot, its surface, its images, its notices and the mailboxes the window drains — with `apply`'s ordered-event reduction, the rule that a snapshot and a surface belong to each other, the one newer projection that waits for its cells, and — re-exported from `spaceport-activation` at the paths the window already names them by — the activation fence and navigation target that keep navigation off an unacknowledged projection; its notices word themselves through `spaceport-i18n` and take the two words it cannot — a client error's sentence and what an agent notice is about — as `words::{Localize, Describe}`, the window's own copy of the selected endpoint's state stays the window's, and no paint, socket, effect or message lives here; no GPUI |
| `crates/spaceport-menu` | The menu below the app and the chrome it is drawn in: the closed set of popup pages and workspace actions, the workspace, tab and pane a row targets with the snapshot fence each still answers to, the `(Method, Value)` request a row sends, the rows a menu offers once the endpoint and this machine are counted, the state machine that brings a popup in and takes it out again with what a closing page reads held until it has gone, the device picker's lines with the session rows and notes that fold under a device, the measured panel with its placement, surface and the layer that keeps what it covers inert, and the in-window menu bar with the menu a heading drops, the name every popup page is announced by with its titles, the host command table, and the one row kit the app menu, the tab menu, the pane menu and the group menu draw (label, dimmed detail, mark and selector, #256); the layout and the paint take a page's own element and the words and presses only the window knows (the `BrowserHosts` shape), so the app keeps the window's menu state, the dimming, its keys and focus, every page body that is still the window's and every word, and words an action's variant itself |
| `crates/spaceport-palette` | The command palette's pure model: what a row is — the label, detail, badge and destination it carries with the identity that makes two rows the same place — which rows each filter lists, how a typed query scores and highlights them (nucleo's fzf algorithm over bounded haystacks), Go To's agent-status filter with the heading order it restores, the destination context every request is fenced by, the key a row's frecency is remembered under, and the errors a chosen row answers with, plus the navigation target a Go To row aims at as its interim home; no entry is built, nothing is painted and no word is said — the app builds every row from its own catalog and daemon state, words every label and badge, and keeps the palette entity, the frecency history, the project scan and the paint |
| `crates/spaceport-reorder` | Where a dropped row or tab lands, and how far the others slide to preview the drop: lift delays, `Beside`, `Slide`, `slot_for` and `preview`, shared by the sidebar's rows and the tab strips; no dependencies and no GPUI |
| `crates/spaceport-truncate` | Cutting text the app shows to a length between grapheme clusters (`cut`, `cut_chars`, `cut_width`, `one_line`, `ellipsize`, `tab_title`); it drops the controls and direction overrides untrusted text carries through `spaceport-client`; no GPUI |
| `crates/spaceport-textfield` | The single-line text fields the app's dialogs and panes share: the search input and the caret, editing and key handling under it (3,700 lines moved out of the app), with the element name parameterized and edit actions declared once; the app keeps `DialogInput` and `MultilineInput` behind the facade, every message, and the multi-line fields; no GPUI-facing window state |
| `crates/spaceport-launcher` | Windows-only windowed launcher that starts `spaceport.exe` without a console window; std plus `MessageBoxW` for start failures |
| `crates/spaceport-osc52` | The daemon's forwarded OSC 52 clipboard payloads: bounded base64 text decoding, and the mailbox bound; no GPUI |
| `crates/spaceport-sound` | The notification-sound policy and the worker that plays it: Herdr's `[ui.sound]` and toast delay read leniently, when a notification sounds, and rodio playback of a bounded custom or bundled MP3; the app owns the settings path, the volume setting and every message; no GPUI |
| `crates/spaceport-state-file` | Small app-owned JSON state files: a bounded read, an atomic replace, and a worker that saves the latest value so a change made on the UI thread never waits for the disk; callers resolve their own paths, and the app owns every message; no GPUI |
| `crates/spaceport-settings` | Reads and writes the settings files: the typed values they carry — sidebar, layout, status bar, terminal padding, tab and panel layouts, agents, fonts and weights — with their defaults, validation and the contrast rules the app draws with; the app owns every message and everything that acts on a value; no GPUI (callers run its blocking load/save on a worker) |
| `crates/spaceport-settings-window` | The Settings window's paint kit: the row a page is built from — its id, title, description, tag and controls, as a group a screen reader reads in context — the toggle, button and segmented strip it holds, the number field with its typed entry and drag slider, the dropdown with the floating list it opens, and the SSH host colour row; it records where each row was painted so search can scroll to it, and it words every message it draws itself through `spaceport-i18n`, reading an id that arrives as data (a catalogued row's title) through `message`; the pages, the controls, the theme browser and the `SettingsWindow` entity stay in the app |
| `crates/spaceport-words` | Word boundaries (UAX #29) for the single-line text fields, so Option-Left, Option-Right, Option-Backspace, and Option-Delete move and delete by word; no GPUI or messages |
| `crates/embed-resource` | No-op stand-in for GPUI's `embed-resource` build dependency, patched in so GPUI embeds no Windows manifest beside `crates/spaceport-gpui/windows.manifest`; not a production crate |
| `crates/test-support/sandbox.rs` | Shared isolated process setup for opt-in integration tests; not a production crate |
| `crates/test-support/pe.rs` | Windows executable (PE) reader for tests that check subsystem and resources; not a production crate |

- Keep dependency direction from UI to client to protocol. Protocol/client code must not depend on GPUI.
- Keep session transitions separate from socket scheduling, and connection ownership separate from window rendering.
- Reuse `ConnectionBridge`, domain targets, geometry helpers, and the test sandbox rather than duplicating their policies.
- **File size limit: 1,000 lines.** No Rust, Python, shell, or Swift file may exceed it,
  tests included; `just check-file-size` enforces it in `just ci` and CI's checks job.
  There is no allowlist: when a change would cross the limit, split the file by
  responsibility in the same change rather than adding an exception.
- Split modules by responsibility; the line limit is a ceiling, not a reason for arbitrary cuts. Prefer wiring and exports in entry points as code grows; do not perform unrelated file reshuffles.
- Keep APIs narrow. Use private items by default and `pub(super)` or `pub(crate)` only where needed; do not expose every field merely to ease extraction.
- Define shared types once in the lowest appropriate layer and re-export them. Do not create mirror enums or convert between them through strings.
- Do not add a crate, runtime, service container, or speculative transport abstraction without a concrete need. Sibling checkouts are references, not build dependencies.

## Parallel Pull Requests

Many pull requests are open at once, so code that every feature appends to is
a merge-conflict hotspot. Most past conflicts were two branches adding lines
at the same spot, or both bumping one hard-coded count.

- Do not assert hard-coded totals such as `assert_eq!(list.len(), 55)` or
  fixed array lengths like `[T; 98]` for lists features grow. Compare against
  the source list, assert uniqueness or membership, or use a slice.
- Declare each item of a growing list once. Prefer one table, or a
  `macro_rules!` table, that generates the matching `match` arms and `ALL`
  lists over parallel lists edited in several places.
- Keep lists whose order carries no meaning (enum variants without derived
  ordering, asset tables, registries) sorted between `// keep-sorted start`
  and `// keep-sorted end` markers. Additions then land on different lines.
  `just check-sorted` (part of `just ci`) enforces the markers and `just sort`
  fixes them. Never mark a list whose order is shown to users or is a wire
  contract, such as menu rows or the vendored bincode types.
- Give a new feature's failures their own `thiserror` enum named `Error` in
  its module's `error.rs`, with variants not prefixed by the feature name, and
  add one `#[error(transparent)] Feature(#[from] feature::Error)` variant to
  the crate's root `Error`, as `pane_move` and `backdrop` do, rather than a
  run of variants in `error.rs`. The root enum stays sorted. Callers name it
  `feature::Error`, never through an `as FeatureError` alias.
- Wrap another crate's error the same way (`Client(#[from] spaceport_client::Error)`).
  Do not mirror its variants in the app's root and convert one by one: each
  failure has one definition, and callers match the nested variant.
- Shared state structs (`SpaceportWindow`, `Config`, `herdr_settings::Settings`)
  and their constructors keep their fields sorted. Run an initializer that has
  an effect or depends on another (registering a subscription, creating an
  entity, reading a value before it moves) in a `let` before the struct
  literal, so the literal itself has no order to preserve. Derive `Debug`
  rather than listing fields by hand; redact a private field through its own
  type's `Debug`.
- `just hooks` also enables `rerere`, so a resolution is reused when the same
  conflict returns on another branch, and a `Cargo.lock` merge driver that
  keeps one side and has Cargo rebuild it before the commit. After a clean
  merge whose only conflict was the lockfile, the merge stops with the rebuilt
  lockfile staged: run `git commit --no-edit` to finish it. The rebuild
  refuses while a `Cargo.toml` or `Cargo.lock` change is unstaged, and an
  aborted merge's pending rebuild is dropped with a warning. Squash merges
  rebuild like merges. Cherry-picks, reverts, and rebases commit without
  these hooks, so they stop with `Cargo.lock` conflicted: run
  `cargo metadata`, `git add Cargo.lock`, and continue. GitHub's Update
  branch button runs no hooks; there, run `cargo metadata` and commit
  `Cargo.lock` by hand.

## Command Line

`spaceport COMMAND` drives the running app over its control socket, so
agents in Herdr panes can use what only this client shows or keeps
(`docs/cli.md`). Agents learn about it from the managed skill in
`skills/spaceport/SKILL.md`, which the app installs and refreshes.

- When a feature adds something an agent or script could reasonably ask for
  (open or focus a view, read state the app holds, change a setting, wait for
  an event), add a command in the same change: the request in
  `spaceport-control-protocol`'s `protocol`, its handler in `control/server`,
  parsing and output in `control/client`, its `--help` text, and tests for
  each.
- Do not add a command that repeats Herdr's own `herdr` CLI (workspaces,
  terminal tabs, panes, agents, scrollback), that the app cannot really do,
  or that skips a confirmation. Destructive or outward actions open the
  app's own dialog, pointed at the caller's checkout or the machines named,
  and the user confirms there; a batch of them is confirmed once.
- Keep the shared contract: `--json` and `--window` on every command, the
  exit-code table in `docs/cli.md`, bounded requests and responses, and
  targeting that refuses rather than acting in another session's window.
- Add the command to `docs/cli.md`. When an agent should reach for it
  unprompted, also add it to the skill's command table and, if it is a new
  kind of task, to the skill's `description`, which is all an agent sees
  before loading it. Keep the skill's `name`: installed copies are found
  by it.

## Rust Conventions

- Require only the capabilities an operation uses. Prefer existing traits such as `Read`, `Write`, `IntoIterator`, `AsRef<Path>`, and `AsRef<OsStr>` over specific streams, collections, or owned paths.
- Use slices, `&str`, and `&Path` when borrowing is sufficient. Keep owned data where it must cross a thread boundary or outlive its caller.
- Prefer `impl Trait` or named generics for static dispatch. Use `dyn Trait` for actual runtime heterogeneity/type erasure, not by default.
- Use `FnOnce`, `FnMut`, or `Fn` for injectable behavior. Do not invent a callback or clock trait when a closure or explicit time value suffices.
- Do not introduce custom traits merely to wrap one struct. A new trait needs a concrete behavioral contract and a real implementation/substitution need that existing traits cannot express.
- Use enums for closed alternatives such as navigation targets, pane/popup input destinations, connection phases, and launch modes. Avoid string dispatch and boolean-plus-ID pairs internally.
- Parse strings at boundaries, then match on domain types. Prefer `From`, `TryFrom`, `Into`, and `TryInto` for conversions.
- Prefer typed request/result structures when shapes are known. Preserve genuinely open-ended protocol envelopes rather than forcing a speculative schema.
- Derive standard traits such as `Default`, `PartialEq`, and `Eq` where their semantics are valid. Compare complete values when deduplicating operations.
- Every crate must expose its own root `Error` and `Result<T, E = Error>` (`pub use error::{Error, Result}`), with errors defined using `thiserror`. Use meaningful typed variants for internal failures and match on variants, never error text.
- Use `anyhow::Result` and context at application/reporting and test-harness boundaries, not as a replacement for typed internal errors. Keep dependency versions in workspace dependencies.
- Never use `String` or `&str` as an error type, including `Result<(), String>`, worker-channel results, and intermediate operation state. Strings are allowed as diagnostic payloads, unchanged wire-protocol fields, and final UI display text, not as catch-all internal error wrappers.
- Propagate errors with `?`; preserve original causes with `#[from]`/`#[source]` and retain structured operation/path context until the display boundary. Do not flatten sources with `to_string()` or `format!()` during propagation. Keep `io::Result` where an actual I/O trait or connector contract requires it, preserving typed sources when adapting errors.
- Test error variants and source chains as well as user-facing diagnostics. Preserve redaction, bounded remote diagnostics, retry/cancellation classification, and protocol compatibility when changing error handling.
- No `unwrap()` or `expect()` in production. Test-only allowances must be scoped to test code.
- A bare `#[test]` under a wildcard glob of a crate that exports its own `test` macro resolves to that crate's macro, not std's — `use gpui::*` anywhere in the module's ancestor chain is enough, and on Linux its expansion can overflow the compiler's stack while macOS compiles the same file. In such modules spell `#[::std::prelude::v1::test]`. If a recursion-limit error's requested limit grows with every bump, the expansion is unbounded: stop raising the limit and find what recurses.
- Prefer safe Rust. Do not introduce `unsafe` unless there is no practical safe alternative; investigate standard-library APIs and maintained safe wrappers first. Any unavoidable exception must be tightly scoped and document both why safe alternatives are insufficient and the safety invariants.
- Prefer guard clauses and readable iterators. Avoid clones, allocations, helper layers, and generic parameters that provide no benefit.
- Comments should explain invariants, ownership, or non-obvious decisions, not narrate assignments.

## GPUI Rules

- Treat app, window, and entity contexts as UI-thread execution. A foreground `spawn` future does not make blocking work safe.
- Never perform blocking socket/disk I/O, process waits, sleeps, or expensive background computation on the UI thread. Use the existing worker threads or GPUI background executor, then apply results through the appropriate context update.
- Start child processes with `spaceport_client::new_command`, never `Command::new` (clippy rejects it outside tests). On Windows a console child built any other way can open a window of its own.
- Keep render and input paths thin. Render from prepared state and bounded caches; do not query the daemon, scan files, or launch processes while rendering.
- Preserve continuously drained ordered events and the coalesced latest-state mailbox. Do not replace them with an unbounded UI event queue.
- Route connection/startup failures through the authoritative inbox. Reconnect and detach must isolate old inboxes so delayed events and paint acknowledgements cannot affect the new connection.
- Derive connection text/indicators from `ConnectionStatus`. Old local operation errors must not hide a disconnection reason.
- Queue success is not daemon acknowledgement. Keep names such as `last_queued_options` honest and include cell pixel metrics in resize comparisons.
- Use the same geometry for painting, hit testing, and IME placement. Popup input and composition bounds must target the popup, not the underlying pane.
- Keep glyph visibility separate from decorations: spaces and wide-character continuation cells may still need underline/strikethrough painting.
- Preserve menu input isolation, focus behavior, Unicode composition, and semantic input routing. Do not synthesize terminal keys for operations with endpoint API methods.
- Activity indicators come from the daemon's snapshot, not guessed terminal output or a client-local seen ledger. Every client shows the same dot; do not reintroduce per-client acknowledgement without asking.
- Animate UI that appears, disappears, moves, or changes in response to a click or hover when motion helps the user follow it: tabs, panels, popovers, revealed controls. Animate both ways, so what slides in slides back out, and reverse from where it is when interrupted. Skip it where motion would only delay the result, such as typing, scrolling content, or terminal output.
- Time animations through `crate::motion`: read `motion::now()` instead of `Instant::now()` for start stamps and progress, pass GPUI `Animation` lengths through `motion::duration`, and prefer `motion::ENTER`/`LEAVE`, `progress`, and `Toggle` over new curves and constants. That keeps motion consistent and lets QA > Slow Animations slow every animation tenfold.

## Protocol And Safety

- Vendored bincode field and enum variant order is a compatibility contract. Do not reorder declarations as cleanup. Preserve attribution in `crates/spaceport-protocol/NOTICE.md`.
- Keep frame/response/geometry bounds, strict decoding, and atomic patch validation. Test malformed inputs and limits, not only valid fixtures.
- Never mix snapshot metadata and surface cells across boot IDs or projection revisions. Preserve future-surface buffering and stale-boot rejection.
- Preserve the single in-flight API request, bounded command/event queues, FIFO ordering, bounded write batches, and request correlation.
- Partial frame prefixes/payloads must survive read timeouts. Finish an inbound partial frame before dispatching commands against potentially stale state.
- Cancellation must not flush queued work, replay commands, or join a worker on the UI thread. Do not add automatic reconnect/replay as an incidental refactor.
- Crossbeam receiver clones compete for events; they are not broadcast subscribers.
- Treat terminal content and daemon messages as untrusted data. Do not execute terminal escapes, follow graphics file paths, or run displayed update commands automatically. The one exception is the daemon's OSC 52 `ServerMessage::Clipboard`: decode it and put only its bounded UTF-8 text on the pasteboard, never arbitrary bytes, files, or images.
- Spaceport permits separately installed Herdr 1 and Herdr Core under `docs/spaceport/plan/decisions.json` D58. Existing Herdr 1 startup and explicit-socket attachment are allowed. Start a trusted installed local runtime only when necessary; explicit sockets remain attach-only. Never bundle, install, stop, migrate, or upgrade either runtime. Keep exact runtime targets and the upstream no-surprise daemon lifecycle rule; remove excluded feature routes independently.
- Never commit secrets, credentials, private terminal output, or machine-local configuration.

## Dependencies And Features

- Put dependency versions in root `[workspace.dependencies]`; member crates inherit with `{ workspace = true }`.
- Use the pinned `rust-toolchain.toml` and GPUI version. Do not copy toolchain, feature, or release assumptions from Moltis/Arbor.
- Keep workspace lint inheritance and fix warnings rather than widening allowances.
- Keep `integration-test` opt-in. Gate imports, modules, and call sites consistently, and verify both default and all-feature builds.
- Do not introduce Tokio or another runtime to wrap the existing blocking socket worker.
- Avoid unrelated lockfile upgrades. Use `--locked` for reproducible verification.

## Changelog

Users learn what changed from the GitHub release body, which is generated from
git history by `git-cliff` (`cliff.toml`) rather than written by hand. The
macOS preview workflow's `build` job runs `scripts/release/generate-changelog.sh`
and `publish` passes its `RELEASE_NOTES.md` to `gh release create --notes-file`.
`CHANGELOG.md` is generated for each release and is not tracked in git.

- Commit subjects are the changelog entries, verbatim apart from a capitalized
  first letter. Write the subject line for a user reading a release page, and
  put rationale in the body; only the first line is published.
- The type chooses the section: `feat` becomes Added, `fix` becomes Fixed,
  `perf`/`refactor`/`revert` become Changed, `docs` becomes Documentation, and a
  `(security)` scope becomes Security. A scope prints as a `[scope]` prefix, and
  anything non-conventional falls into Changed.
- `chore`, `ci`, `build`, `style`, `test`, and merge commits never reach users,
  so nothing user-visible may ship under those types. A breaking change is the
  exception: `type!:` or a `BREAKING CHANGE:` footer is always published, under
  Changed and prefixed `**BREAKING:**`, whatever its type.
- Previewing the generated changelog before a release is covered by the
  `release` skill.
- These rules are enforced, not advisory. `just hooks` installs a `commit-msg`
  hook that refuses a bad subject before the commit exists, and CI's `commits`
  job checks every commit in a pull request; it is part of the `Format, lint,
  and test` gate. Run `just hooks` once per clone, and `just check-commits`
  to check a branch by hand. Fix a rejected subject with `git commit --amend`
  rather than `--no-verify`.

## Verification

Prefer the existing `just` recipes; `just` with no arguments lists them all,
and [`docs/just-operations.md`](docs/just-operations.md) explains each one.
Before handoff, a requested commit, or a
push, run relevant quick checks expected to finish in under 10 seconds each,
including compilation time. Keep the overall local validation pass short;
defer longer tests, full workspace gates (`just ci`), cross-platform lints,
and release builds to CI jobs. Run longer checks locally only when explicitly
requested or needed to investigate a specific failure. When an operation is
repeated or complex enough that a second agent could not perfectly
reconstruct it, codify it as a recipe in the justfile instead of leaving the
steps in a conversation. Report what ran and
what was deferred to CI.

- For linking, startup, or packaging changes, rely on CI build and CLI smoke checks; use `just test-build` locally when requested or needed to investigate a failure.
- The `windows` CI job checks formatting, lints every target and feature, and
  runs workspace tests with default and all features, including headless UI and
  CLI tests, split like `checks` into parallel `default` and `all` legs. The
  `windows-native` job runs `live_gui` (the `test-gui` and `test-sidebar`
  equivalents) against a pinned `herdr.exe`, and the release-mode performance
  budget, on both Windows runners, except the live-daemon test on
  ARM64, where the emulated daemon ignores Enter; the optimized Windows build
  and its installer run in `windows-package` when their inputs change. Keep the `cfg(unix)`/`cfg(windows)`
  split honest and say plainly what a platform cannot do instead of stubbing
  behavior that appears to work.
- A `cfg` gate is only proven on the platform it names. For changes that touch
  one, rely on the corresponding platform CI jobs: a macOS run cannot see a
  Linux-only gate, and neither can the Windows cross-check. `just lint-linux`
  and `just lint-windows` are available for local investigation when needed.
- Documentation-only changes need command/path/link review and `git diff --check`; do not claim a code test run that did not happen.
- Add deterministic tests for invariants: ordering, cancellation, fragmentation, revision fencing, error paths, geometry, and cleanup. Prefer explicit coordination and bounded waits over timing guesses.
- Never wait on the wall clock in a test. Mock the clock and complete on a
  signal instead: a blocking join on a worker thread, a channel receive, or an
  executor the test advances is explicit coordination, not timing. Give a join
  or receive that could otherwise never return a failure-only timeout, so a
  worker that stops signalling fails the test instead of hanging the run; that
  bound is the one clock read such a test may make.
- Reject sleeps and deadline polling loops, and count-bounded yield spins, which
  are the same flake in a costume. A socket read/write timeout, or the join or
  receive timeout above, that only bounds a failing run is acceptable, and a
  retry or activation timestamp an assertion compares as a domain value is not
  a wait.
- In production prefer monotonic time over the wall clock: `Instant` for elapsed
  time and deadlines, and `motion::now()` only where the elapsed time is an
  animation's, as the motion rule above requires. Justify any `SystemTime` use
  by an actual user-facing need, such as a displayed timestamp.
- Test pure session/CLI/geometry logic without a socket or window where possible. Exercise public behavior with mock peers and headless GPUI tests where integration matters.
- Keep test bodies out of production files, which only declare `#[cfg(test)] mod tests;`.
  Put tests for a new feature in a new topic file under the module's `tests/` directory
  (for example `keymap/tests/pane_keys.rs`, starting with `use super::*;`) rather than
  inserting them into an existing large test module, and keep shared fixtures in the
  parent `tests.rs`. Parallel PRs then edit different files instead of conflicting.
- Fix flaky tests rather than hiding them with retries or new ignores. Existing live/native tests are ignored because they require explicit external resources.
- For visual changes, verify the actual native UI when a desktop is available, including narrow layouts, long labels, focus, popup/IME behavior, and clipping. Headless layout tests do not prove native glyph or OS input correctness.
- Measure interactive performance in release mode (`just run`), not debug mode. Preserve bounded glyph caching and deterministic paint budgets; native timing budgets are machine-dependent.
- Keep CLI parsing pure and based on OS strings so socket paths need not be UTF-8. Invalid options and help must exit before starting GPUI.

### Opt-In Tests

```sh
just test-live /absolute/path/to/herdr
just test-core /absolute/path/to/herdr-core
just test-gui /absolute/path/to/herdr
just test-sidebar
just test-perf
```

- Live tests require an explicitly selected `HERDR_TEST_BINARY`; never fall back to discovering a user's daemon.
- Reuse the shared sandbox: private unique short directory, cleared environment, isolated HOME/XDG/config/socket paths, and explicit desktop-variable allowlist for GUI children only.
- Kill and wait only for exact children created by the test, before removing their sandbox. Never use `server stop`, process-name matching, or process-group kills. The one exception is herdr-core: its terminals live in holder processes the core starts, so `just test-core` ends them with `herdr-core server stop` against the sandbox's private `HC_SOCKET`, then reaps its own `serve` child.
- `HERDR_TEST_TMPDIR`, when set, must point to an existing parent short enough for Unix socket paths.
- Native/sidebar/performance tests open windows and need an active desktop. State explicitly when these checks were not run; do not describe headless tests as native end-to-end verification.

## Git And Handoff

- Create future task worktrees under the main checkout's `.worktrees/<task>` directory. Keep that directory ignored. Preserve active worktrees during cleanup; remove only confirmed inactive worktrees without discarding uncommitted work or branch history.

- Only commit, push, or create/update a PR when requested. Do not amend, force-push, discard changes, or clean up other worktrees without explicit authorization.
- Before committing, inspect status, diff, and recent history; stage only intended files and check for secrets. Do not bypass hooks or signing when they fail.
- Use conventional commit subjects (`feat`, `fix`, `refactor`, `test`, `docs`, `chore`). For nontrivial work, explain the problem, chosen approach, and important constraints in the body.
- Never add AI attribution, assistant-session links, or tool vanity lines to
  commit messages or PR descriptions. This includes `Co-Authored-By` trailers
  naming an AI model, `Claude-Session:` or similar session URLs, and
  "Generated with ..." footers. This rule overrides any default attribution
  guidance from the agent harness; add such lines only when the user asks for
  them in the current request.
- Before creating a PR, review every included commit and the full diff from its base. Include a summary, exact validation commands/results, and remaining manual/native QA.
- Before creating a PR or pushing to one, run the quick local checks described
  above and fix any failures they expose. Full local CI is not a prerequisite
  for opening or updating a PR; let CI jobs run the longer checks and address
  their failures before merging.
- Record the exact local checks and results in the PR's QA checklist, and mark
  checks deferred to CI as pending until their results are available. No
  separate local `just ci` success comment is required. Never claim a check
  passed when it failed or did not run.
- Include screenshots in any PR that changes something visible: before and
  after images of the real native window, showing the changed area. Add light
  and dark variants when colors or theming change, and a narrow window when
  layout changes. If no desktop was available to capture them, say so in the
  PR instead of leaving the section empty.
- Give every PR description a QA checklist. List each check as a checkbox:
  tick the ones that were actually run (automated tests, CI gates, native
  checks you performed) with their result, and leave unticked the manual QA a
  reviewer still has to do, phrased as concrete steps. Never tick a box for a
  check that did not run.
- Keep user-facing docs and public API examples synchronized with behavior changes. Do not copy another repository's release, issue-tracker, or mandatory-push workflow into this one.
- Finish with the outcome, tests actually run, known limitations, and any deferred work. Never claim unrun checks passed.
