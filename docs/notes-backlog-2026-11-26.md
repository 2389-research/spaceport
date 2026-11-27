# Spaceport Sequenced Backlog — 2026-11-26

Pulled from the crew board's icebox/backlog/ready rows and the split plan's open
stages; stale rows verified against landed work and re-filed. Sequenced small to
large inside each track. Status column: `next` = launch now, `soon` = behind one
gate, `gated` = waiting on an owner decision or external event.

## Track A — Finish the extraction programs (the split plan)

| # | Ticket | Size | Depends on | Status |
|---|--------|------|------------|--------|
| A1 | **i18n stage 4**: move the 21 foreign-type `Localize` impls out of `crates/spaceport-gpui/src/i18n/errors/` to their owning crates (`spaceport_web_url`, `spaceport_settings`, `spaceport_widgets`, tunnel, `SshFailure`, …). The 36 app-`Error` impls stay. | S (~250 lines) | none — fully mapped | **done** — landed in #254, which moved every cross-crate mapping into its crate's `src/localize.rs` |
| A2 | **Menu S6c**: workspace popover (~270 lines: header, tiles, mute toggle) → `spaceport-menu`, wording app-side, layout in-crate. | S | none | **next** |
| A3 | **Settings stage 5**: theme browser (`themes.rs`, appearance/grid/system pages) — blocked on the appearance service extraction first (P5) and `SearchInput` (have it). Split: 5a appearance service → `spaceport-settings`, 5b the browser. | M | none for 5a | **next** |
| A4 | **Text-field stage 2**: `DialogInput` + `MultilineInput` into `crates/spaceport-textfield` (facade keeps re-exports; app keeps messages). ~600 lines. | M | none — stage 1 proved out in production | **next wave 2** |
| A5 | **Menu S6d**: Keybinds page (~700 lines) — needs `spaceport-textfield` (A4) for the search field; shortcut editor stays app-side as an injected element. | M | A4 | soon |
| A6 | **Menu S6e part 1**: rename forms — tab rename, pane rename, workspace rename/new/note dialogs move once `DialogInput` is crate-side; convert the closing-page read (`appear/kept.rs`) to a prepared `String`. | M | A4 | soon |
| A7 | **Menu S6e part 2**: Add Device + WSL pages (fields are `Entity<SearchInput>` triples) — need endpoint-state `Hosts` prepared-values treatment, then move. | L | A4, endpoint Hosts pass | gated on A4 + a scout pass |
| A8 | **Menu S6e part 3**: device picker, sessions management, host-menu forms (rename/tags/theme/forwards), theme picker, shortcut editor. The last and largest of the menus program. | L | A4 + A7's pattern | gated |
| A9 | **Settings stage 6**: the eleven page modules + `controls/*` (3,692 lines) — the final settings push, P1 ruling now answered so wording can cross. | XL | A4 (DialogInput), staged like the menus program | gated |
| A10 | **Chat stage 4** (wx65): attach over `spaceport-live` values — the last chat stage. | M | none | **closed** — #234 landed the slice (owner decision); what is left of the stage is app-side paint, not a ticket |
| A11 | **Endpoint residual**: drain split deferred at wx75 — revisit if the mailbox routing changes. | S | none | backlog |

## Track B — Release & qualification (the "Complete ..." backlog rows)

| # | Ticket | Size | Depends on | Status |
|---|--------|------|------------|--------|
| B1 | **wx44**: native_sidebar environment gap + mid-resize paint flake. Small, concrete, already scoped. | S | none | **next** |
| B2 | **wx52**: docs sync — crate tables in AGENTS.md + split plan vs reality (partially done today; finish after A1/A2 land). | S | A1, A2 | soon |
| B3 | **wx53**: release-mode performance budget re-verify after the extraction waves (menu + i18n + textfield + about). | S | next release cycle | soon |
| B4 | **wx16**: post-rename QA pass on user-visible strings (was gated on the rename; the rename has shipped — un-icebox it). | M | none | ready |
| B5 | **wx17**: Core 0.1.20 schema re-audit when it ships. | S | external (Core release) | gated |
| B6 | **wx18**: machine-gated run-sheet session scheduling (M/W/L/P2). | M | wx17 | gated |
| B7 | Native macOS application acceptance (6q5m): the T12 evidence route exists; finish the qualification matrix. | L | r69x signing | soon |
| B8 | Native Windows application acceptance (h8g5). | L | none | backlog |
| B9 | Native Linux application acceptance (r844). | L | none | backlog |
| B10 | Binary upload/download behavior complete + verified (05m4). | M | none | backlog |
| B11 | Usage and automation-status feeds authoritative (yt4a). | M | none | backlog |
| B12 | Accessibility, resource and adversarial qualification complete (a5ks). | XL | staged internally | backlog |

## Track C — Release pipeline & identity

| # | Ticket | Size | Depends on | Status |
|---|--------|------|------------|--------|
| C1 | **r69x**: finish signing + native verification of the nebula Mac preview. | M | owner credentials | soon |
| C2 | **zafp**: choose + implement replacement Spaceport updates (post-D60; Sparkle route exists for previews — production channel is the open piece). | L | owner decision | gated |
| C3 | **megk**: Spaceport-owned release channels + download metadata (landing page, appcast naming, stable channel). | M | C2 | gated |
| C4 | **d4z8**: Windows + Linux package identities and signing. | L | C2 | gated |
| C5 | **007r**: verify installation + upstream-state isolation. | M | none | backlog |

## Track D — Big rocks (need owner sequencing)

| # | Ticket | Size | Depends on | Status |
|---|--------|------|------------|--------|
| D1 | **ync5**: manager agent — needs a spec + owner scoping session before any ticket splits. | XL | owner | gated |
| D2 | **rnbn**: graduate Herdr Core from experiment to default. | XL | Core stability, wx17 | gated |
| D3 | **6nht**: implement the Spaceport fork against the spec task graph — the meta-item; most of it is now done via the extraction programs; re-scope what remains. | XL | a re-audit pass | soon |
| D4 | **t6vb**: figure out sheprd-acp's fate (the fork's ACP client story). | L | owner decision | gated |
| D5 | **eg9b**: Core machine actions target the selected permanent identity. | M | D2 | gated |
| D6 | **dgw9 / f48r**: Core-mediated loopback forwarding; a supported Core route for remote native chat. | L | D2 | gated |

## True icebox (external-gated, do not plan)

- **wx45** publish generic crates to crates.io (owner decision)
- **wx46** obsolete by progress — close it
- **wx5d** chat stage 4 residual (superseded by A10)
- **wx17/wx18** carry their gates inline (also in B5/B6)

## Suggested launch order (next 3 waves)

1. **Wave 1 (now)**: A1 (i18n stage 4) + A2 (menu S6c) + B1 (wx44) in parallel — three small tickets, disjoint files.
2. **Wave 2**: A4 (text-field stage 2) + A3a (settings appearance service) + A10 (chat stage 4).
3. **Wave 3**: A5 (Keybinds page) + A6 (rename forms) + B2 (docs sync) — everything A4 unblocks starts landing.

Close on merge of each: update the board row, the split-plan stage table, AGENTS.md's crate table.
