# Usage presentation

Spaceport retains usage panels, rate-limit windows, balances, account details,
and daily spending charts. The selected report leads the status bar, followed
by reports closest to their limits. Missing observations remain unavailable,
not zero usage. No authoritative runtime usage feed has been verified yet.

`sheprd-usage` contains report models, static provider display metadata and the
existing bounded host shell used by listening-port inspection. It contains no
provider polling, sign-in, cookie extraction, credential discovery or provider
HTTP requests. Provider names, icons and external dashboard/status links remain
presentation metadata; they do not imply a working feed or sign-in.

The `[usage]` settings `show`, `topbar`, `inline`, `show_providers` and
`hide_providers` remain display preferences. `browser_cookies` and
`[usage.providers]` configuration are rejected. The app never imports the old
provider credentials or reads provider environment variables.

Native rendering lives in `crates/sheprd-gpui/src/usage/`; CLI report conversion
lives in `control/server/hosts.rs`. The report models and metadata are in
`crates/sheprd-usage/src/model.rs`, `details.rs` and `registry.rs`. The shell in
`crates/sheprd-usage/src/shell.rs` preserves bounded output, timeout fencing,
SSH channel ownership, WSL platform checks and child cleanup. It runs only
through its existing retained callers; it is not a usage feed.
