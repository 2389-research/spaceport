# Reorder and truncation integration

Scope: integrate main `44c5325e8b902f70d90f26d19d9188ae87c49d78` into draft PR 125.
Main adds PR 121's shared reorder geometry and truncation crates.
The topic branch retains the reviewed CI fixture repairs and native Windows process qualification.
Doctor Biz's recorded draft-publication approval covers this necessary topic integration.
No main merge or release is authorized here.

## Retained behavior

Resolved one textual conflict in the sidebar reorder module.
Kept its local keyboard module and captured-input fencing.
Imported the shared geometry types alongside the window type.
Manifest merging retains existing OS dependencies, test dependencies, and the working-session harness.

All three reorder test bodies and all seven truncation test bodies moved intact.
Truncation still calls the same client's safe-text implementation.
The integrated code contains no remaining references to the removed top-level modules.
Independent review found no lost keyboard, truncation, or resource-ownership contract.

## Native motion regression

The extracted reorder crate initially selected zero-duration slides through a dependency feature.
GPUI enabled that feature through a dev dependency.
[Cargo unifies those features during tests](https://doc.rust-lang.org/cargo/reference/resolver.html#feature-resolver-version-2),
including dependencies used by the normal `CARGO_BIN_EXE_spaceport` binary.
Native and performance harnesses would therefore lose the prior 150-millisecond motion.
This was a new extraction regression, despite headless geometry tests passing.

The shared crate now retains a fixed 150-millisecond duration.
Only GPUI's motion policy selects immediate settling under `cfg(test)`.
Workspace, device, and tab slides use that presentation policy consistently.
Removed the temporary dependency feature and its dev-dependency declaration.
Normal application and native-test binaries keep real motion.
The extraction note now describes this actual distinction.

The normal-consumer regression failed first at midpoint progress `1`, proving immediate completion.
It then passed intermediate progression and the 150-millisecond contract.
Cargo built the normal application binary alongside that integration test.
No production diagnostic flag or mock mode was added.
Independent review approved the feature graph, all three callers, and the regression evidence.

## Verification

Passed `cargo metadata --locked --no-deps --format-version 1`.
Passed all three reorder cases and seven truncation cases with all features.
Passed the normal-consumer integration case with all features.
Passed 474 focused GPUI cases, with ten explicit-resource fixtures ignored directly.
The affected parent tests invoke their private fixture entry points.
Passed canonical `just lint` and `just lint-windows` in both feature configurations.
Both commands check all workspace targets with warnings denied.
Passed formatting, file sizes, sorted regions, operations, localization, and specification validators.
The specification gate explicitly reports JSON Schema validation as not run.
Passed both staged and unstaged diff checks.

The GPUI scopes cover drag/reorder, tab strips, browser state, truncation callers, localized errors,
status bars, connection setup, OS-input boundaries, and the corrected platform fixtures.
Both offscreen Metal pixel cases passed immediately before this extraction.
The extraction does not change image geometry or painting code.

Logs use `/tmp/spaceport-9s44-reorder-*.log` and `/tmp/spaceport-9s44-ci208-*.log`.
Earlier hosted native and packaging successes qualify source `208fa3929748d89cc815a79eba1340745b6a7a15`.
They do not establish a passing hosted run for this combined candidate.
Windows OpenSSH and Linux fixture corrections still require their platform reruns.
Foreground Settings, accessibility, native presentation, and release working-session profiling remain pending.
Kata remains 32 closed children and 21 open children after verified Windows process cleanup closed.
