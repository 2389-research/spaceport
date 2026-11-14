# Spaceport

<!-- impeccable:product-schema 1 -->

## Platform

Native desktop, built with GPUI.
This is not an iOS, Android, or web application.

## Product Purpose

Spaceport presents terminal and agent sessions owned by separately installed Herdr runtimes.
People inspect sessions, navigate between them, and interact through the native client.

## Capabilities and Constraints

Keep the existing GPUI interface and terminal engine.
Preserve runtime ownership when a view closes or changes.
Herdr Core remains an optional experiment, disabled by default.
Treat session output as untrusted content.
Preserve compatibility identifiers and upstream attribution.

The authoritative product scope remains in [docs/spaceport](docs/spaceport/START_HERE.md).
The approved overview-chat experience is recorded in its [design note](docs/spaceport/notes/overview-meta-chat.md).
Its final interaction design and implementation remain in progress.

## Brand Commitments

Use Spaceport as the application name.
The approved application icon uses a flying saucer.
Credit the Herdr and Sheprd teams.

## Product Principles

- Preserve one authority for each fact and side effect.
- Make targets, evidence, freshness, and uncertainty visible.
- Keep ordinary operation independent of model calls.
- Preserve the existing native desktop experience.
