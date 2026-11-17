# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Static HTML, CSS and JavaScript with no build step, matching the inherited
sites in this repository. Inferred from repository convention, not asked.
GitHub Pages serves it from a GitHub Actions workflow at
`https://2389-research.github.io/spaceport/` (confirmed by Doctor Biz,
2026-11-17). The page uses relative paths so a custom domain can follow.

## Users

Developers who run coding agents and shells in terminals and want one native
desktop window for them. They arrive to decide whether to download the macOS
preview (confirmed 2026-11-17).

## Product Purpose

Spaceport is a native desktop workspace for terminals and AI agent chats. It
is the client; separately installed Herdr 1 and Herdr Core runtimes own the
terminal processes and sessions. Success for the landing page is a download of
the signed macOS preview.

## Positioning

Spaceport's claims are about Spaceport: the native window, its rich chats,
workspaces, customization, notifications, diagnostics and its `spaceport`
command line. Herdr's capabilities are Herdr's. Do not present them as
Spaceport's own (confirmed 2026-11-17: "let's not make herdr claims our
claims"). Credit Herdr as the runtime Spaceport connects to.

The fork's motive (Doctor Biz, 2026-11-17): simplify the app and focus it on
running agents without supervision. State this as the aim, not as a finished
capability.

## Operating Context

- macOS 14.2 or later on Apple silicon for the preview.
- Herdr is installed separately. Spaceport does not bundle, install, update or
  migrate it.
- Herdr Core support is experimental and off by default.
- Installed previews update through Sparkle's Check for Updates.
- Agents in panes can drive the app with `spaceport COMMAND` (`docs/cli.md`).

## Capabilities and Constraints

- Kept: native terminals, rich chats, workspaces, customization, file
  interactions, service access, notifications, diagnostics, local desktop CLI.
- Removed from the fork: IDE, Git, GitHub, embedded browser, cloud
  provisioning, Teleport, orchestration, mods, scheduler, phone control.
- Under development. The preview is a prerelease; general production releases
  and Linux and Windows distribution have open gates. Do not claim them.
- Spaceport is not affiliated with or endorsed by Sheprd or Herdr.

## Brand Commitments

- Name: Spaceport, by 2389 Research, Inc.
- Icon: the sculpted nebula rocket, `design/spaceport-icon/rocket-nebula.png`
  (decisions D62 and D63). Use its exact bytes or derivatives of them.
- Credits: an independent fork of Sheprd (Fabien Penso); Apache-2.0.

## Evidence on Hand

- Signed, notarized macOS arm64 preview DMGs on GitHub Releases.
- The approved icon and its platform derivatives under `assets/icons`.
- `docs/cli.md` and `skills/spaceport/SKILL.md` for real command names.
- No current Spaceport screenshots. `docs/screenshots` is inherited from
  Sheprd and shows removed features; do not use it. The page draws an
  illustrated window and labels it as an illustration (confirmed 2026-11-17).
- No testimonials, customer names, benchmarks or pricing exist. Do not invent
  them.

## Product Principles

1. Say what Spaceport does, and name what belongs to Herdr.
2. State preview status plainly; never imply a finished release.
3. Show the product working rather than describing it.
4. One clear action: download the preview.
