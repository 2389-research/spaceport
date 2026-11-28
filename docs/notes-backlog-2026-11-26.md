# Spaceport Sequenced Backlog — 2026-11-26

The original sequence came from the crew board and the extraction plan.
Status was checked against main at `f1b4665e7`. PR references below track
the work and its evidence. The acceptance criteria live in the fork spec.

## Track A — Finish the extraction programs (the split plan)

| # | Ticket | Size | Depends on | Status |
|---|--------|------|------------|--------|
| A1 | **i18n stage 4** — move each cross-crate error mapping to its owner. | S | None | Done in #254. |
| A2 | **Menu S6c** — workspace popover. | S | None | Done in #303. The app prepares actions and values. |
| A3 | **Settings stage 5** — appearance service and theme browser. | M | A4 supplies the search field | #304 landed the service. #315 tracks the theme browser. |
| A4 | **Text-field stage 2** — dialog and multiline inputs. | M | Stage 1 (#252) | Done in #305. |
| A5 | **Menu S6d** — Keybinds page. | M | Search field | Done in #308. The app supplies the shortcut editor. |
| A6 | **Menu S6e part 1** — workspace, tab and pane dialog bodies. | M | Draft field | Done in #309. The app owns draft fields and handles the buttons. |
| A7 | **Menu S6e part 2** — Add Device and WSL pages. | L | Prepared host data | #332 tracks the page extraction. |
| A8 | **Menu S6e part 3** — device picker, sessions, host forms, theme picker and shortcut editor. | L | A7’s prepared host data and actions | Follow-up after the device pages. |
| A9 | **Settings stage 6** — settings pages and controls. | XL | A3 and A4 | Follow-up after the theme browser. #250, #253 and #254 settled where controls can get their words. |
| A10 | **Chat stage 4** (wx65): attach over `spaceport-live` values — the last chat stage. | M | none | **closed** — #234 landed the slice (owner decision); what is left of the stage is app-side paint, not a ticket |
| A11 | **Endpoint residual**: drain split deferred at wx75 — revisit if the mailbox routing changes. **Revisited: the routing did not change, and the desktop drain is already split — see [`notes-a11-endpoint-drain.md`](notes-a11-endpoint-drain.md); closed as still deferred.** | S | none | closed |

## Track B — Release & qualification (the "Complete ..." backlog rows)

| # | Ticket | Size | Depends on | Status |
|---|--------|------|------------|--------|
| B1 | **wx44** — measure each native sidebar frame at its painted width. | S | Active desktop for the native probe | #306 fixed the resize probe. Native activation still requires an attended desktop. |
| B2 | **wx52** — align the crate table and split plan with landed work. | S | A1, A2 | #310 records this census. |
| B3 | **wx53**: release-mode performance budget re-verify after the extraction waves (menu + i18n + textfield + about). | S | next release cycle | soon |
| B4 | **wx16**: post-rename QA pass on user-visible strings (was gated on the rename; the rename has shipped — un-icebox it). | M | none | ready |
| B5 | **wx17**: Core 0.1.20 schema re-audit when it ships. | S | external (Core release) | gated |
| B6 | **wx18**: machine-gated run-sheet session scheduling (M/W/L/P2). | M | wx17 | gated |
| B7 | Native macOS application acceptance (6q5m): the T12 evidence route exists; finish the qualification matrix. | L | r69x signing | soon |
| B8 | Native Windows application acceptance (h8g5). | L | none | backlog |
| B9 | Native Linux application acceptance (r844). | L | none | backlog |
| B10 | **05m4** — binary upload and download qualification. | M | A supported download route | #314 records [upload safety coverage and the missing download route](spaceport/evidence/b10-binary-transfer-matrix.md). |
| B11 | **yt4a** — authoritative usage and automation-status feeds. | M | An authoritative feed | #298 removed the views that lacked a feed. |
| B12 | Accessibility, resource and adversarial qualification complete (a5ks). | XL | staged internally | backlog |

## Track C — Release pipeline & identity

| # | Ticket | Size | Depends on | Status |
|---|--------|------|------------|--------|
| C1 | **r69x**: finish signing + native verification of the nebula Mac preview. | M | owner credentials | soon |
| C2 | **zafp** — replacement updates after D60. | L | Release authorization | The macOS preview uses Sparkle. #307 tracks stable-channel support. |
| C3 | **megk** — Spaceport release channels and download metadata. | M | C2 | #311 tracks stable feed and artifact names. |
| C4 | **d4z8** — Windows and Linux package identities and signing. | L | Signing credentials for signed artifacts | #312 landed package identities. The [package evidence](spaceport/evidence/d4z8-package-identity.md) records signing requirements. |
| C5 | **007r** — verify installation and upstream-state isolation. | M | Native install environments | #313 tracks the procedure. A procedure alone does not qualify an install. |

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

## Dependencies for remaining work

- Follow #315 for the settings theme browser. Stage 6 uses its theme browser boundary.
- Follow #332 for device pages. A8 uses the same boundary for host state and actions.
- Qualify release and install behavior on each target named in the spec. Record the observed results in its evidence note.

When a slice lands, update the board row, the split-plan stage table and the crate table in `AGENTS.md`.
