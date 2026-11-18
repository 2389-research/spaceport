# Naming endgame: retiring the `sheprd-` prefix

## Scope and authority

Spaceport's product name, dev bundle identity, state paths, control socket,
deep-link scheme, managed skill and display copy are already Spaceport's. The
source tree, fifteen workspace crates, one shipped helper binary and several
compatibility lookups still carry the inherited `sheprd-` name, and two lookup
chains still accept the older `herdr-` generation as a fallback.

This note is a proposal, not a rename. Nothing here changes source, packaging,
evidence or hosting. Doctor Biz asked for a staged plan because the dual-name
debt fails silently: `bundled::locate_in` accepts either generation, so a host
that only has `herdr-acp` works today, while the error that appears when
neither name resolves names one generation only
(`crates/sheprd-gpui/src/chat/start.rs:58`). Removing the fallback would turn
every `herdr-acp`-only installation, SSH host or older bundle into that
message, and nothing in the current code says which name was found. Also, a
repository-wide rename now would collide with
open feature PRs, the deletions in T04/T10, and already-installed previews.

Rules this proposal keeps: `docs/spaceport/AGENTS.md` ("new product branding is
not permission to rewrite external identities", "do not delete by name"), D59
(credit the Herdr and Sheprd teams), and the T01 identity boundary that already
separated Spaceport's names/state/endpoints while leaving production values
gated. Canonical status stays in `plan/tasks.json` and `plan/decisions.json`.

## Inventory

Counts from this checkout (`50b9063be`): 1,603 files mention `sheprd`
(1,135 `.rs`, 256 `.md`). Surfaces below are grouped by who sees them.

### Workspace crates

Fifteen of the sixteen workspace members are `sheprd-*`:
`sheprd-acp`, `sheprd-backend`, `sheprd-chat-protocol`, `sheprd-client`,
`sheprd-commands`, `sheprd-control-protocol`, `sheprd-core-client`,
`sheprd-diagnostics`, `sheprd-gpui`, `sheprd-i18n`, `sheprd-launcher`,
`sheprd-protocol`, `sheprd-remote`, `sheprd-terminal`, `sheprd-usage`.
`embed-resource` and `test-support` are not product crates.

Crate names are not purely internal:

- `sheprd_client` is named in code (`use sheprd_client::…`) and in
  `docs/cli.md`, `crates/sheprd-gpui/README.md` and most evidence files.
- `log_targets.rs` keeps diagnostics by `crate_prefix: "sheprd_"` plus the
  compiler-derived root crate. Renaming a crate without updating that constant
  drops its tracing events with no error.
- Crate directories are cited by path in 236 evidence files and in
  `crates/sheprd-protocol/NOTICE.md`.

### Binary names

| Binary | Package | User-visible |
| --- | --- | --- |
| `spaceport` | `sheprd-gpui` `[[bin]]` | The app and the CLI agents run |
| `spaceport-launcher` | `sheprd-launcher` | Windows windowed start |
| `sheprd-acp` | `sheprd-acp` | Bundled beside the app; agents and SSH hosts run it; `docs/cli.md` documents `cargo install … sheprd-acp --root ~/.herdr` |
| `herdr-acp` | none | Legacy lookup name only; still shipped by older installations and SSH hosts |

### Bundled lookup chains

- `crates/sheprd-gpui/src/bundled.rs`: `preferred_name` maps a legacy
  `herdr-{acp,automate}` request to `sheprd-{acp,automate}`, and `locate_in`
  tries the resolved name and then the requested name, beside the executable
  before `PATH`. The `automate` arm has no workspace crate or binary today; it
  can only match a leftover upstream installation.
- `crates/sheprd-gpui/src/chat/start.rs`: `PROGRAM` is `herdr-acp` and the
  locator upgrades it, so the app's own request is expressed in the legacy name.
- `crates/sheprd-gpui/src/chat.rs`: `remote_program_script` tries `sheprd-acp`
  then `herdr-acp` in `~/.herdr/bin`, `~/.cargo/bin`, `~/.local/bin` and the
  `Herdr.app`, `Sheprd.app` and `Spaceport.app` bundle `MacOS` directories,
  user and system.
- Packaging: `scripts/release/package-macos.sh` and `package-windows.py`
  ship `sheprd-acp` as the only companion; `sign-macos.sh` accepts exactly
  `spaceport` and `sheprd-acp`; `verify-macos-preview.sh` runs the bundled
  `sheprd-acp --help`.
- Herdr-facing identity: `sheprd-acp` announces itself as `herdr-acp` in the
  ACP `clientInfo` name and as the Herdr activity report `SOURCE`
  (`src/session.rs`, `src/codex/bridge.rs`, `src/herdr.rs`).

### Installed skill

`skills/spaceport/SKILL.md` with `name: spaceport`, the marker
`<!-- spaceport-managed-skill v1` and `agent_skill::NAME = "spaceport"` under
`~/.claude/skills` and `~/.agents/skills`. Already Spaceport; `AGENTS.md`
requires keeping the skill's `name`, because installed copies are found by it.
No `sheprd` skill is installed or refreshed.

### Product-name constant

`crates/sheprd-client/src/app_name.rs` holds `APP_NAME = "Spaceport"`, read by
`sheprd-gpui`'s `WINDOW_TITLE`/`HERDR_NAME` presentation and by the
`sheprd-acp` Codex bridge `clientInfo.title`. The constant is correct; only its
crate path is inherited. `crates/sheprd-gpui/tests/removed_updater.rs` locates
the file by that path, so a crate rename must move the guard too.

### Already-Spaceport runtime surfaces (keep)

`<state root>/spaceport` for preferences, logs, saved devices and the Unix
control socket; `<config root>/spaceport/{config,config.local,view}.toml`; the
`\\.\pipe\spaceport-control-` pipe; `spaceport://` as the only accepted deep
link; `Spaceport.app` with `spaceport.development` for development and
`ai.2389.spaceport` (Team `HD9NM9NSMK`) for signed previews.

### Evidence and notes file prefixes

Evidence files are named by task (`t00-`…`t12-`, plus topic files), not by
product name: 236 files, 108 of them `t04-`. They are historical records of
specific revisions and name the crate paths they observed. They are not a
rename target.

### Website and landing copy

Root `site/` and `website/site` are the inherited Sheprd landing pages
(sheprd.org, herdrui.dev) with `sheprd-avatar`/`sheprd-social-preview` assets
and `design/sheprd-website/`. `evidence/t12-site-publication-ownership.md`
records that the root Netlify build is paused, that `website/netlify.toml` can
publish the inherited page, and that those pages still advertise removed Sheprd
features. No Spaceport landing page exists yet, so there is no Spaceport copy to
rename; editing the upstream pages is out of scope here.

### Guard-rail tests that encode the dual name

`bundled/tests/branding.rs`, `bundled/tests/precedence.rs`,
`chat/tests/start_branding.rs`, `chat/tests/spaceport_discovery.rs`,
`sheprd-acp/src/codex/bridge/tests/branding.rs`,
`chat/preview/tests/branding.rs`, `about/tests/attribution.rs`. They currently
assert the precedence order, not that the legacy name still exists anywhere.

## Hard constraints

| Cannot change yet | Why | Owner |
| --- | --- | --- |
| Vendored wire declarations in `sheprd-protocol` (`wire.rs`, `endpoint.rs`, surface encodings, JSON fixtures) | Herdr 1/2 generation-1 compatibility: bincode field and variant order, capability names, message kinds and decoder limits are a wire contract, not branding | T04/T10 keep them; `NOTICE.md` |
| `NOTICE.md` attribution and its pinned Herdr commit | Apache-2.0 attribution and provenance for vendored code | Present in any rename |
| Upstream program names `herdr`, `herdr-core`, `hc`, and the `herdr`/`herdr-dev` config dirs | Separate runtimes under D58; the app attaches to them and must not rebrand them (`upstream_program_names_are_not_rebranded`) | D58 |
| `HERDR_*` environment variables (`HERDR_ACP`, `HERDR_PANE_ID`, `HERDR_SOCKET_PATH`, `HERDR_RELEASE_VERSION`, …) and the `~/.herdr` install root | Set or read by Herdr, the daemons and existing SSH hosts; `docs/cli.md` documents the install root | External contract |
| Herdr-visible ACP identity `herdr-acp` (clientInfo name, activity `SOURCE`) | Herdr records and distinguishes these reports; changing them alters what the daemon shows for existing panes | Runtime integration |
| Installed-preview update identity: `ai.2389.spaceport`, Team `HD9NM9NSMK`, `SUFeedURL`, the fixed `macos-preview-feed` release asset with `appcast.xml`, the Sparkle Ed25519 key, immutable dated tags | D61/D64/D65/D66 and `scripts/release/verify-macos-preview.sh` assert these; renaming or repointing breaks updates for already-installed previews | T12 |
| Closed production identity: `SPACEPORT_APP_ID`, the `HERDR_RELEASE_VERSION` compile assertion, packaging/desktop-entry names | The release route is deliberately refused until identity and update routes are verified; renaming surfaces behind that gate hides nothing and verifies nothing | T12, G-RELEASE |
| Installed skill `name: spaceport` and its marker | `AGENTS.md`: installed copies are found by the name; the app only rewrites files carrying its marker | Agent skill |
| `<root>/spaceport` state and config paths | User settings, saved devices and logs already live there; renaming strands them | T01 |
| Historical evidence text and `sheprd-labs/sheprd` credit links | Evidence records what a revision contained; D59 credits name a third party | D59 |

## Options

### A. Rename only user-visible surfaces, keep internal `sheprd-*`

Finish what T01 started (already done), then retire the dual-name lookups behind
an explicit generation check: one name to ship, one name to accept, with a
visible diagnostic when only the legacy name resolves.

- Pros: smallest risk; no crate churn; lockfile and merge-conflict surface
  untouched; evidence citations stay valid.
- Cons: keeps 1,603 files of inherited naming; `cargo install … sheprd-acp`,
  `log_targets`' `sheprd_` prefix and every crate path remain misspelled for the
  life of the fork; the internal/external split is permanently inconsistent.

### B. Staged crate renames after the deletions and before T12 qualification (recommended)

Take A's user-visible work, then rename surviving crates in coherent units once
the dependency graph stops changing.

- Pros: converges on one name; each unit is small and reviewable; the rename map
  keeps evidence honest; diagnostics, guards and docs move with their crate.
- Cons: needs several branches and one integrating writer for
  `[workspace.members]`, workspace dependencies, `Cargo.lock`,
  `log_targets::LOG_TARGETS` and the crate-path guards; touched in parallel PRs
  it conflicts.

### C. Rename everything now

- Pros: one big change, no interim inconsistency.
- Cons: collides with open feature PRs and the T04/T10 deletions (renaming crates
  those tasks delete is wasted work and conflict bait); silently drops
  diagnostics until `log_targets` is updated; invalidates 236 evidence files'
  paths; forces `Cargo.lock` churn across every branch; risks the packaged
  preview identity while G-RELEASE is open.

**Recommendation: B**, with A's completed pieces kept as the interim state, and
one added mitigation regardless of option: make the legacy fallback loud. When
only `herdr-acp` resolves, chat start should say that the app's own helper is
missing and that the older one was used, instead of quietly succeeding or
failing with a generic message.

## Sequencing

1. **Now (T01 support).** Add the loud-legacy fallback and its test; nothing
   else changes name. This is the only piece worth doing before the deletions,
   because it removes the silent-failure mode this proposal exists for.
2. **T04 (blocked, removing excluded families).** Do not rename crates or
   helpers T04 deletes. When the automation route is finally removed, delete the
   dead `automate` arm of `bundled::preferred_name` and its `precedence.rs`
   case in the same change, and leave `acp` as the only renamed helper.
3. **T10 (finalizing dependencies).** This is the rename window: the member
   list, workspace dependencies and dead code stop moving. Rename in units —
   leaf protocol/terminal/value crates first, then client/backend/command
   layers, then `sheprd-acp`, then `sheprd-gpui` — each unit updating
   `Cargo.toml` members, the `[workspace.dependencies]` entries, import paths,
   `log_targets` (`crate_prefix`), path-based guards and `docs/cli.md`. Record
   the old→new map in one note so evidence files can be read with it; do not
   rewrite their prose.
4. **T12 (blocked, release identity and updates).** No crate rename touches
   `ai.2389.spaceport`, `SUFeedURL`, `macos-preview-feed`, the signing scripts'
   allowlists or the `HERDR_RELEASE_VERSION` gate. If the helper is renamed,
   previews must keep shipping the current name for at least one full update
   cycle, and `package-macos.sh`, `sign-macos.sh` and
   `verify-macos-preview.sh` change together.
5. **Later.** A Spaceport landing page is a separate T12/ownership decision; the
   inherited `site/` and `website/` pages are not rename targets here.

## Verification for any rename unit

- `git grep -n 'sheprd'` before and after, reviewed as a diff rather than a
  count.
- `log_targets::tests` plus one test that a renamed crate's events still reach
  the log.
- `bundled/tests/branding.rs`, `bundled/tests/precedence.rs`,
  `chat/tests/start_branding.rs`, `chat/tests/spaceport_discovery.rs` — the
  precedence order must survive each unit.
- `removed_updater.rs` for the `app_name.rs` path, `about/tests/attribution.rs`
  for credits.
- `cargo metadata --locked` for the member/dependency rename, and the release
  package tool tests (`scripts/release/tests`) for companion names.
- A real signed preview is required before claiming an installed-update
  surface changed; source and packaged-artifact checks are not that proof.

## Open questions for the owner

- Is the internal crate namespace wanted at all, or is the inherited prefix
  acceptable indefinitely in exchange for zero churn?
- Should the legacy `herdr-acp` fallback ever be removed, and if so after which
  supported version stops shipping it?
- Does a future Spaceport site inherit `site/` in this repository, or move to its
  own repository?
