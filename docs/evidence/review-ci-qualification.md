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

## Native Windows path and accessibility results

CI tested source `aaff481f9703bffdb138374eeb9047a7b58e900e` in [run 44446880785](https://github.com/2389-research/spaceport/actions/runs/44446880785).
The final PR status reports sixteen passed checks, six failures, and one skipped target.
The failures comprise four Windows workspace legs, x64 accessibility, and the aggregate gate.
Both macOS workspace legs, all Linux legs, and native macOS host routing pass.
The CodeRabbit success remains a draft skip.
All four Windows workspace legs now reach the GPUI suite with only three failures each.
Default legs report 4,277 passed; all-feature legs report 4,315 passed.
Each reports six private fixture entries ignored directly.
The owner-rights, denied-write, and identification-only pipe cases pass on both architectures and feature configurations.
The Windows drive-classifier, Settings catalog, recovery, and exact CLI snapshot regressions also pass.
The native ARM GUI job and the Windows package/installer job pass.

The three remaining workspace failures compare path spelling rather than file identity or click behavior.
The endpoint click fixture supplied a short Windows temporary path but expected its canonical long spelling.
It now supplies the canonical working directory while keeping the exact final URL assertion.
The two missing-file cases confused a literal path's separators with a file URL's decoded native separators.
Their expectations now distinguish those representations and retain exact diagnostics and hover behavior.
Production file resolution and security checks remain unchanged.
All 66 focused link and file-resolution cases pass locally after these fixture corrections.
Native Windows confirmation of these corrections remains pending.

The x64 native job passes all seven ordinary GUI cases before its new accessibility step fails.
The actual UI Automation control tree omits the version label; all other expected names appear.
The version renderer supplied an accessible name without the role GPUI requires for inclusion.
It now uses `Role::Label` and the localized value, matching existing passive status text.
A regression against the real renderer first fails with no role, then passes after the correction.
It checks the spoken value and absence of Click or Focus actions.
All 13 focused status-bar cases pass locally.
The native assertion requires Text with the version name and forbids Invoke and Value action patterns.
The pinned AccessKit Windows adapter maps Label to Text and derives its name from its value.
The native tree and subsequent keyboard-focus sequence still require a fresh hosted run.

Logs use `/tmp/spaceport-9s44-version-a11y-*` and `windows-link-aliases-tests.log` with the same prefix.
The baseline native logs use `/tmp/spaceport-9s44-aaff-*`.

Canonical host and Windows GNU lint pass both feature configurations after the label and path repairs.
The later Windows-only owner fixture also passes both Windows GNU lint configurations.
The formatting, size, sorting, operations, localization, specification-package, and diff checks pass.
The specification gate still reports JSON Schema validation as not run.
Independent review finds no material issues in either delta.
The default-owner fixture and its explicit native scheduling are described in [pipe qualification](review-windows-safety.md#native-pipe-qualification).

## Drive prefixes and accepted CLI streams

Source `86842ddc3e6b9296ca6f7efacc0318b15f280608` finishes [run 44449866662](https://github.com/2389-research/spaceport/actions/runs/44449866662).
Its PR status has fifteen passed checks, seven failures, and one skipped target.
The failures comprise four Windows workspace legs, macOS default, the x64 release CLI suite, and the aggregate.
Both Windows native jobs and the Windows installer pass.
The x64 UI Automation case now passes the actual version-text and seven keyboard-focus observations.
The version appears as Text with no action patterns.
Screen-reader speech, Core accessibility, and the full Settings keyboard matrix remain separate.

Every Windows workspace leg now has exactly one failed GPUI case: the canonical-directory endpoint click fixture.
Default legs pass 4,281 cases; all-feature legs pass 4,319.
Each retains six private fixture entries ignored directly.
The missing-file diagnostics and all other link cases pass.

The earlier canonical-directory correction produces a Windows verbatim drive path.
The resolver already permits that prefix, but queried it before adding its root separator.
It now defers that incomplete lookup until the next component.
Remote-prefix refusal still runs before this deferral and before every filesystem read.
The exact endpoint URL assertion remains unchanged.
New Windows regressions use an actual canonical document and require remote/device prefixes to return Remote.
The existing native endpoint failure supplies the failing runtime baseline.
Direct execution of the new Windows cases remains unavailable on this Mac and awaits CI.
All 66 affected link and file-resolution cases pass locally.

Both macOS failures report EAGAIN while the scripted CLI peer reads its first request.
A private standard-library probe reproduces inherited nonblocking mode on accepted macOS sockets.
Its initial read returns immediately; explicit blocking mode makes the existing timeout effective.
The peer now clears nonblocking mode before applying its unchanged five-second read timeout.
The listener stays nonblocking for bounded acceptance and duplicate-request detection.
A deterministic actual-descriptor regression first fails on O_NONBLOCK, then passes after the correction.
The first attempted exact test filter selected zero cases; the corrected full-name run supplies the genuine failure.
All 45 compiled CLI cases pass locally, with one opt-in clipboard check ignored.
Command deadlines, exit codes, response envelopes, and no-retry assertions remain unchanged.

Both native default-owner probes report same-owner acceptance with both startup settings.
They do not qualify mismatched-owner refusal.
The fixture now also requests an existing group owner without enabling privileges or creating an account.
Only documented owner/access failures report that assignment unavailable.
Successful assignment must expose a different actual owner and prove readable unchecked access before checked refusal.
The negative qualification still requires its actual native branch markers.
See [the pipe qualification limits](review-windows-safety.md#native-pipe-qualification).

Logs use `/tmp/spaceport-9s44-868-*`, `cli-socket-mode-*`, `verbatim-link-tests.log`, and `prefix-and-peer-*`.
The short names share the `/tmp/spaceport-9s44-` prefix.
The Mac harness built from `86842ddc3` with `--no-run`; no foreground fixture ran.

The final local CLI run also passes all 45 cases with default test parallelism and one opt-in check ignored.
Canonical host and Windows GNU lint pass both feature configurations after these corrections.
Formatting, size, sorting, operations, localization, specification-package, and diff checks pass.
The specification gate still reports JSON Schema validation as not run.
Independent security review confirms unchanged refusal ordering, symlink bounds, document safety, and endpoint fencing.
Kata `jvcf` closes from restored scheduling and actual native execution, independently of these application failures.
Its closure event and scope limits are recorded in [Windows CI evidence](review-windows-ci.md#ci-restoration-closure).

## Green published checkpoint

Source `46738c99c92851245887114dc174d95221cf95b1` passes [CI run 44452942264](https://github.com/2389-research/spaceport/actions/runs/44452942264).
Its PR status reports 22 passed checks and one skipped target.
The CodeRabbit success remains a draft skip, not an independent code review.
The separate supply-chain workflow also passes.

All Windows workspace legs pass, including the exact canonical-directory endpoint URL assertion.
Both all-feature GPUI suites report 4,322 passed, zero failed, and six private fixtures ignored directly.
Both native GUI jobs pass; x64 runs seven cases and ARM runs six.
The x64 UI Automation case passes the version-text tree and seven keyboard-focus observations.
Both native owner probes prove readable mismatched-owner refusal with automatic startup enabled and disabled.
The Windows package job passes actual CLI and installer smoke checks.
The macOS CLI failures no longer occur.

These results qualify the published checkpoint only.
The consent-context follow-up below requires a new hosted run.
Foreground Mac Settings, screen-reader speech, and interactive release profiling remain pending.

## Native workspace consent follow-up

The close dialog's title, targets, pane count, and interruption text lacked accessible roles.
Its inline error also lacked an Alert role.
The real text producers now expose active labels and a named, assertive error alert.
Departing captures expose no accessibility role.
Unit regressions verify names, values, and absence of Click or Focus actions.
They do not prove native speech or execution of the live-region callback.

The context regression first fails with no label role.
The inline-error regression separately fails with no Alert role.
The corrected close-dialog and departure scopes pass 105 tests, with one existing private fixture ignored directly.
The PowerShell helper retains its exact child and bounds command waits to 20 seconds.
It caps capture reads at 1 MiB; the capture file can grow during the bounded command.
The new timeout test checks actual child reaping.
Named-button invocation searches only the owned GUI's windows.
Keyboard delivery requires the exact foreground window and focused process.

The standalone Windows scenario creates a private workspace and reads its actual consent context.
It cancels once, then reopens and confirms with fresh keyboard input.
It checks the original workspace, checkout marker, and owned daemon afterward.
Actions run once; readiness polling only observes.
The existing x64 native workflow selects every ignored accessibility scenario without a name filter.
Neither this new scenario nor the Windows timeout test has run on this Mac.
Their runtime results remain pending CI.

Logs: `/tmp/spaceport-9s44-close-context-red.log`, `close-error-context-red.log`, and `close-context-final.log`.
Short log names share the `/tmp/spaceport-9s44-` prefix.
Independent review finds no material issue in the text, error, native targeting, or exact-child cleanup changes.
Canonical `just lint` and `just lint-windows` pass both feature configurations.
Both scheduling tests pass against the actual workflow command.
Formatting, file-size, sorted-region, operations, localization, specification, and diff checks pass.
The specification gate explicitly reports JSON Schema validation as not run.
Logs use `/tmp/spaceport-9s44-consent-{host-lint,windows-lint,ci-scheduling,static}.log`.

## First native consent run

Source `1ef9582317a8d9a561034628f5c7a48c7d0e4380` reaches actual Windows UI Automation in [run 44458748028](https://github.com/2389-research/spaceport/actions/runs/44458748028).
The x64 job passes all seven ordinary native GUI cases and its pipe-owner probe.
Both UI Automation scenarios then time out before returning their first tree.
Each command reaches its 20-second deadline and the owned child is reaped.
Neither the consent scenario nor the existing tree/focus scenario passes on this source.
The cause of the stall remains unproven.

The next diagnostic retains at most 1 MiB of partial output after timeout cleanup.
The script reports assembly loading, native-helper compilation, window discovery, and tree completion stages.
The deadline and keyboard ownership guards remain unchanged.
The diagnostic does not retry actions or claim the timeout is fixed.
Log: `/tmp/spaceport-9s44-consent-native-x64.log`.

## Empty-capture diagnostic

Source `260250da5` repeats both native UIA timeouts with zero captured bytes.
No assembly-loading stage marker appears.
The stall therefore precedes those markers; its cause remains unproven.
The first marker now uses direct Console output before the encoding assignment.
A separate plain-script probe uses the same cleared environment and private capture before any GUI starts.
It has a 60-second diagnostic bound and reports actual startup duration.
The existing UIA command deadline stays at 20 seconds.
This distinguishes host startup from encoding setup and later UIA calls.
The Windows workspace suite also runs that plain-script test without a desktop or daemon.
Native execution of the new probe remains pending.
The prior 100-millisecond timeout test did not establish that its script body began executing.

Microsoft documents [PowerShell's startup phases](https://learn.microsoft.com/en-us/powershell/scripting/dev-cross-plat/performance/startup-performance).
The [module-path documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_psmodulepath?view=powershell-5.1) explains reconstruction when the process variable is absent.
These references guide diagnosis; they do not establish this runner's cause.
Log: `/tmp/spaceport-9s44-motion-native-x64.log`.

## Startup ruled out

Source `2520ac72c` executes the plain private script in 180 and 167 milliseconds on native Windows x64.
The [native job](https://github.com/2389-research/spaceport/actions/runs/44461941941/job/135350958417) then fails both UIA scenarios at their unchanged 20-second limit.
Each capture contains only the direct script-entry marker.
PowerShell startup, the private capture file, and basic Console output therefore work.
The next boundary contains encoding setup and the first cmdlet call.
The cause within that boundary remains unproven.

The next trace adds a direct marker after encoding setup.
Two separate Windows process tests exercise UTF-8 snowman output and `Write-Output` loading.
They use the same private environment and capture helper.
Their native execution remains pending.
No environment variable or existing UIA deadline changes.
Log: `/tmp/spaceport-9s44-startup-native-x64.log`.

## Isolated cmdlet failure

Source `134f912e3` passes the plain-script and UTF-8 process probes on Windows x64 and ARM.
The independent `Write-Output` probe times out on both architectures.
Its capture contains only `CMDLET START`.
The failing operation therefore does not require a GUI, UI Automation, or encoding mutation.
Both all-feature GPUI unit suites pass 4,328 cases before this separate process-test failure.
The native x64 trace also reaches its encoding-complete marker before the same first-cmdlet stall.

Verified [x64 process logs](https://github.com/2389-research/spaceport/actions/runs/44463521378/job/135356144268)
and [ARM process logs](https://github.com/2389-research/spaceport/actions/runs/44463521378/job/135356144186).
Logs use `/tmp/spaceport-9s44-shell-operations-{x64-all,arm-all,native-x64}.log`.

The PowerShell command now restores existing `ProgramFiles` and `ProgramFiles(x86)` values from its parent.
Microsoft documents their role in [module-path construction](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_psmodulepath?view=powershell-5.1).
A maintainer reports their omission breaking Windows PowerShell in [PowerShell issue 3545](https://github.com/PowerShell/PowerShell/issues/3545).
This is a controlled correction for the module-loading hypothesis, pending native results.
The shared sandbox, private profile paths, system PATH, deadlines, and child cleanup remain unchanged.

Both Windows GNU lint configurations and static validators pass with this correction.
Independent review finds no material isolation or ownership issue.
The actual cmdlet probe and both UIA scenarios still need successful native execution.
Logs: `/tmp/spaceport-9s44-powershell-system-paths-{windows-lint,static}.log`.

## Explicit system module bootstrap

The system-path correction on `210e5a653` is insufficient.
The x64 default process suite passes its cmdlet probe after about 17 seconds.
Both ARM process suites still time out at `Write-Output`.
The x64 UIA scenarios also retain the first-cmdlet timeout.
Logs: `/tmp/spaceport-9s44-system-paths-{x64-default,arm-all,arm-default,native-x64}.log`.

The next correction loads Utility and Management explicitly from a shared private bootstrap.
The module-qualified Core loader avoids automatic command discovery for `Import-Module` itself.
Both the UIA script and the existing unqualified `Write-Output` regression load that same file.
The regression also requires the completed-imports marker.
Plain startup and UTF-8 probes retain their independent paths.
Diagnostic stage messages use direct Console output.
Native tree/focus output, assertions, environment isolation, deadlines, and cleanup remain unchanged.

Microsoft documents [module-qualified commands](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_command_precedence)
and [script imports](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/import-module?view=powershell-5.1).
By-name imports still use `PSModulePath`; this is not path-pinned loading.
Independent review finds no material scope, file-path, lifetime, or test-fidelity issue.
Native execution must establish whether this correction resolves the stall.
Both Windows GNU lint configurations and static validators pass.
The specification gate explicitly skips JSON Schema validation.
Logs use `/tmp/spaceport-9s44-powershell-modules-final-{windows-lint,static}.log`.

## Native bootstrap results and sidebar retention

[Run 44468640390](https://github.com/2389-research/spaceport/actions/runs/44468640390) tests `09b5ae61b`.
The explicit module bootstrap completes on Windows x64.
The unqualified cmdlet and owned-child timeout regressions pass in both Windows all-feature process suites.
Each architecture also passes 4,328 application cases, with six private fixtures ignored directly.
The native chrome tree and seven keyboard-focus moves pass.
Both UIA scenarios reach completed tree captures without the former cmdlet timeout.

The workspace-consent scenario still fails before opening consent.
Its private window exposes the terminal and status bar but omits the sidebar subtree.
The window title contains the expected original workspace name.
This is not proof of workspace consent or screen-reader speech.
The capture is `/tmp/spaceport-9s44-modules-native-x64.log`.

The pinned GPUI 0.3.10 source clears accessibility nodes on each active frame.
Its cached view replay retains paint and input state but omits accessibility nodes and actions.
Spaceport always caches its sidebar, so later frames can drop its accessible subtree.
The repair bypasses this cache while accessibility is active.
The real-view regression first fails when a second accessible frame reuses the cache.
It then passes with the uncached path, alongside the existing ordinary-redraw cache regression.
This headless result proves rendering behavior; it does not prove native AccessKit output.
The unchanged native workspace scenario must qualify the repair in CI.

## Close-sequence fixture contention

The same run fails the Linux ARM all-feature close-sequence fixture.
It reports 4,545 passing cases, one failure, and 19 ignored fixtures.
The failed peer read returns `WouldBlock` after its existing three-second bound.
The log does not identify which receive call timed out.

Source review finds a deterministic race after both surface-barrier prerequisites arrive.
The fixture advances once, then blocks waiting for the second close.
If the inbox is busy, that advance defers work until the next tick.
Production supplies subsequent ticks; the fixture has stopped supplying them.

The regression holds the real inbox guard during that final advance.
It reproduces the same receive timeout on macOS without changing the socket deadline.
The repair uses the existing bounded polling helper to resume advancement after releasing the guard.
Both barrier arrival orders and the exact second-tab assertion pass.
Production command timing, ordering, and retry behavior remain unchanged.
Baseline and red logs use `/tmp/spaceport-9s44-close-sequence-{baseline,contention-red}.log`.

The combined focused suite passes four cases with no failures or ignored cases.
It includes both sidebar policies, shared-indicator invalidation, and the close-sequence fixture.
Logs: `/tmp/spaceport-9s44-sidebar-accessibility-red.log` and `/tmp/spaceport-9s44-sidebar-close-green.log`.
Formatting, file size, sorted regions, operations, localization, and diff checks pass.
Both host and Windows GNU lint configurations pass through `just lint lint-windows`.
Independent review finds no remaining issue after correcting the use of GPUI's public cache API.
Lint and static logs use `/tmp/spaceport-9s44-sidebar-close-{lint,static}.log`.
Native Windows rerun, Mac foreground checks, speech, and interactive profiling remain pending.

## Native sidebar and close-sequence qualification

[Run 44471538211](https://github.com/2389-research/spaceport/actions/runs/44471538211) tests source `96b5295d5`.
Both Windows native GUI jobs pass.
The x64 job passes both actual UI Automation scenarios, including workspace consent.
They run without ignored cases and retain exact foreground-window guards.
The workspace case cancels, then confirms fresh consent and preserves unrelated work.
Linux ARM all-feature CI passes 4,547 application cases, with 19 existing private fixtures ignored directly.
It includes the real-view cache regression and both controlled close-sequence arrival orders.
Logs use `/tmp/spaceport-9s44-sidebar-{native-x64,linux-arm-all}.log`.
The full run passes: 18 jobs succeed and one unconfigured target is skipped.
The separate supply-chain run also passes.
Mac foreground checks, speech, and interactive profiling remain pending.

## Reader-closure fixture isolation

Run `44474270771` tests `17cc0d2ab` and passes all four native Windows UIA cases.
Its Linux ARM all-feature job fails the immediate BrokenPipe assertion in the process-output fixture.
The assertion does not record the actual unexpected write result or identify another reader.
The focused local client scope passes the same assertion before its fixture changes.

`Capture` owns its reader in one worker and joins that worker on drop.
Rust documents that [join waits for thread completion](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.join).
The pipe assertion also requires every other reader copy to be closed.
Linux documents [pipe EOF and BrokenPipe behavior](https://man7.org/linux/man-pages/man7/pipe.7.html)
and [close-on-exec behavior](https://man7.org/linux/man-pages/man2/open.2.html).
A concurrent fork can retain a reader before exec closes it.
This makes the fixture's global assertion stronger than its owned-reader contract.

A coordinated local probe retains a CLOEXEC reader in an exact child before exec.
Closing the parent's reader still permits a six-byte write.
BrokenPipe appears after the retained child exits and is reaped.
This proves the competing inheritance path; the exact CI inheriting process remains unknown.

The fixture now creates its pipe after exec in one exact owned test subprocess.
It preserves the real BrokenPipe assertion, without write retries or a relaxed cleanup deadline.
The child reports the actual write result on failure.
After success, it emits a receipt that the parent requires, so an empty test filter cannot pass silently.
The parent bounds the child wait and captured diagnostics and reaps that exact child on every path.
It reuses the shared sandbox and its cleared private environment.
The test import uses the existing performance harness's scoped expectations for unused daemon helpers and their assertions.
No production cleanup code or lint policy changes.
The closure check lives in `crates/sheprd-client/tests/process_output.rs` beside the crate's other integration targets.
This lets the shared sandbox use the client as an external crate on Windows.
The final integration target passes three cases; its one private child entry is ignored directly and invoked by the parent.
Log: `/tmp/spaceport-9s44-output-integration-final.log`.
The final client library scope passes 33 cases without ignored cases.
All four host and Windows GNU lint configurations pass.
The tiny receipt addition also passes focused host and Windows target lint.
The existing caught sandbox-unwind probe prints its expected panic under `--nocapture` and passes.
Logs use `/tmp/spaceport-9s44-copy-pipe-final-{client,lint,static}.log` and `/tmp/spaceport-9s44-output-receipt-{host,windows}-lint.log`.
Source `363a4480d` passes [the complete CI run](https://github.com/2389-research/spaceport/actions/runs/44478003624).
All 18 executed jobs pass; one unconfigured target is skipped.
The separate supply-chain run also passes.
Linux ARM all-feature CI executes the moved integration target: three cases pass and the parent invokes its private child.
Both native Windows jobs pass, including all four x64 UIA scenarios without ignored cases.
Logs use `/tmp/spaceport-9s44-copy-pipe-{linux-arm-all,native-x64}.log`.
Mac foreground checks, speech, and the release working-session profile remain pending.

## Windows pipe fixture startup on `7607bde87`

The [Windows all-feature job](https://github.com/2389-research/spaceport/actions/runs/44537828031/job/135608225409) fails three SSH anonymous-pipe fixtures.
Each failure records zero bytes before its unchanged 30-second startup bound.
They fail before their actual handshake, cancellation, and banner-limit assertions.
This establishes a fixture startup failure, without proving why PowerShell stalled or a production bridge regression.
The same source passes ARM all-feature checks.

The replacement launches an exact ignored child of the compiled test binary.
Its cleared environment retains only the selected test case and required Windows system paths.
The child flushes a startup sentinel, then waits for a parent acknowledgment before emitting protocol traffic.
The parent bounds libtest's preamble to 16 KiB.
Successful children flush and exit directly, so libtest cannot append bytes to the exact EOF check.

The existing three native cases retain their real Windows anonymous pipes and original assertions.
They still cover the compatible handshake, exact echo and EOF, idle timeout, cancellation, and oversized banner.
The existing `cmd.exe` stderr/exit classification case remains unchanged.
The fixture still uses the existing exact-child kill/wait guard.
The one added ignored entry is a child endpoint invoked by its enabled parent tests.
No existing test is ignored, retried, or given a longer deadline.

Independent source review finds no material framing, ownership, or scope issue.
Both Windows GNU lint configurations pass.
Logs: `/tmp/spaceport-9s44-ssh-self-child-windows-lint.log` and `ssh-self-child-minimal-windows-lint.log` under the same prefix.
Actual Windows execution remains pending; this Mac cannot run those platform-specific cases.

## Native startup and refusal consumers on `b5a55f455`

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44638908188/job/135964929274) fails before UI Automation starts.
The initial GUI connection reports Windows error 231 after 178 milliseconds.
Seven other GUI cases pass; the ARM native job passes its configured cases.
The image cleanup rename and producer are not exercised by this run.

The shared Windows readiness probe opens and drops a real client connection.
Locked interprocess 2.4.7 documents that this can leave a dead connection awaiting server acceptance.
That interference is proven by the library contract; its sole responsibility for error 231 remains unproven.
The fixture now uses [non-consuming availability inspection](https://learn.microsoft.com/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew).
The outer deadline, child-liveness check, and 20-millisecond polling interval remain unchanged.
Production connection behavior remains unchanged.
The dependency's connector already waits for busy instances; it is not a single-attempt connector.

Two Windows regressions use a real owned pipe and require no desktop or daemon.
Repeated readiness checks must leave its first instance available for the real client's byte exchange.
A missing pipe must remain unavailable.
Their Windows execution and old-probe baseline remain unrun on this Mac.
Cross-compilation does not establish that the startup failure is resolved.
Independent review finds no material unsafe-call, ownership, deadline, or dependency issue.
Both canonical Windows GNU lint configurations pass after the readiness change.
Log: `/tmp/spaceport-9s44-pipe-readiness-windows-lints.log`.

The [Linux all-feature job](https://github.com/2389-research/spaceport/actions/runs/44638908188/job/135964929602) also exposes two missed refusal-copy consumers.
Default Windows and both Linux architectures report the same two failures.
The handlers return the corrected message; the tests still expect the old wording.
Both tests now compare against the typed cause's localized message.
They still require the failure code, repeated refusal, unchanged selection, and matching status notice.
The host-control and localized-error scopes pass all 75 cases, with none ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- control::server::hosts::tests:: i18n::errors::tests::`.
Log: `/tmp/spaceport-9s44-host-refusal-consumers-green.log`.
The existing debug-link unwind and external `block` notices remain unchanged.
Final GPUI all-target/all-feature Windows GNU lint also passes after correcting those consumers.
Log: `/tmp/spaceport-9s44-readiness-hosts-final-windows-lint.log`.

## Split Core scheduling contract on `daab7134e`

Both default Linux jobs fail the release-script scheduling check on `daab7134e`.
The test treats YAML's block indicator as the shell command and attempts to execute `|`.
It also predates the separate Core qualification step.
The same failure reproduces locally before the correction.

The test now executes both actual shell commands through its existing argument recorder.
Ordinary UIA may skip only `core::`; the separate step must select that entire namespace.
Both steps retain ignored-test selection, serialized execution, and the accessibility harness.
The Core condition runs only on x64 and refuses cancelled runs.
The shared restricted condition evaluator supports that explicit status function and negation.
Unknown expressions still fail rather than executing workflow text.

The two accessibility scheduling cases, 11 policy cases, and release-transition cases pass locally.
Logs: `/tmp/spaceport-9s44-core-ci-{contract-red,contract-green,policy-green,transition-green}.log`.
The full `just test-release-scripts` run executes 396 cases with 20 skipped.
It fails four cases and errors once in the unchanged macOS Sparkle cleanup checks.
The executable census rejects one observed row whose UID is nonnumeric.
No process names, arguments, or private paths are captured in that diagnosis.
Those failures remain visible in `/tmp/spaceport-9s44-core-ci-release-suite.log`; cleanup controls remain intact.
The full suite is not claimed passing.
