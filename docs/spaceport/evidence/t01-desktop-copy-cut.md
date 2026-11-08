# T01/T09 retained desktop copy cut

Base: `wip/spaceport-t00` at `08f76a38`. This cut changes 22 English Fluent
values in nine files. Source readers confirm the keys appear in the retained
app menu (`menus.rs`), command catalog (`sheprd-commands`), settings pages,
config diagnostic cards, clipboard errors, and local control errors. Message
IDs and Rust identifiers remain intact; `Herdr` remains where the text names
the upstream runtime. No provider, Git/worktree, browser, legacy connection,
updater, bundle, or release ownership text changed.

`scripts/release/tests/test_desktop_product_copy.py` checks the exact 22 keys.
Its first run failed all 22 subtests on `Sheprd`; it passed after the value
edits. The test checks the English source values, while `just check-i18n`
checks Fluent syntax, use, and references. Neither is a native screenshot or
an installation test.

## Remaining copy at this revision

Source search still finds 55 lines with `Sheprd` in 26 English Fluent files.
`en-US` is the only locale directory; there are no separate translated files
to update. The remaining files have distinct owners or need a route decision:

| Ownership | Files still containing `Sheprd` |
| --- | --- |
| Live retained or migrating paths, pending route review | `add-device.ftl`, `error-chat.ftl`, `error-devices.ftl`, `host-menu.ftl`, `settings-integrations.ftl` |
| Excluded Git, browser, cloud, orchestration, or usage-provider paths | `error-orchestrator-azure.ftl`, `error-remote-container.ftl`, `error-remote.ftl`, `error-setup-relay.ftl`, `github-menu.ftl`, `menu-page.ftl`, `settings-cloud-azure.ftl`, `settings-cloud-container.ftl`, `settings-cloud.ftl`, `usage-jetbrains.ftl`, `usage-openrouter.ftl`, `usage-windsurf.ftl`, `worktree-source.ftl` |
| Legacy Herdr connection paths | `error-client-ssh.ftl`, `error-client.ftl`, `link.ftl`, `version-mismatch.ftl` |
| Release and updater identity, gated by G-RELEASE | `error-updater.ftl`, `update-panel.ftl`, `whats-new.ftl`, `window.ftl` |

These groups are ownership notes, not proof those routes are absent. In
particular, the five live or migrating files need their backend/retained-UI
owners to decide the copy along with behavior. The release files must not
imply a Spaceport-signed update while upstream signing identity remains.

## Checks on this branch

- Red: `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest scripts.release.tests.test_desktop_product_copy` failed 22 subtests on the old values.
- Green: the same focused test passed one test with 22 checked keys.
- `just check-i18n`: 20 checker tests passed; 4,576 messages in 248 files
  parsed, were used, and had valid references.
- `just format-check`, `just check-file-size`, `just check-sorted`, and
  `git diff --check` passed.
- Package `spec.py check` passed with the existing package virtualenv (10
  strict JSON files and 92 local links checked). A Rust production check was
  not run in this isolated branch; the integrating agent will run it after
  the shared target is free.

T01/T09 remain in progress. A42 stays not run. G-RELEASE remains open. This
source/i18n cut does not prove full product branding, native menu appearance,
or a releasable package.
