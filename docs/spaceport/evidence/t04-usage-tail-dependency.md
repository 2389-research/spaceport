# T04/RM-PROVIDERS: usage tail dependency

Source audit at `dde45280`. No product code changed in this evidence cut. The
earlier [provider probe removal](t04-provider-probe-removal.md) disconnected
the normal app from usage reads, but the `sheprd-usage` worker, credential
reader, cookie reader, and provider fetch code still compile. T04 remains
blocked on G-LINK, T08 on G-OBS, and A20/A30 remain `not_run`.

`sheprd-usage/src/lib.rs:249-291` still exposes `busy`, `allow`, `refresh`, and
`poll`; `lib.rs:505-549` starts the worker and enters its read loop. The app
has no production caller of these methods after the prior cut. Removing only
that worker was tested in an isolated worktree: a new source guard first
failed on `poll`, then passed 2/2 after the removal; the existing app
probe-absence guard passed 3/3, and the retained `shown_providers` Rust tests
passed 2/2. Default and all-feature app checks completed, but each emitted
four **new** `dead_code` warnings for `CookieJar::forgive`, `Exec` variants,
`Consent` variants, and `Probe::new`/`missing`. Those warnings proved that
the worker was their last production caller. The experimental code and guard
were rolled back; those checks do not qualify a committed worker removal.

The next cut must separate display data from the old fetch graph in one
coherent change:

- `model.rs:50` stores a `&'static dyn Service` in `Provider`; `service.rs:88-93`
  combines `meta()` with `fetch(&mut Probe)` in that trait. `registry.rs:6-93`
  has 87 ordered service entries, matching 87 provider metadata definitions.
  Each provider module holds metadata beside fetch code. Preserve the exact
  IDs and shown order only where a verified feed and retained UI need them.
- The panel, sections, icons, page titles, and saved config examples read
  provider metadata (`usage/panel.rs:95`, `usage/section.rs:106`,
  `usage/icons.rs:344`, `menu/page_title/tests.rs:90`,
  `usage/tests/example_config.rs:15-71`). `settings.rs:69-81` also uses the
  registry to validate old provider sign-in settings. Do not preserve that
  credential configuration as a second way to feed reports.
- `probe/shell.rs:48` defines `Shell` inside the probe graph. Retained
  system-load and listening-port workers call `Shell::connect`,
  `connect_wsl`, `start`, and `run` (`system_load.rs:82-138`,
  `listening_ports.rs:127-152`). Extract its bounded command/session contract
  before removing probe and cookie code. The GUI also renders two report
  detail types from provider modules through `lib.rs:36-40`; keep those model
  shapes with their presentation tests.

G-OBS still has no verified Core usage report identity or schema. The
metadata-only model for authoritative reports cannot yet be specified from
Core, and retaining the old sign-in registry as a placeholder would keep the
excluded provider configuration path. This is a concrete external feed gap
plus a local presentation migration, not permission to invent a feed. No
credential, provider endpoint, personal service, or native UI was exercised
for this audit. The committed change only records this dependency; the
experimental Rust results above describe the rejected approach.
