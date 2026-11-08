# Source evidence and scope

**Review date:** 7 November 2026. **Sheprd:** `806b49870ac7761b98cbcea4bd115aa2f2b00d75`. **Herdr Core:** `cd12ddd88bc11490228343291dc1096c9ed3bf77`.

These revisions were obtained from the repositories during this task. They are investigation baselines, not verified release pins. Source was read through the connected GitHub tools; a container-side attempt to retrieve full source bytes failed because network name resolution was unavailable. No repository checkout, application build, live daemon test or native-platform test was performed.

A Library search did not locate a Spaceport-specific design document to patch. This package consolidates the user decisions in the conversation into new authoritative documents; it does not modify unrelated Library documents or a remote repository.

## Primary sources

### S01 — `herdrdev/herdr-core/docs/machines.md`
[Pinned source](https://github.com/herdrdev/herdr-core/blob/cd12ddd88bc11490228343291dc1096c9ed3bf77/docs/machines.md) · Core-owned remote machines, aggregation, reconnect and file operations.

### S02 — `herdrdev/herdr-core/schema/README.md`
[Pinned source](https://github.com/herdrdev/herdr-core/blob/cd12ddd88bc11490228343291dc1096c9ed3bf77/schema/README.md) · Wire protocol, attach streams, events, sequence resets, opaque documents and reported status.

### S03 — `herdrdev/herdr-core/crates/herdr-core/src/api.rs`
[Pinned source](https://github.com/herdrdev/herdr-core/blob/cd12ddd88bc11490228343291dc1096c9ed3bf77/crates/herdr-core/src/api.rs) · Complete method table and hello capability declaration; inspected lines 1–210.

### S04 — `sheprd-labs/sheprd/docs/session-model.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/docs/session-model.md) · Actual split between Herdr 1 workspaces and Core terminal/layout state; migration plan is not implemented parity.

### S05 — `sheprd-labs/sheprd/docs/rich-chat.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/docs/rich-chat.md) · Herdr 2 chat endpoint labels, local-only native attach, conversion and resume side effects; inspected lines 1–190.

### S06 — `sheprd-labs/sheprd/crates/sheprd-acp/README.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/crates/sheprd-acp/README.md) · Host-side conversation adapter, attach protocol, agent ownership and restore behavior; retrieved output was truncated after the behavior section.

### S07 — `sheprd-labs/sheprd/docs/cli.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/docs/cli.md) · Existing private local control interface, JSON output, targeting and command families; inspected lines 1–210.

### S08 — `sheprd-labs/sheprd/Cargo.toml`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/Cargo.toml) · Current crate inventory and dependencies; inspected lines 1–150.

### S09 — `sheprd-labs/sheprd/AGENTS.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/AGENTS.md) · Crate responsibilities and engineering conventions; requested lines 1–220, output was truncated in Rust conventions.

### S10 — `sheprd-labs/sheprd/crates/sheprd-core-client/src/method.rs`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/crates/sheprd-core-client/src/method.rs) · Existing Core adapter exposes only a subset of the current Core API.

### S11 — `sheprd-labs/sheprd/crates/sheprd-terminal/README.md`
[Pinned source](https://github.com/sheprd-labs/sheprd/blob/806b49870ac7761b98cbcea4bd115aa2f2b00d75/crates/sheprd-terminal/README.md) · Pure Alacritty-based emulator, damage, input and bounded rendering contracts.

### S12 — `herdrdev/herdr-core/schema/hc.openrpc.json`
[Pinned source](https://github.com/herdrdev/herdr-core/blob/cd12ddd88bc11490228343291dc1096c9ed3bf77/schema/hc.openrpc.json) · Inspected lines 1–160: persistent core identity, attach flags and versioned document result types; not a full schema audit.

## Interpretation rules

Facts about existing software are limited to the inspected source scope. API shape is not proof of tested runtime behavior. Absence of a dedicated method in the inspected table is not proof that no supported route could ever exist; affected capabilities remain investigation gates.

User scope choices come from this conversation, not from upstream documentation. All new architecture, interface budgets, task sequencing, workspace view schema and acceptance criteria are proposed engineering requirements for this fork. No trademark, domain, signing-account or production-infrastructure ownership has been verified.
