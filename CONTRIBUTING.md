# Contributing

Thanks for looking at Spaceport. This file covers the mechanics; the
engineering rules that actually govern changes live in
[AGENTS.md](AGENTS.md) — architecture boundaries, Rust conventions, and
verification expectations. Read it before a non-trivial change. It applies to
human and agent contributors alike.

## Before You Start

- **macOS, Linux, and Windows builds.** GPUI uses Metal on macOS, Vulkan
  with X11/Wayland on Linux, and DirectX on Windows. Active CI covers macOS
  and Linux. Windows CI is paused until the native performance issue is fixed
  under [decision D66](docs/spaceport/plan/decisions.json).
- **Choose a terminal backend.** Spaceport connects to separately installed
  [Herdr 1](https://github.com/herdrdev/herdr) and Herdr Core runtimes.
  Spaceport may start an installed local runtime when needed. Explicit socket
  connections remain attach-only. It never bundles, installs, updates, stops,
  or migrates either runtime. Daemon process,
  session or socket bugs belong upstream; `sheprd-terminal` emulation and
  native presentation bugs belong here.
- **Open an issue first for anything large.** Agree on the scope and approach
  before starting a substantial implementation.

## Setup

Install rustup and, on macOS, the Xcode command-line tools. On Ubuntu 24.04,
run `bash scripts/install-linux-deps.sh` for build libraries and fonts;
see [the fork guide](README.md#work-on-the-fork) for current requirements
and limitations.
The toolchain is pinned in
`rust-toolchain.toml` and GPUI is pinned to an exact version, so `rustup show`
is enough — do not upgrade either as a side effect of another change. `mise.toml`
mirrors the same Rust version for [mise](https://mise.jdx.dev) users (`mise install`);
change both files together.

```sh
just run          # optimized build; use this for anything interactive
just run-debug    # unoptimized, notably slower with a dense terminal
```

On macOS these launch `target/<profile>/Spaceport.app`; the visible app is
**Spaceport** and its executable is lowercase `spaceport`.

Naming changes follow [the Spaceport specification](docs/spaceport/SPEC.md).
Preserve retained protocol contracts, legal notices and personal settings.
The [upstream compatibility inventory](docs/rebranding.md) is historical reference.

## Verifying a Change

Run relevant quick checks before opening a PR:

```sh
just format-check check-file-size check-sorted check-i18n
```

Run targeted tests that fit the short local validation pass. Leave longer tests,
full workspace gates (`just ci`), cross-platform lints and release builds to CI.
Run longer checks locally when requested or needed to investigate a failure.
Active CI checks formatting, lints and tests default and all features on macOS
and Linux. It also builds Apple Silicon, Intel macOS and Linux x86_64/ARM64
binaries, then exercises their release CLI without a desktop. Use `just test-build`
to investigate linking or packaging failures. See [release tooling](scripts/release/README.md)
for packaging checks and prerequisites. External contributors require GitHub
workflow approval. See the
[contributor CI policy](scripts/release/README.md#contributor-ci-approval).

Some tests are opt-in because they need explicit external resources:

```sh
just test-live /absolute/path/to/herdr   # needs an explicit daemon binary
just test-gui  /absolute/path/to/herdr   # also needs an active desktop
just test-sidebar                        # native glyph regression
just test-perf                           # release-mode scene budget
```

Never point a live test at your personal daemon — they create and clean up
their own isolated one. For visual changes, check the real native window:
narrow layouts, long labels, focus, and clipping. Headless layout tests do not
prove native glyph or input correctness.
Linux X11 and Wayland both need manual desktop coverage; the performance harness
and AppKit click probes remain macOS-only. The GUI test sandbox resolves the
parent Wayland socket without exposing its HOME or runtime directory to daemons.

## Pull Requests

- Use conventional commit subjects: `feat`, `fix`, `perf`, `refactor`, `revert`,
  `docs`, `chore`, `ci`, `build`, `style`, `test`. Explain the problem and the
  chosen approach in the body for non-trivial work.
- Subjects are published: the release changelog is generated from them, so
  `feat` appears under Added, `fix` under Fixed, `perf`/`refactor`/`revert`
  under Changed, `docs` under Documentation, and `chore`, `ci`, `build`,
  `style` and `test` are never shown to users. Write the subject for someone
  reading a release page, and keep rationale in the body; only the first line
  is published. A breaking change is always published: mark it `type!:` or add
  a `BREAKING CHANGE: <what breaks>` footer.
- Run `just hooks` once per clone to install the `commit-msg` hook that checks
  this locally; CI checks it again for every commit in a pull request. Preview
  the result with `just changelog-unreleased`. The same recipe enables
  `rerere` and a `Cargo.lock` merge driver; see Parallel Pull Requests in
  `AGENTS.md`.
- `main` requires a pull request and green CI. Review your own full diff
  against the base before marking it ready.
- State the validation you actually ran, and say plainly which native or
  manual checks you could not run. Do not describe a headless test as
  end-to-end verification.
- Keep user-facing docs in step with behavior changes.

## License

By contributing, you agree that your contributions are licensed under
Apache-2.0, matching [LICENSE](LICENSE).
