# T04/RM-GIT: local folder-open command cut

Base revision: integration `956ddf831`. The `spaceport open DIR` CLI family
validated a local folder, started the GUI if needed, and sent `AppOp::Open` to
the running app. The app then used the legacy palette's Open Project path to
focus or create a Herdr workspace. This was a separate route from the removed
`spaceport://open?path=...` link and from ordinary terminal/chat file actions.

The cut removes the CLI family/help, retry/start worker, control protocol
variant/handler, and the palette helper used only by that handler. The
remaining `app focus` and other local CLI commands keep their existing control
request and target rules. The `spaceport://device` and notification links are
unaffected. The shared palette's own project picker remains for a later full
legacy-workspace cut; this change does not claim it has been removed.

The Inno Setup recipe and three mirrored PowerShell installer sources no
longer register the Explorer folder/background verbs that called the removed
command. The PowerShell source no longer removes those generic `HerdrUI` keys
on install or uninstall, because their owner may be an upstream installation.
The Windows installer smoke source no longer expects or polls for those verbs.
It still contains upstream release identities and link handlers under the
separate G-RELEASE/T12 gate. No registry or installed app was changed here.
`docs/windows-qa.md` is explicitly marked as an inherited upstream prompt;
its old Sheprd installation steps are not current Spaceport instructions.

TDD evidence: the new protocol test first decoded `app.open` successfully and
failed; after removing the variant it passed. The CLI parser test first parsed
`open .` and failed; after removing the family it rejects `open .` and
`open --help` before GUI startup. The installer source test first found all
four verb registrations and failed; after the source cut it passed. The
retained `app focus` parser case and protocol `app.focus` decode remain in
these tests. A binary CLI test checks actual exit status and help text.

Local checks on macOS arm64: the focused Python installer suite passed six
tests with two Windows-only skips. The real `spaceport` CLI integration suite
passed 19/19, the control protocol library passed 60/60, and eight app control
tests passed. `cargo +1.99.0 check --locked --offline -p sheprd-gpui -p sheprd-control-protocol --all-targets` passed. These Rust commands used
`CARGO_INCREMENTAL=0` and the shared integration target. `just format-check`,
`just check-file-size`, `just check-sorted`, and `git diff --check` passed. The package validator
passed schema and local-link checks, and all 21 package tool tests passed;
neither proves application behavior. `just check-i18n` still reports the two
clean-base findings at `menu/github.rs:760` (`GitHub`) and
`locales/en-US/browser.ftl:5` (`browser-no-pages`). The macOS test binary emitted
the inherited compact-unwind linker warning and `block v0.1.6` future-compat
notice. Native Windows registry
behavior, macOS UI and Linux package behavior have not been tested by this
source cut. T04 remains blocked by G-LINK and other excluded routes; A29
stays `not_run`.
