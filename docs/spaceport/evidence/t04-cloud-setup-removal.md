# T04/RM-CLOUD: cloud setup execution cut

Based on `c3cfdf82`, the T04 packet and [connector boundary](t04-container-connector-dependency.md). T04 stays blocked on G-LINK; RM-CLOUD remains partial and A30 is `not_run`. No real engine, agent installer, Core, saved catalog, or user config was used.

The excluded `cloud_setup` worker, pending queue, `tab.create` script launcher, agent-install recipes, host-menu **Run setup script**, Settings setup card/search entry, and related messages/errors are removed. Normal window tick no longer polls or launches a cloud setup tab. The worktree script and chat start slot still use their own existing paths. Old `[cloud]` and `[cloud.container]` keys parse as inert data; a Container engine save leaves them in the private config fixture unchanged. The sample config and current app README no longer offer setup execution.

The saved Container connector, `cloud-devices.json` catalog, provider rail, Settings device card and exact saved-row Forget remain active. A stopped saved Container may still start on connection. This cut makes no claim about Core machine ownership or Container removal. The full connector cut still needs the private old-JSON/startup/retained-action fixture and the coordinated endpoint/view deletion described in the connector boundary.

## Checks and limits

- Red source guard `scripts/release/tests/test_cloud_setup_absence.py` failed 2/3 on the live window tick and menu route before edits, then passed 3/3. Combined with the prior Container absence guards, 10/10 Python checks passed.
- The all-feature app check first failed because removing `Error::CloudSetup` left its `#[error(transparent)]` on `ConfigFile`, producing cascading conversion errors; an unused `SCRIPT_ENV` re-export was also found. Both roots were removed, then `cargo +1.99.0 check -p sheprd-gpui --all-targets --all-features --locked` passed. The default `--all-targets --locked` check passed too.
- Focused all-feature GPUI tests passed: retired cloud config parse (1), private config-file engine save (1), retained cloud device card (7), retained cloud host menu (1), and worktree script tests (7). No agent installer or Container engine ran.
- `just check-i18n` validated 4313 messages in 243 files. `just check-file-size`, `just check-sorted`, `cargo +1.99.0 fmt --all -- --check`, package-root `tools/spec.py check` (schema and 178 local links), Python guards, and `git diff --check` passed.
- Cargo reported the pinned `block v0.1.6` future-incompat notice and the large debug GUI binary's macOS compact-unwind linker warning. Native GUI and cross-platform CI were not run. The retired Container JSON/startup fixture belongs to the future full connector cut, not this worker cut.
