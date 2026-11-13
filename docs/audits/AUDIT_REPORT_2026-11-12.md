# Spaceport branding audit

Audit date: November 12, 2026, Chicago time; evidence collected November 13 UTC.
Source: `bd1782f9c6507abe5436983aa31a9be91f2b316f`, after PR #19.
Branch: `audit/spaceport-branding`.

This is a historical snapshot. Source links pin the inspected revision.
See [the follow-up record](../spaceport/notes/branding-audit-followups.md) for fixes and verification.

The remaining work includes functional rename regressions, not just old display text.
The GUI's log filter rejects its renamed namespace.
Remote helper discovery omits the current Spaceport bundle name.
The repository also directs private security reports to upstream maintainers.

This audit changes no application behavior, repository settings, releases or Kata records.
It records nine finding groups and one existing release-gate group.
Two finding groups have P1 priority, four have P2 priority, and three have P3 priority.
P1 means fix promptly; P2 means a concrete correctness or customer-facing issue; P3 means cosmetic copy.

## Findings

### BR-01 · P1 · The GUI rename silently drops application diagnostics

The GUI binary is named `spaceport` in [Cargo.toml](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/Cargo.toml#L9).
Default tracing targets therefore start with `spaceport`, including its `app` and config modules.
However, [LOG_TARGETS](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/log_targets.rs#L4) still admits `sheprd_gpui` and omits `spaceport`.
[App startup](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/app.rs#L386) passes that exact list to the diagnostics collector.
The collector's [kept_target](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-diagnostics/src/lib.rs#L435) accepts only listed roots and their modules.

An isolated executable reproduced the mismatch using the actual target list and compiled diagnostics library.
It used Rust crate name `spaceport`, real tracing events, private state and the collector's flush API.
The result was:

```text
crate_name=spaceport app_default_captured=false legacy_control_captured=true client_control_captured=true dropped=0
```

The two control events prove that the collector and writer worked.
The GUI event disappeared before queue accounting, so `dropped` stayed zero.
This affects default GUI events, not events from retained dependency crates such as `sheprd_client`.

The [Logs search placeholder](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/locales/en-US/log-window.ftl#L4) also suggests the old GUI namespace.
The existing [coverage test](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/log_targets/tests.rs#L16) asserts the obsolete name explicitly.

Follow-up: fix the GUI root and search example together.
Cover the real producer namespace and collector, rather than another hard-coded old-name assertion.

### BR-02 · P1 · Private security-report guidance names the wrong recipient

The active [issue chooser](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/.github/ISSUE_TEMPLATE/config.yml#L9) labels an upstream advisory link “Security vulnerability.”
It asks reporters to report privately, without explaining that another project receives the report.
[SECURITY.md](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/SECURITY.md#L3) explicitly says that upstream form is not a Spaceport reporting channel.

A fresh GitHub API read also returned `{"enabled":false}` for Spaceport's private vulnerability reporting.
Simply replacing the repository in the link would not establish a verified working channel.
No report was submitted and no reporting setting was changed.

Follow-up: choose and verify the Spaceport-owned private reporting route.
Then align the issue chooser and security policy with its actual recipient and availability.
Do not invent a contact or silently enable reporting as a text-only branding change.

### BR-03 · P2 · Remote helper discovery misses Spaceport.app

[REMOTE_PROGRAM_DIRS](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/chat.rs#L190) includes `Herdr.app` and `Sheprd.app`.
It omits both `~/Applications/Spaceport.app` and `/Applications/Spaceport.app`.
The [current packager](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/scripts/release/package-macos.sh#L43) ships the helper inside `Spaceport.app`.

This is a different path from local bundled-helper lookup, which already prefers the shipped helper name.
The remote producer is used for [attachment](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/chat/chats.rs#L66),
[startup](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/chat/start.rs#L131), and [agent discovery](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/chat/harnesses.rs#L122).

A probe extracted the actual constant and script producer, then executed its output against private bundle fixtures.
It rebased absolute `/Applications` candidates into the fixture and used an empty private PATH.
No personal installation or SSH host was queried.

```text
Sheprd.app:   exit 0, helper found
Spaceport.app: exit 127, “sheprd-acp is not installed here”
```

Follow-up: include the current bundle locations and add a real filesystem/script regression.
Preserve existing helper names and previously supported locations.
This finding concerns the retained SSH path, not proof of Core remote-chat parity.

### BR-04 · P2 · Generated configuration still instructs users to operate Sheprd

This is active generated content, not an archived example.
[Config](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/config.rs#L99) embeds `config-gpui.example.toml` as `DEFAULT_CONFIG`.
[prepare_files](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/config/files.rs#L73) writes those bytes to the managed configuration.
Its [opening comments](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/config-gpui.example.toml#L1) name Sheprd and the upstream `herdr/config-gpui*` paths.
Its [chat-width example](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/config-gpui.example.toml#L444) runs `sheprd config set`, not `spaceport`.

The separate [new-overrides seed](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-client/src/settings_files.rs#L23) has a Spaceport heading.
Its remaining instructions still name `config-gpui.toml`.
The actual [managed path](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/config.rs#L381) is `spaceport/config.toml`.
The local overrides path is `spaceport/config.local.toml`.

Following the old command fails on a Spaceport-only machine.
If Sheprd is also installed, it can target the other application's state.

Follow-up: correct the two real producers, retaining their existing public interfaces.
Verify newly generated content and preserve existing personal overrides byte-for-byte.
The internal template filename itself does not need renaming to fix its contents.

### BR-05 · P2 · The exposed Nix package installs an upstream-branded launcher

[flake.nix](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/flake.nix#L99) installs `org.sheprd.Sheprd.desktop` unchanged.
The [desktop entry](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/scripts/release/org.sheprd.Sheprd.desktop#L3) says `Name=Sheprd` and runs `spaceport`.
Its `StartupWMClass=org.sheprd.Sheprd` differs from the executable's development identity, `spaceport.development`.
The flake also advertises `https://sheprd.org` as its [homepage](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/flake.nix#L112).

The [default and spaceport outputs](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/flake.nix#L120) expose this package independently of the disabled production workflow.
This is therefore a reachable package route, not merely a dormant release template.
The source mismatch is confirmed; Nix installation and native desktop grouping were not run.

Follow-up: align the package, desktop metadata, icon installation and actual app identity as one change.
Qualify the resulting package on Linux before claiming launcher or window-grouping correctness.

### BR-06 · P2 · Current entry-point docs still teach upstream identities and excluded features

These are active guides, rather than historical records:

| Surface | Verified mismatch | Current source or contract |
| --- | --- | --- |
| [CONTRIBUTING.md:3](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/CONTRIBUTING.md#L3), [40](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/CONTRIBUTING.md#L40) | Calls the project Sheprd and says run recipes launch `Sheprd.app` with executable `sheprd`. | [justfile:18](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/justfile#L18) and [144](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/justfile#L144) use `Spaceport.app` and `spaceport`. |
| [CONTRIBUTING.md:11](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/CONTRIBUTING.md#L11), [16](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/CONTRIBUTING.md#L16) | Promises current Windows coverage and bundled Core. | D66 pauses Windows checks; [justfile:156](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/justfile#L156) removes stale bundled `herdr-core`. |
| [rich-chat.md:36](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/rich-chat.md#L36), [49](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/rich-chat.md#L49), [60](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/rich-chat.md#L60) | Copyable `sheprd config` and `sheprd chat` commands remain. | The real GUI binary and its [help](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/control/client/help.rs#L35) use `spaceport`. |
| [i18n.md:3](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/i18n.md#L3), [32](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/i18n.md#L32) | Names Sheprd, the old CLI and old override filename. | [config.rs:381](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/config.rs#L381) selects the Spaceport directory and filenames. |
| [helper README:65](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-acp/README.md#L65) | Says GUI settings normally use `config-gpui.toml`. | Spaceport uses `spaceport/config.toml`. |
| [cli.md:223](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/cli.md#L223) | Documents the Windows pipe as `herdr-gpui-control-SID`. | [socket.rs:46](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-control-protocol/src/socket.rs#L46) constructs `spaceport-control-{sid}`. |
| [rich-chat.md:9](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/rich-chat.md#L9) | Deep-links into archived upstream configuration instructions. | The [destination warning](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/README.md#L3) says it is not current Spaceport setup guidance. |
| [rich-chat.md:182](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/rich-chat.md#L182) | Claims embedded platform webviews and links the historical browser plan. | [chat/page.rs:85](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/chat/page.rs#L85) opens the system handler; Spaceport excludes the embedded browser. |

The CONTRIBUTING Linux setup link also targets a nonexistent `README.md#linux-builds` section.
Its compatibility mandate treats the historical rename inventory as current Spaceport policy.
The [historical guide](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/rebranding.md#L1) expressly disclaims that interpretation.

Follow-up: update CONTRIBUTING, current command/path examples, and current configuration/browser guidance in bounded slices.
Keep actual `sheprd-acp` commands, Rust package names and Herdr daemon instructions intact.
Keep historical documentation labeled; stop directing current setup flows into it.

### BR-07 · P3 · Public repository copy still emphasizes inherited Git worktree positioning

A fresh API read of [2389-research/spaceport](https://github.com/2389-research/spaceport) returned:

> View terminal sessions, workspaces, Git worktrees, and agent activity through your local Herdr daemon.

The current [README](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/README.md#L17) and SP-015 exclude Git/worktree product controls from the target scope.
Generic workspace and folder viewing remain supported.
This is editorial positioning cleanup, not proof that every interpretation of “view Git worktrees” is broken.
The repository name and homepage point to Spaceport correctly.

Follow-up: update the public description to the retained Spaceport scope.
No repository metadata was changed by this audit.

### BR-08 · P3 · The live Fonts specimen still displays “sheprd / workspace”

[settings-fonts.ftl:18](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/locales/en-US/settings-fonts.ftl#L18) contains that literal.
[sample_messages](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/settings_window/controls/fonts.rs#L126) selects it for the sidebar face.
[render_font_specimen](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/settings_window/controls/fonts.rs#L309) localizes and renders those messages.
This is current sample text, rather than a required protocol name or legal credit.

Follow-up: use Spaceport in the specimen and verify the existing narrow-layout and pseudo-locale checks.
The source path is confirmed; no fresh native Fonts screenshot was captured during this audit.

### BR-09 · P3 · The current artwork preface names the superseded emoji choice

The present-tense Spaceport preface in [rebranding.md:6](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/rebranding.md#L6) still identifies D62's rocket emoji.
[D63](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/docs/spaceport/plan/decisions.json#L439) selected the sculpted nebula artwork.
The [icon guide](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/assets/icons/README.md#L5) identifies the correct approved master.

Follow-up: correct that current preface sentence.
The historical Sheprd inventory below it remains valid historical material.

## Known release-gate group · Do not count as new macOS preview regressions

- [Windows resources](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/windows_resource.rs#L17) still embed Sheprd in ProductName and FileDescription.
  Both the GUI and launcher use this embedder.
  This is already tracked by Kata `d4z8`; no new Windows artifact was inspected here.
- The [Windows installer](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/scripts/release/sheprd.iss#L31) retains upstream installation ownership, AppId, publisher links and installation directory.
  Do not re-enable it without independent identity and installation-isolation qualification.
- The [Linux distro producer](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/scripts/release/package-linux-distro.py#L20), archive scripts and artifact manifest retain upstream package names or ownership.
- The [Homebrew cask](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/homebrew/Casks/sheprd.rb#L5) and [winget producer](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/scripts/release/winget.py#L37) still describe upstream distribution.
  They must not be recommended or published unchanged as Spaceport installers.

The inherited [release workflow](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/.github/workflows/release.yml#L41) is repository/actor gated to upstream.
The GUI also [refuses](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-gpui/src/constants.rs#L14) the inherited production release environment variable.
These guards do not make manually invoked packaging templates correct.
They distinguish those open T12 gates from the active, separately implemented macOS preview route.
Windows CI remains paused under D66; this audit does not pass Windows acceptance.

## Positive checks and intentional retained names

- The shared [product-name constant](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/crates/sheprd-client/src/app_name.rs#L5) is Spaceport.
- App menu labels, macOS bundle names, executable and permission explanations name Spaceport.
- Preview bundle ID is `ai.2389.spaceport`; development uses `spaceport.development`.
- Active preview publishing and the signed-feed URL use `2389-research/spaceport`.
- Public release metadata names `Spaceport 20261112.1 (macOS preview)` and its Spaceport DMG.
- The fetched appcast says `Spaceport macOS previews` and points to that DMG under the current organization.
- Approved rocket tests and Linux icon reproducibility checks pass.
- About, NOTICE and required upstream credits intentionally retain Sheprd and Herdr.
- `sheprd-acp`, retained dependency crate names, protocol identifiers and real Herdr runtime names remain intentional.
- Existing `Sheprd.app` discovery locations are compatibility entries; BR-03 concerns the missing current location.
- Legacy icon filenames and the `Sheprd` asset-catalog key contain the approved rocket artwork.
  The [icon guide](https://github.com/2389-research/spaceport/blob/bd1782f9c6507abe5436983aa31a9be91f2b316f/assets/icons/README.md#L16) explains why consumers and catalog keys must move together.
- Historical guides, tests, fixture workspace names and internal type names are not visible-branding failures by themselves.

App-owned temporary prefixes such as `herdr-gpui-chat-*` remain internal naming debt.
This audit did not establish an upstream-state access or collision caused by those temporary names.

## Method and executed evidence

The documentation pass extracted active identity, command, namespace, setup and discovery claims.
The second pass expanded old CLI, config, IPC, contact, bundle and artwork patterns.
Packaging review followed producers, consumers and actual release gates.
Source review covered Rust strings, Fluent catalogs and generated configuration templates.

- Broad Rust/Fluent/TOML scan: 380 matching source lines, not 380 defects.
- AST string-literal scan: 305 matches; 64 filename-filtered candidates were reviewed with their cfg context and callers.
- Diagnostics probe: actual target list, compiled collector and tracing producer; reproduced BR-01 with two positive controls.
- Discovery probe: actual extracted Rust producer and real shell/filesystem fixtures; reproduced BR-03.
  Absolute app roots were rebased for isolation; this is not remote SSH or Core end-to-end evidence.
- Approved-rocket tests: six passed.
- Linux icon reproducibility tests: three passed.
- Spaceport release-transition tests: four passed, including actual aggregate shell conditions.
- Read-only plist and asset-catalog inspection confirmed metadata/key consistency.
- Fresh public GitHub repository, private-reporting status, paginated release metadata and small appcast reads completed.
  The `releases/latest` endpoint returned 404; the published previews are prereleases and were found through pagination.

Full probe sources, commands, synthetic logs and metadata snapshots live under this worktree's ignored `target/branding-audit/`.
The diagnostics probe links cached Rust 1.99.0 libraries from the PR #19 build, whose source tree matches the audited merge.
Both probes use private state and synthetic data.
No personal app, daemon, transcript, keychain or credential file was used as a fixture.

No application fixes, installs, signing, notarization, publication or private reporting submission ran.
No new full workspace build, native desktop acceptance or published DMG inspection ran.
The feed read checked names and URLs, not its signature or installed updater behavior.
Nix execution and native Linux/Windows behavior remain unverified.
The public website beyond repository/release metadata was outside this audit.

Independent read-only review checked the material findings against source and probe evidence.
Its one correction, an out-of-range source-line link, is fixed.

## Recommended order

Fix diagnostics and current-bundle discovery before the next preview.
Resolve the private reporting recipient promptly with the repository owner.
Then fix the generated config producers and active setup/command documentation.
Qualify the exposed Nix route and replace the public description.
Finish specimen/artwork copy afterward.
Keep existing cross-platform packaging and release gates open until their artifacts are qualified.
