# Just operations

Every recurring operation in this repository is a `just` recipe. Run `just`
(with no arguments) to list them all; `just --dry-run RECIPE ARGS` shows the
generated shell. Agents should reach for these targets instead of re-deriving
command sequences: a recipe encodes the flags, environment, and working
directory that the CI gate or the release process actually expects, and a
sequence that cannot be perfectly reconstructed a second time must become a
recipe rather than a note.

## Verification (quick, run before handoff)

| Recipe | What it does |
| --- | --- |
| `just format-check` | rustfmt across the workspace |
| `just lint` | clippy, default and no-default-features, warnings denied — this is where each function's cognitive-complexity (25) and line (100) ceilings, set in `clippy.toml`, are enforced |
| `just test` | the whole workspace test suite, default and all-features |
| `just nextest *args` | the same suites under cargo-nextest |
| `just check-file-size` | the 1,000-line source-file ceiling |
| `just check-sorted` | `keep-sorted` marker regions |
| `just check-i18n` | Fluent catalog: messages valid and used |
| `just check-spec` | the `docs/spaceport` package's validator + tool tests |
| `just check-just-operations` | every recipe is documented in this guide (self-enforcing) |
| `just check-artwork` | generated icon sources match their inputs |
| `just check-commits` | conventional-commit subjects over `origin/main..HEAD` |
| `just ci` | all of the static checks, lint, and tests — the pre-push gate |

## Focused and repeated test loops

| Recipe | What it does |
| --- | --- |
| `just test-gpui filter reps` | the `spaceport` bin's tests, filtered; build once, then drive the test binary directly for cheap repetitions. Example: `just test-gpui "endpoint::" 10` |
| `just test-protocol` | the five small protocol/client crates' suites in one command |
| `just test-release-scripts` | `scripts/release/tests` the way CI invokes it (env, cwd included) |
| `just check-spec-tools` | the spec package's own `test_spec.py` + `spec.py check` |
| `just stress-gpui filter reps cores` | a focused gpui filter under synthetic full CPU load, for flake-hunting |

## Opt-in integration tests (external resources required)

| Recipe | What it does |
| --- | --- |
| `just test-ios-duo device` | fold and unfold a selected iPhone Duo simulator while typing into a disposable SSH session; see [mobile setup](../mobile/README.md) |
| `just test-live binary` | live Herdr 1 daemon tests; requires an explicit `HERDR_TEST_BINARY` |
| `just test-core binary` | herdr-core tests; ends the core's terminals on exit |
| `just test-gui binary` | real native window tests; needs an active desktop |
| `just test-sidebar` / `just test-sidebar-routing` | native font/routing suites |
| `just test-perf budget` | release-mode performance budget |
| `just stress-linux iterations cpus filter` | the Linux stress loop for endpoint tests (CI-adjacent; see the recipe) |

## Building and running

| Recipe | What it does |
| --- | --- |
| `just run args` | launch the bundle (macOS reads identity/privacy from it); non-macOS runs cargo |
| `just default` | list every recipe (what bare `just` prints) |
| `just run-debug *args` | debug-profile equivalent |
| `just build-release` / `just test-build` | release build, then smoke the built CLI |
| `just bundle profile features` | produce the app bundle |
| `just install` | install the built binary |
| `just icons` / `just icons-linux` / `just icons-windows` | regenerate icon assets |
| `just dmg VERSION profile arch` | build a disk image (debug default) |

## Maintenance and release

| Recipe | What it does |
| --- | --- |
| `just sort` | fix unsorted `keep-sorted` regions |
| `just format` | apply rustfmt |
| `just hooks` | install git hooks: commit-msg check, rerere, Cargo.lock merge driver |
| `just audit-deps` | `cargo deny check` over the dependency graph |
| `just audit-workflows` | GitHub Actions security audit |
| `just lint-windows` / `just lint-linux platform` | cross-platform lint investigation |
| `just changelog` / `just changelog-unreleased` | render the git-cliff changelog |
| `just release-check` | static release checks: bash syntax, release tests, actionlint, zizmor, whitespace |
| `just setup-ci` | install CI's tool dependencies |

## Conventions

- Recipes whose path or environment matters (`test-release-scripts`,
  `check-spec-tools`, `release-check`) encode it themselves; do not `cd` around
  them.
- New recurring operations belong here as recipes with a comment saying what
  they are for — including verification loops a worker derived once, if a
  second agent would need them again.
- `check-spec` and `release-check` are CI gates; changes to the CI matrix in
  `.github/workflows/ci.yml` must keep both runnable locally.
