# User decision ledger

**Generated from `plan/decisions.json`; edit the JSON, not this view.**

| ID | Decision | Status |
|---|---|---|
| D01 | **Workspaces first** — Keep projects/workspaces as the primary navigation. | active |
| D02 | **No IDE or diff viewer** — Delete IDE integrations and specialized diff/review UI. | active |
| D03 | **Full terminals** — Keep general-purpose shells, tabs, splits and interactive agent terminals. | active |
| D04 | **No Git integration** — Remove Git/worktree/branch/checkpoint management and Git-specific navigation. | active |
| D05 | **Browser versus forwarding** — Remove embedded browser and annotations; keep listening-port discovery and forwarding; open URLs externally. | active |
| D06 | **No cloud provisioning** — Remove provider integrations and provisioning; consume existing Herdr machines. | active |
| D07 | **Automation visibility only** — Keep supported read-only automation status/results; remove editing, scheduling engines and mods. | active |
| D08 | **Equal chat and terminal** — Both remain first-class views; do not imply every terminal has a native chat capability. | active |
| D09 | **Unified and host views** — Unified workspace navigation with optional grouping by host; retain upstream presentation where possible. | active |
| D10 | **Full notifications** — Retain configurable agent attention, completion, error and supported job-result notifications. | active |
| D11 | **Herdr integrations** — Surface supported Herdr integration state/actions; do not own an integration subsystem. | active |
| D12 | **Agent-focused monitoring** — Keep reported agent/usage/rate-limit information; remove detailed host resource dashboards. | active |
| D13 | **No Teleport** — Remove client migration and execution relocation; reconnect to existing sessions. | active |
| D14 | **Workspace controls** — Keep create/rename/open/archive/delete controls; operate on Herdr-backed presentation metadata, not a new runtime. See reconciliation. | clarified |
| D15 | **External Tailscale** — Manual addresses/SSH aliases; Tailscale is independently installed and managed; no discovery or embedded login. | active |
| D16 | **Full customization** — Keep existing themes, layouts, fonts, shortcuts and appearance controls. | active |
| D17 | **Activity history** — Keep an activity timeline; later decision D49 rules out adding a new timeline architecture or history store. | qualified |
| D18 | **Online only** — No persistent offline content cache or offline action queue. View preferences may persist. | active |
| D19 | **Timeline-scoped search** — Search only information actually available in the retained activity timeline; no new file or global terminal index. | active |
| D20 | **Desktop first and full windows** — macOS/Windows/Linux first; retain windows, tabs, splits and pane management. Mobile is a separate exercise. | active |
| D21 | **Full command palette** — Keep the palette and relevant actions; remove commands for deleted features. | active |
| D22 | **File interactions** — Keep drag/drop, clipboard images, attachments, file transfers and external opening; no IDE navigation. | active |
| D23 | **Session creation** — Expose supported session/template launch controls through Herdr. Do not create a second agent registry or runtime. | active |
| D24 | **Local startup** — Connect to or start the separately installed local Herdr Core when needed. | active |
| D25 | **No bundled Herdr** — Do not bundle, download, install or update Herdr on the user's behalf. | active |
| D26 | **Spaceport auto-updates** — Retain app updates using Spaceport-owned identities and release infrastructure; no Herdr updates. | active |
| D27 | **No provider/model configuration** — Remove client-owned model/provider credentials and configuration. Agent programs keep their own settings. | active |
| D28 | **Rich chat** — Keep rich text, code, images, tool events and interactive prompts through a verified existing chat contract. | active |
| D29 | **Attention indicators** — Keep existing sidebar indicators, unread badges, OS badges and notification sounds. | active |
| D30 | **No terminal annotation bridge** — Remove terminal-selection annotations and send-to-agent tools; keep ordinary copy/paste. | active |
| D31 | **No GitHub integration** — Remove client-side GitHub sign-in, account integration, repository/PR/issue/security UI. | active |
| D32 | **Mirror supported input** — Expose only capabilities the backing Herdr/host-side agent contract actually supports. | active |
| D33 | **Quit on last window** — No background tray service; quitting Spaceport must not terminate Herdr or its agents. | active |
| D34 | **No plugins** — Remove all client-side plugin/mod extension systems. | active |
| D35 | **Client CLI** — Keep the relevant full client CLI; agent-runtime operations belong to hc, not duplicate Spaceport implementations. | active |
| D36 | **Restore views** — Restore windows/layout/selection and reattach; do not restart missing agents to recreate a view. | active |
| D37 | **Guided onboarding** — Detect/start installed local Core and offer manual remote setup through Core; never bootstrap missing Herdr. | active |
| D38 | **Existing status display** — Preserve upstream sidebar/status presentation; only report status supported by authoritative data. | active |
| D39 | **Defer terminal swap** — Keep current Alacritty-based emulation/GPUI rendering; no Ghostty migration in this fork. | active |
| D40 | **Host manager UI** — Keep the host-manager experience; D48/D56 place machine state and remote connection logic in Herdr. | qualified |
| D41 | **Existing conversation organization** — Preserve current conversation navigation and session organization. | active |
| D42 | **Existing sidebar** — Preserve hierarchy, interactions and layout options minus deleted feature entries. | active |
| D43 | **Existing terminal UX** — Preserve selection, scrollback, keyboard, mouse, themes and shortcuts. | active |
| D44 | **Existing transfers** — Retain file transfer behavior where supported by the Herdr-backed route; parity must be proved. | active |
| D45 | **Existing settings UI** — Keep design and organization, deleting settings for removed features. | active |
| D46 | **Existing menus** — Keep native/context menu structure and shortcuts, deleting removed actions. | active |
| D47 | **Diagnostics** — Keep troubleshooting logs and diagnostic tooling, without deleted-feature or secret leakage. | active |
| D48 | **Herdr owns connections** — Core owns remote machine catalog, transport, reconnect and aggregated state; Spaceport consumes it. | active |
| D49 | **No timeline redesign** — No change to activity timeline behavior beyond necessary backend migration and deleted-feature cleanup. | active |
| D50 | **Keep profile shell** — Keep profile/avatar UI where meaningful; remove GitHub actions and do not add a client identity system. | active |
| D51 | **Complete rebrand** — Spaceport product identity, executable, app IDs, release identity and assets; preserve required attribution and wire compatibility. | active |
| D52 | **Name** — Spaceport. | active |
| D53 | **Independent fork** — Maintain independently rather than routinely merging upstream; keep provenance and security diligence. | active |
| D54 | **Real deletion** — Delete unwanted code/dependencies/UI/tests/settings/docs; do not merely hide it behind flags. | active |
| D55 | **Herdr 2 only** — Remove legacy Herdr 1 compatibility; this does NOT mean removing Core protocol generation 1. | superseded |
| D56 | **Herdr-mediated connectivity** — Use Core for remote runtime connectivity; no new Spaceport transport/federation service. | active |
| D57 | **Remove mobile server** — Delete phone remote-control server and its desktop integration; defer all mobile design/implementation. | active |
| D58 | **Allow Herdr 1** — Herdr 1 is an allowed runtime alongside Herdr Core. Preserve working native UX instead of requiring a Core-only startup or deleting the Herdr 1 backend. D55 is superseded. D24, D37, D48 and D56 apply to Core routes; existing Herdr 1 startup and connectors may serve Herdr 1 routes. Retain exact runtime targets and session ownership. This does not restore excluded features, authorize runtime installation/update or imply untested parity. | active |
| D59 | **Credit both upstream teams** — The Spaceport About view must explicitly reference the Herdr team and the Sheprd team. Label upstream links as credits, retain required copyright/license notices and avoid implying endorsement. | active |
| D60 | **Remove the inherited custom updater** — Delete the inherited custom Rust updater and its update/install/restart machinery. Spaceport updates remain a required release capability; this is a replacement transition, not removal of SP-023. Support macOS, Windows, Linux and package-manager installation routes. Sparkle and other platform mechanisms were discussed but no replacement library, Homebrew update-ownership policy, signing identity, release endpoint or publication was approved by this deletion instruction. Preserve release-build refusal while independent identity and update routes remain unverified. | active |
| D61 | **macOS signed preview workflow** — Implement a macOS-only build-and-sign/notarize artifact workflow using 2389 Research signing credentials and bundle ID ai.2389.spaceport. Windows signing is deferred for this work, not removed from the eventual desktop scope. D64 extends this route to tag-triggered GitHub prereleases. App installation, credential upload, automatic updater and general production release enablement remain separate from this decision. Retain the release refusal and distinguish ordinary development builds from signed previews. | active |
| D62 | **Rocket emoji icon** — Use a rocket as Spaceport's app icon. D63 selects the later sculpted nebula artwork after the initial emoji exploration. Keep Spaceport as the product name; derive platform artwork from one shared source. | qualified |
| D63 | **Ship the nebula rocket artwork** — Use the approved sculpted ivory/red rocket with a violet/cyan nebula background as Spaceport's app artwork. The selected master is design/spaceport-icon/rocket-nebula.png. Generate platform and branding exports from this image. | active |
| D64 | **Tag-triggered macOS preview releases** — Extend the macOS preview workflow to build, sign, notarize and publish a DMG plus checksum to a GitHub prerelease when harperreed pushes vYYYYMMDD.COUNTER to harperreed/spaceport. Require the exact tagged commit to belong to main and pass full main CI. Keep signing credentials in a protected signing job; grant release write access only to publication. Preserve manual artifact-only runs, immutable version reservations and the broader production release refusal. Doctor Biz's follow-up instruction to push and make it work authorizes main integration, hosted environment setup, credential upload and the first verified tagged preview; keep private signing inputs outside Git. | active |
| D65 | **Sparkle updates for macOS previews** — Use pinned Sparkle 2.10.1 for installed macOS previews, with native Check for Updates UI, consent for automatic checks and automatic installation off by default. Keep development/worktree builds excluded. Embed and sign the framework inside the existing ai.2389.spaceport bundle for Team HD9NM9NSMK. Use a separate Spaceport Ed25519 key outside Git/Cargo and in the protected macos-preview environment; require signed feeds without timed recovery (SUSignedFeedFailureExpirationInterval=0) and pre-extraction archive verification. Publish the signed preview feed as the fixed macos-preview-feed GitHub release asset only after the immutable dated release passes verification. Enforce exact Spaceport app/Team identity in packaging, signing and publication. Sparkle permits trusted publisher key and app identity rotation; do not claim its runtime enforces an unconditional old app ID match. Preserve normal GUI quit/detach and never change Herdr/Core lifecycle. Qualify with isolated real signed update/relaunch fixtures and negative trust/rollback cases. Carry implementation through full source CI and a fresh signed release. Other platform/package routes and overall T12 gates remain open. | active |
