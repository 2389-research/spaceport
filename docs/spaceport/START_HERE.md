# Spaceport — fork specification and coding-agent handoff

**Revision 1.0 · 7 November 2026 · Desktop implementation specification.**

Build **Spaceport**, an independent, fully rebranded fork of Sheprd (formerly herdr-gpui). Keep the desktop experience the user likes. Remove the unwanted product surfaces and backend duplication. Consume a separately installed **Herdr Core**; do not replace it.

This package is a specification plus working **specification-navigation/validation tools**. It is not a fork, an application implementation, or evidence of successful runtime integration. No remote repository has been changed.

## Start here, not with the entire repository

1. Read [AGENTS.md](AGENTS.md), then [SPEC.md](SPEC.md) and [SYSTEM.md](SYSTEM.md).
2. Install the specification-tool dependency as described in [tools/README.md](tools/README.md). Run `python tools/spec.py check`, `python tools/spec.py ready`, then `python tools/spec.py context T00`.
3. Follow [the delivery plan](docs/04-delivery-plan.md). Complete the source/capability baseline before bulk deletion.

[BOOTSTRAP_PROMPT.md](BOOTSTRAP_PROMPT.md) is the prompt to give the implementing agent. `READING_COPY.md` is a generated combined reading copy, not another source of truth.

## The whole system in one paragraph

Herdr owns live terminals, machine connections and execution. Host-side agent programs own their conversations and agent-specific behavior. Spaceport owns views, terminal emulation, input translation, presentation metadata and desktop integration. A thin Core adapter produces one inspectable, explicitly versioned projection. The GUI and existing local CLI use the same command semantics. Every change is linked to a user decision, acceptance scenario and source revision. Useful discoveries become a test, a small contract or a corrected document—not another service or an ever-growing prompt.

## Placement in the fork

Keep this package intact under `docs/spaceport/` in the new fork, or at one explicit external specification root while bootstrapping. Run package tools from that root; their `--root` option also accepts it. Candidate code paths in the deletion/source inventories are relative to the **application repository**, not the package. Add a short pointer to the fork's root `AGENTS.md` while preserving applicable upstream engineering rules; do not overwrite that file blindly. The package is not a substitute for reading the actual source.

## Document authority

| Artifact | Owns |
|---|---|
| `plan/decisions.json` | Historical user decisions and their qualifications; generated view in `docs/DECISIONS.md` |
| `SPEC.md` | Product requirements, identified by `SP-xxx` |
| `SYSTEM.md`, `docs/01-*`, `docs/02-*`, `docs/06-*` | System boundaries and implementation contracts |
| `plan/tasks.json` | Work graph and task status; no parallel handwritten progress tracker |
| `plan/acceptance.json` | Planned scenarios and their actual execution status; initially all `not_run` |
| `plan/capabilities.json` | Source-observed backing mechanisms and unresolved compatibility gates |
| `plan/removals.json` | Feature disposition and candidate code/dependency roots |
| `plan/sources.json` | Exact upstream evidence revisions; not deployment pins |
| `evidence/validation.json` | Checks actually run on this specification package |

User decisions outrank implementation convenience. Source establishes what exists, not what the user wants. A proposal, available method, successful compile, and verified behavior are four different states.

## Three corrections carried into the plan

**Remote Core support exists.** Current Core documentation assigns SSH connections, reconnects, and terminal/event aggregation to the local Core. The old “Core is local-only” premise described Sheprd's integration, not Core's actual capability. [S01, S02]

**Core is not the old workspace daemon.** Sheprd's session-model document distinguishes Core terminals from the older daemon's workspaces/tabs/agents. Workspace views need an explicit presentation model, not invented `workspace.*` RPCs. [S04]

**Rich chat is not a terminal engine feature.** It uses a host-side adapter. Sheprd documents local native chat attachment for Core, with remote Core terminals retaining their terminal view. Preserve the renderer, prove the route, and record the remote-chat gap rather than building another server. [S05, S06]

Sources are linked in [SOURCES.md](docs/SOURCES.md). The engineering resolutions are in [decision reconciliation](docs/07-reconciliation.md).
