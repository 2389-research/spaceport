# T04 palette repository clone removal

Source base: integration `ece5aae07`. The project palette offered `git clone`
for explicit repository URLs, using a local child process and a staging folder.
That Git action was reachable after the native Git menu and PR routes were cut.

The palette now offers only a new folder under the first configured project
root. It rejects HTTPS, SSH and scp-style repository URLs as project names.
The folder action still creates the directory off the UI thread, refuses an
existing name, resolves the path and opens it through the existing project
flow. The clone command, staging/publish helpers, clone error and related
messages were removed. No repository was contacted and no user folder was
changed.

## Reproduction and checks

The new `project_palette_does_not_offer_repository_clones` test failed first:
the HTTPS URL returned `Source::Clone`. After the cut,
`cargo +1.99.0 test -p sheprd-gpui --bin spaceport
palette::project_create::tests --locked --offline` passed 7/7, including
folder creation, duplicate refusal and URL rejection. The first green run
found an unused `pull_request::run` re-export; that dead import was removed,
and the second run passed with only the existing macOS compact-unwind linker
warning and `block v0.1.6` future-compatibility notice. `just format-check`,
`check-file-size`, `check-sorted`, `check-i18n` (3,349 messages in 210 files),
and `git diff --check` passed.

This covers the project palette route only. Other Git readers and embedded
browser routes remain separate T04 work. A29 stays `not_run`, T04 stays
`blocked` by G-LINK, and native/macOS, Windows and Linux application checks
have not run for this change. Integrated project-open and all-feature checks
are pending after cherry-pick.
