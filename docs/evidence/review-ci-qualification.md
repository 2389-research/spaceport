# Draft review CI qualification

Scope: repair failures from [PR 125's first complete run](https://github.com/2389-research/spaceport/actions/runs/44421915465).
The tested public head was `6d89f5e42c0478aa865df329720f4a017e3764f5`.
This run failed. Earlier focused local results did not prove platform qualification.

## Repairs

Linux Clippy rejected an attribute gap and a redundant qualification.
Removed both without changing runtime behavior.
Linux verification remains assigned to hosted CI.

Windows native fixtures requested 480 pixels while ordinary windows enforced a 640-pixel minimum.
The actual window-options producer now exempts explicit isolated fixtures from that minimum.
Ordinary windows retain their 640-by-400 minimum.
The strict native width assertion remains unchanged and now reports the observed width.
The regression first failed with the ordinary minimum and then passed with the exemption.
Windows native qualification remains pending.

The ACP discovery fixture used Unix paths on Windows.
It now supplies native absolute paths through `join_paths`.
The production security filter remains unchanged.

Windows resources still embedded the inherited product name.
Both executables now read the canonical dependency-free product-name module.
The existing installer smoke checks both executables' description and product-name fields.
It retains shortcut targeting and AppUserModelID checks.
Real Windows metadata verification remains pending.

Global gesture feedback had lost its terminal-cell placement.
Terminal feedback now uses the actual padded cell container.
Other views retain window-body feedback.
The regression suite keeps padding and Devices/Zen visibility assertions.
Independent review caught an intermediate paint-order error.
The chat hover regression fails when chat paints over the warning.
Its first fixture attempt placed the warning outside chat and did not prove that regression.
The corrected fixture explicitly uses the top-left position inside the chat pane.

The retry-dialog fixture observed a retired handle before the reader finished draining.
It now waits for the production `retry_ready` fence before advancing the retry clock.
The dialog and generation assertions remain intact.

The notice-expiry fixture now starts its expiry interval after publishing the replacement.
This avoids measuring its reading time from before the replacement exists.
The CI failure did not reproduce locally; this timing explanation remains an inference.
The test still rejects inherited hover pauses.
The smoke error test now checks the typed source's current Display text.

## Isolated macOS runtime

Ten all-feature failures explicitly reported a missing Sparkle library before fixture entry.
Two additional child failures omitted stderr and had the same launch shape.
Their failure messages now include their private fixture logs.
Their deadlines and cleanup assertions remain intact.

The `stage-tests` command validates SDK bytes against the pinned archive.
It exposes that framework at both existing Cargo executable-relative lookup locations.
It refuses conflicting files and preserves them for diagnosis.
CI stages these links after restoring the Cargo cache and before all-feature tests.
Production linker paths and sandbox environment clearing remain unchanged.

For local all-feature tests, use the existing pinned SDK preparation first.
Then stage the framework in the selected Cargo target's debug profile:

```sh
python3 scripts/release/sparkle_framework.py stage-tests target/debug --sdk "$SPARKLE_FRAMEWORK_PATH"
```

If `CARGO_TARGET_DIR` is set, replace `target/debug` with that directory's `debug` path.
The command prints both prepared locations.
This prepares test dependencies; it does not qualify updates or release packaging.

## Verification

The native runtime regression failed before `stage-tests` existed.
Both runtime cases then passed against the real pinned SDK.
The test compiles real Objective-C executables and runs them with an empty environment.
It covers both Cargo executable layouts and repeated staging.
The conflict case verifies that an existing file survives refusal.

The final all-feature scopes pass 109 cases with ten explicit resource fixtures ignored directly.
Their parent tests invoke the applicable private fixtures.
The scopes include window options, padding, chat hit testing, Devices/Zen feedback, and the reported macOS failures.
All nine feedback cases pass after correcting paint order and fallback-group selection.
ACP path discovery passes two cases on macOS.
The existing Windows package script suite passes eight cases.
These macOS results do not prove Windows behavior.

Logs use `/tmp/spaceport-9s44-ci-repair-final-tests.log`, `ci-acp-native-paths.log`,
`ci-windows-package-unit.log`, `sparkle-stage-green.log`, and `flash-layer-verified.log`.
All short filenames share the `/tmp/spaceport-9s44-` prefix.

Formatting, size, sorted regions, operations, localization, and specification validators pass.
Both touched application crates pass all-target, all-feature Clippy with warnings denied on macOS.
The specification gate explicitly reports JSON Schema validation as not run.
Hosted platform checks, native Settings input, accessibility, and release profiling remain pending.
No Kata issue closes solely because these local checks pass.

## Canonical workflow steps

The [updated CI run](https://github.com/2389-research/spaceport/actions/runs/44431160334) tested head `208fa3929748d89cc815a79eba1340745b6a7a15`.
Three default jobs stopped at the existing release-security contract test.
That test requires each workspace test command in its own canonical step.
The inline framework preparation had changed that step's shape.

Moved framework preparation into a separate macOS/all step immediately before the unchanged workspace test.
The step still runs after Cargo cache restoration and uses the verified SDK.
Permissions, triggers, credentials, and required test gates remain unchanged.
The three release-security checks and all 24 CI policy checks now pass locally.
Independent review approved the condition, ordering, and preserved security contract.
The other platform jobs still require their actual hosted results.

## Platform fixture qualification

The same run completed with twelve passing checks, ten failures, and one skipped target.
The failure count includes the aggregate gate.
Both Windows native GUI jobs, both macOS release builds, and the Windows installer job passed.
macOS all-feature workspace tests and native host routing passed.
The GPUI scope reports 4,545 passed and 23 explicit-resource cases ignored.
Windows ARM skips its documented live-daemon case; both Windows performance steps remain skipped.

The remaining failures fall into three groups:

- Three default jobs hit the canonical workflow-step contract described above.
- Both Linux all-feature jobs found two shortcut assumptions and a floating-point assertion.
- Four Windows workspace jobs failed the same three isolated OpenSSH parser cases.

Linux intentionally leaves layout-dependent symbol shortcuts unbound.
The config test now derives GUI defaults from the typed command catalog before adding daemon chords.
The sidebar layout fixture uses a real default when one exists, or the production Keybinds handler otherwise.
Its editing gestures use the existing platform Edit keys.
Focus, search, composition, and layout assertions remain intact.

The image test compared independently reconstructed `f32` texture sizes bit for bit.
Linux reported `19.999996` versus `20`: two ULPs, about four millionths of a pixel.
The mapping assertion now permits four relative floating-point epsilons.
Shared mask edges still compare exactly; fragment count, order, and cache assertions remain unchanged.
Both offscreen Metal pixel regressions pass again.
Independent review approved this arithmetic bound against the actual geometry producer.

[Microsoft's OpenSSH source](https://github.com/PowerShell/openssh-portable/blob/0172cc6adb99d8b4324b8f0391b5b1fd6f39b2c6/contrib/win32/win32compat/w32fd.c#L230)
requires `ProgramData` during startup, before initializing stderr.
The cleared parser fixture omitted that variable, matching its blank startup failures.
The fixture now supplies private `ProgramData` and retains only `SystemRoot` alongside its private `HOME`.
It preserves the resolved executable, explicit private `-F` configuration, and all three forwarding-policy assertions.
Failure diagnostics include the executable, status, and stderr.
The installed runner OpenSSH version was not captured; native Windows confirmation remains pending.

All three real OpenSSH parser cases and twelve affected GPUI cases pass locally on macOS.
These results do not qualify the Windows or Linux branches.
The complete four-file fixture correction received independent review.
Kata `rder` separately closed from verified native process results; see [Windows evidence](review-windows-safety.md).

## Shortcut snapshot checkout

Source `5f788861a7c8eced4c7ef197903fb0b5288603bb` passes both Windows native GUI jobs.
The x64 workspace suite passes all 279 client cases, including the three OpenSSH parser regressions.
It then fails the command catalog snapshot without reporting any changed lines.
See [the actual Windows job](https://github.com/2389-research/spaceport/actions/runs/44437876917/job/135271422037).

An actual Git checkout with `core.autocrlf=true` reproduced CRLF conversion of that snapshot.
The existing exact-byte snapshot test then failed with the same empty line diff.
The fixture now has a specific `text eol=lf` attribute, matching its generated format.
The same checkout preserves identical LF bytes, and all 38 command tests pass against those bytes.
The snapshot content, ordering assertion, and shortcut defaults remain unchanged.
The checkout reproduction is throwaway; it restores the original fixture in a `finally` block.
Logs are `/tmp/spaceport-9s44-shortcut-checkout-red.log` and `shortcut-checkout-green.log` with the same prefix.
Actual Windows workspace execution of this attribute correction remains pending.

The new x64 native job passes seven cases, including live Herdr, local controls, and sidebar checks.
The ARM native job passes six cases and retains its documented live-daemon exclusion.
Both skip the symbol cascade because no Nerd Font is installed; performance remains separately gated.
These jobs do not qualify foreground Settings input, accessibility, or release working-session profiling.

## Close batch contention

The same run ends with fourteen passing checks, eight failures, and one skipped target.
The failures include the aggregate gate, all four Windows snapshots, both macOS notice fixtures, and one Linux close sequence.
Both Linux all-feature jobs and ARM default pass; x64 default fails the close sequence.

The accepted batch treated a busy inbox mutex as failed ownership and discarded the remaining closes.
The reader legitimately holds that mutex while publishing an event.
This is a production race, confirmed by independent source review.
New held-lock regressions fail before the repair with `busy inspection must wait`.
An initial fixture response omitted its correlation ID; that setup failure did not prove the regression.

Batch inspection now distinguishes `WouldBlock` from invalid ownership and a poisoned mutex.
A busy inspection preserves the phase and its original five-second deadline.
Every resumed inspection checks owner, connection generation, boot, status, and pending metadata before another effect.
Initial confirmation still refuses busy inspection; queue refusal and poisoned state still stop the batch.
An already-sent close remains sent and cannot replay.
The four regressions pass for Sent, Settling, and Ready phases.
They exercise actual queued requests, recovery, unchanged deadlines, ownership changes, and poisoned locks.
Logs use `/tmp/spaceport-9s44-close-contention-{red,green}.log`.
The combined close-modal, endpoint lifecycle, status-bar, and transfer scope passes 235 cases with none ignored.
`just lint` and `just lint-windows` pass both feature configurations with warnings denied.
Formatting, file size, sorted regions, operations, localization, and specification-package checks pass.
The specification check still reports JSON Schema validation as not run.
Independent review approves the actual ownership checks, deadline behavior, and test evidence.
Hosted qualification remains pending on the repaired source.

## Isolated warning fixture state

Both macOS workspace legs fail the warning-pause reset case with a permanent connection warning.
The peer fixture starts a private failed connection, retires its inbox, and then installs explicit peer state.
It previously retained the startup attempt's last diagnostic in the window projection.
The fixture now initializes that projection from an empty `LiveState` before populating the peer state.
Production notice priority and expiry remain unchanged.
The original case passes locally before this repair; its startup timing did not reproduce on this Mac.
Fresh hosted execution remains necessary to qualify the observed CI failure.

## Windows workspace repair

Source `620a31af7992f5020cf33ef5042132d6647f13fc` completes with seventeen passing checks, five failures, and one skipped target.
All macOS and Linux workspace jobs pass, along with native Windows GUI, release builds, and the installer.
The failures are four Windows workspace jobs and their aggregate gate.
The CodeRabbit success is a draft skip, rather than a review.

Windows x64 default reaches 4,144 passing GPUI cases, 132 failures, and six private fixture entries ignored directly.
Of those failures, 124 stop at the same lifecycle pipe handshake.
The positive owner-rights pipe fixture also reports permission denied.
The lifecycle listener used a default descriptor; the owner-rights fixture omitted its owner field.
[Windows obtains a default object owner from TokenOwner](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-token_owner), which can differ from TokenUser.
A default-owner mismatch fits these failures; the runner's actual default owner SID was not captured.
Both fixtures now declare the current user's owner SID explicitly.
The existing owner-rights DACL, denied-write case, identification-only access, and strict production owner comparison remain unchanged.

Two CLI golden snapshots differ only by CRLF conversion in the captured Windows assertions.
Narrow LF attributes now preserve their exact bytes under an actual `core.autocrlf=true` checkout.
The strict snapshot assertions and command output remain unchanged.

Windows OSC 8 drive paths were parsed as one-letter URI schemes before file classification.
The classifier now recognizes bounded native drive targets first, preserving both separator forms.
Hierarchical `x://` and opaque one-letter schemes still use external-scheme confirmation.
Rich-text links reuse this classifier and retain network/device refusal and location parsing.
Existing file resolution, remote checks, document safety, background I/O, and selection fences remain unchanged.
The missing-file fixture now preserves the original path suffix when constructing its expected diagnostic.
Windows-only regressions cover drive forms, bounds, controls, unknown schemes, locations, and network/device refusal.
They await actual native execution; the original Windows file-click failures provide the failing runtime baseline.

Settings selected a read-only source description on Windows after constructing the catalog row.
Search therefore indexed different text from the rendered row.
The catalog now selects that same message through the existing edit-capability predicate.
The row consumes its catalog description without an override.
The Windows read-only behavior and rendered copy remain unchanged.

The recovery fixture's next-only poll could accept the reservation snapshot before the actual tab was written.
It now joins the fixture writer, checks the saved counter, reloads the Store, and checks the opened identity.
It also joins the reload's idle writer before temporary-directory cleanup.
This strengthens real persistence assertions without changing production storage or increasing a timeout.
The original Windows timeout's scheduling or write-failure cause remains unproven.

The combined local recovery, Settings, terminal/window link, and CLI scope passes 167 cases with none ignored.
Independent reviews approve the security boundaries, shared catalog, and test-writer ownership.
Canonical `just lint` and `just lint-windows` pass both feature configurations with warnings denied.
The Windows-only regressions compile in those checks; they do not execute on this Mac.
Formatting, file size, sorted regions, operations, localization, specification-package, and diff checks pass.
The specification check still reports JSON Schema validation as not run.
Native Windows qualification remains pending on the repaired source.
