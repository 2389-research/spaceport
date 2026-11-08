# T12: CLI guide migration boundary

At merged source `7016ac25`, the active `docs/cli.md` still named the removed
code/review tabs, upstream state socket path and a bundled Core. The guide now
identifies the current development executable as `sheprd`, lists the live tab
families, and gives the Spaceport state socket path derived from
`sheprd_client::local_path::state_dir` and the Unix control socket code. It
states that Herdr 1 startup is still a migration defect and that Spaceport
may start only a separately installed, trusted Core when needed. This is a
documentation correction, not a product rename or Core lifecycle test.

The path and command names were reviewed against source; `git diff --check`
passed. The specification package check passed for links and schema. No CLI
binary, native UI, package, updater, Windows pipe or release install was run
for this documentation change. The rest of the inherited CLI guide still
contains commands and features under T04 removal; T12 and A42–A44 remain open.
