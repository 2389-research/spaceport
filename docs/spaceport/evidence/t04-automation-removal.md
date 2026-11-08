# T04 automation execution removal

This bounded cut starts at `ef75bd1c`. T04 and RM-AUTOMATION remain `in_progress`; A20, A29 and A30 remain `not_run`. It removes the client scheduler and desktop execution surface, not every excluded feature family. G-OBS remains open.

## Removed reachability

- `crates/sheprd-automate`, `crates/sheprd-automation` and their remaining `sheprd-luau` interpreter dependency left the workspace. The GUI no longer starts, polls, deploys, approves, pauses or stops scheduler jobs. Its 15/60-second reader, action dialogs, notifications, timeline projection, log source and sidebar counts are gone (`crates/sheprd-gpui/src/{automations.rs,window.rs,log_window.rs,sidebar/activity/counts.rs}`). `automations.rs` now renders one explicit unavailable message; it neither represents missing jobs as zero nor calls a worker.
- `automation approve` left CLI parsing, help, control protocol and server. The bundled `herdr-automations` skill and current automation guide were removed. Existing skill files in a personal home are never deleted: the shared upstream marker and path lack a Spaceport ownership receipt.
- Cargo, `Justfile`, `flake.nix`, CI, release, desktop installers, site install scripts and package/signing manifests no longer build, copy, sign, launch or stop the scheduler. The macOS, Linux and Windows package tests assert the revised companion set. Previously installed automation binaries or services lack a Spaceport ownership receipt; this cut does not stop or erase them from a user home. The release gate still blocks publication by its separate identity and signing rules; this cut makes no release claim.

## Retained boundary

The Automations activity destination and `[sidebar] automations` preference control only whether an unavailable page is shown. SPEC.md requires passive activity visibility, but `plan/gaps.json` G-OBS and `evidence/t08-observability.md` establish no authoritative Core job/run feed. T08 must supply a supported read-only feed before that page can show facts. No host-script scan, local scheduler, or synthetic empty history remains.

`host_build` digest/copy primitives and `remote/links` still serve the phone supervisor. A local SSH/cloud host match in `remote/links.rs` and `sidebar/tag_groups.rs` replaces the removed automation `Machine` type only to keep that existing T10 path compiling. It is transitional, not a retained product contract. T10 must remove the phone server/tag seam separately. General log viewing, ordinary sidebar navigation, terminal/chat, and existing Core machine display remain.

## Checks

The new CLI rejection and headless unavailable-page tests failed against the base, then passed after removal. The package absence test failed on nine live build/package routes, then passed. Focused Rust check compiled without new source warnings. The full release Python suite passed (190 tests, 9 platform skips); release security policy passed (26 tests, 1 skip). Phone-link tests (4), activity-section tests (4), managed-skill tests (15) and log-window tests (15) passed. The macOS test linker still reports its pre-existing compact-unwind warning; Cargo still reports the pre-existing `block 0.1.6` future-compatibility notice. No native desktop, Windows installer smoke, compiled release package, personal daemon, or Core test ran in this cut. Full A29/A30 and native QA stay open.
