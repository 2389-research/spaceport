# Ask Spaceport native implementation evidence

Verified November 13, 2026, on macOS with Rust 1.99.0.
This note covers native presentation and controller checks.
The helper and observation broker have separate qualification evidence.

## Ownership

One app-global controller owns the attach link and helper worker.
Each native window owns its draft, focus, selection, and reading position.
Opening an idle page does not start a helper or submit a prompt.
Only explicit submission and confirmed interrupted recovery request helper startup.
The controller never calls `Session::resend`.

The worker performs process and checkpoint operations outside the UI thread.
It writes durable Stop intent before processing a later Quit command.
New Chat retains the previous conversation directory.
Checkpoint reads reject oversized, malformed, and mismatched state.
Helper startup errors retain bounded stderr and the actual exit status.

The view reuses the native chat preparation, selection, and list contracts.
The transcript uses virtualized rows and revision-based prepared bodies.
Source buttons resolve opaque targets from actual observation replies.
Answer citations resolve only evidence from the same user turn.
Source captions include the observation date and local time.
Expired source targets report unavailable instead of matching similar labels.

## Regression evidence

The running New Chat confirmation test failed before its implementation.
The Ask activity-panel test failed because the variant did not exist.
The configuration tests failed because Ask agent preferences did not exist.
The queue-loss test failed with the original draft-clearing behavior.
Its failure compared a missing draft with the exact submitted Unicode text.
The fixed queue-loss test passes.

Not every added test has separate baseline failure evidence.
The tests also cover boundaries discovered during implementation and review.

## Commands run

All Cargo commands used `--locked`, disabled incremental compilation, and cleared `HERDR_BUILD_PR_NUMBER`.
Initial implementation checks accidentally used a separate target cache.
Final status filters used the parent's current shared GPUI test binary.

| Command | Result |
| --- | --- |
| `cargo +1.99.0 test --locked -p sheprd-control-protocol ask_and_overview_are_separate_activity_panels --lib` | 1 passed |
| `cargo +1.99.0 test --locked -p sheprd-gpui ask --bin spaceport` | 69 passed before final presentation refinements |
| `cargo +1.99.0 test --locked -p sheprd-gpui --features integration-test ask:: --bin spaceport` | 23 passed after shared autoscroll integration |
| `cargo +1.99.0 test --locked -p sheprd-gpui --features integration-test chat::tests::selection --bin spaceport` | 7 passed |
| `cargo +1.99.0 test --locked -p sheprd-gpui --features integration-test chat::tests::prepared --bin spaceport` | 9 passed |
| `cargo +1.99.0 test --locked -p sheprd-gpui --features integration-test chat::tests::glide --bin spaceport` | 5 passed |
| `cargo +1.99.0 test --locked -p sheprd-gpui --features integration-test cli::tests::ui_panel --bin spaceport` | 2 passed |
| `just check-file-size` | Passed |
| `just check-sorted` | Passed |
| `just check-i18n` | Passed after replacing the Settings agent label with its locale key |
| `git diff --check` | Passed |

The macOS linker reports its existing large unwind-table warning.
Cargo also reports the existing `block` future-compatibility warning.
These checks introduced no additional compiler warnings.

## Remaining qualification

The real native Ask fixture passed both themes at 480 and 960 pixels.
It captured first use and a saved conversation with long text.
It verified native typing, shifted newlines, selected-draft deletion, and hidden-terminal keyboard isolation.
Eight PNGs live under ignored `target/ask-spaceport/native-captures/`.
Independent review inspected all eight captures and found no material layout regression.

Review found that unconditional key consumption blocked native text input.
The corrected page consumes reader keys only while the reader owns focus.
Plain Enter sends, shifted Enter inserts a newline, and other modifiers preserve the draft.
Regression tests verify Unicode input through the actual native field handler.
Secondary text and source links use the retained 4.5:1 contrast adjustment.
The contrast regression covers four existing themes.

Those initial checks did not use agent credentials, a real model turn, or a personal daemon.
Doctor Biz later approved isolated real-provider qualification using Pi's existing configuration.
The results below record that separate qualification.
Full workspace CI and platform CI remain separate gates.

Visible excerpt selection, Copy, Select All, and native composition use shared contracts.
Ask and pane chat share one bounded pointer-drag autoscroll policy.
The stationary-drag test first failed because selection never reached unmounted rows.
It now selects beyond the viewport using explicit frame times and real text geometry.
Additional tests verify timer scrolling, mouse-up, deactivation, window close, and history replacement.
History replacement also rejects stale selection geometry.
The compiled test binary reran shared selection and glide filters after the extraction.
Those filters passed seven and five tests respectively.
The Ask reader does not execute or open project-file links from model output.

### Source callback contract

A headless integration regression clicks the actual Ask source chip using broker-issued fixture evidence.
It initially returned `WindowClosed` because the control handler borrowed the window during its own input event.
The callback now defers navigation until that event releases the window.
The same regression passes and reaches the correct readiness result.
Busy sources show retry guidance; stale sources keep the dated-answer explanation.
The final observation filter passes 37 tests, with two existing tests ignored.
This verifies the callback boundary without a provider call.
It does not replace the remaining native exact-pane navigation and Back acceptance check.

### Final fixed-build attempt

The final serialized run used the corrected settlement bridge and deferred source callback.
Navigation preflight and native input passed before its single question.
Pi completed the real session-list and terminal-read tools against the private Herdr fixture.
Its list reply contained the corrected connection and runtime coverage alternatives.
The answer did not finish before the bounded wait expired.
The captured request ended interrupted after graceful cancellation, and helper ownership released.
The saved overall native verdict remains false.
The capture does not establish why the answer stalled.
No provider or authentication setting changed, and no further provider request followed.

Exact native source-click and Back remain the unresolved live acceptance steps.
Manual OS IME behavior and full workspace/platform CI remain unrun.

### Optional feature settings

The Ask feature now defaults off and changes its enablement only after restart.
The agent picker remains available separately and applies to the next New Chat.
The headless Settings test clicks the actual switch and Pi segment, then checks independent saved choices.
The native Settings fixture saves and reloads both preferences through the real private configuration files.
It verifies that pending enablement never changes the running window or starts an Ask controller.
The fixture passed in both themes at 680 and 960 pixels without any provider request.
Native captures live under `target/ask-spaceport/settings-native/ask-*.png`.
The narrow pending and wide off captures show both controls without clipped labels.
The existing native Ask presentation fixture also passed after its explicit test-only opt-in.
The focused Ask filter passed 104 tests, with one existing opt-in test ignored.
Strict Clippy, formatting, source size, sorting, and localization checks passed.
Logs are `target/ask-spaceport/enable-{ask-final,native-settings,native-ask,clippy,static}.log`.

## Read-only Ask status inspection

`spaceport status --json` includes optional app-global Ask controller facts.
The getter reads the existing controller without creating one or starting a helper.
It bounds displayed diagnostics to 512 characters and preserves the existing status fields.
The CLI keeps its existing JSON and exact-window options.

The protocol regression first failed because serialization dropped the supplied Ask facts.
The failure compared JSON null with the expected active-answer boolean.
Final shared-binary filters passed seven tests covering producer, getter, parser, output, and help.
Final control-protocol checks passed all 69 unit tests and four observation socket tests.
The final combined Ask filter passed 33 tests, including the opted-in owned-process shutdown test.
The final strict Clippy and canonical formatting, size, sorting, and localization checks passed.

## Real native provider qualification

Doctor Biz approved real Pi/provider tests after the explicit sign-in request.
The opt-in fixture uses the actual Ask controller, Worker, helper, installed Pi, and private Herdr daemon.
The daemon contains only synthetic terminal output.
The app uses a private HOME and the existing `PI_CODING_AGENT_DIR` reference.
The fixture copies no credentials and changes no provider or model setting.

The native fixture checks typing, Shift+Enter, and the actual field's composition handler.
Run 3 proves plain Enter submission; it does not prove clicking the Send button.
It checks marked CJK text, emoji, UTF-16 bounds, and Enter suppression while composing.
These checks do not establish manual OS IME behavior.
It also checks that Ask input never reaches the hidden terminal.

| Run | Actual result | Remaining failure |
| --- | --- | --- |
| 1 | No conversation or provider question started. | Native Pi button geometry missed the painted button. |
| 2 | One native question ran both observation tools and read the synthetic marker. | Provider reported `scheduler admission timeout`; no final answer appeared. |
| 3 | One native question completed with the exact random marker and opaque source citation. | The fixture coalesced page switching with a deferred tab action; source clicking never ran. |
| 4 | Staged page/tab navigation passed before one native question. | Provider returned HTTP 429 `CONCURRENT_REQUEST_LIMIT_EXCEEDED`; no tools or answer ran. |

Run 3 completed request `1cb4342d-7498-48bb-bb71-a6c3b0d4e379`.
Its real answer quoted the random terminal result, which the question did not contain.
It cited target `7326bb7f-af02-483a-aa22-55aa107930b1` from the actual terminal observation.
The native answer screenshot is `target/ask-spaceport/native-provider-3/ask-provider-answer.png`.

The fixture now anchors its geometry recorder to the actual padded button.
It also crosses separate updates between page switches and deferred tab actions.
Run 4 passed that navigation preflight before provider submission.
The source-click-and-Back gate remains unverified against a real provider answer.
Provider retries stopped after the configured organization's concurrency rejection.

Review then found a restricted bridge settlement bug in those captured app bytes.
Pi can emit `agent_end` before its retry sequence reaches `agent_settled`.
The old bridge could publish Failed or Idle while Pi still retried.
Runs 2 and 4 recorded Failed checkpoints, but those records do not prove final retry settlement.
Their captured provider errors remain real; their overall fixtures did not pass.
Root owns separate deterministic qualification for the bridge correction.
No additional provider run occurred here.

Runs 2 and 3 overlapped after a yielded tool response was mistaken for completion.
An early claim that run 2 had not reached the provider was incorrect.
Its checkpoint proves both real observation tools ran before the provider admission failure.
No further run started while another fixture remained active.

Each run has a private `ask-provider-audit.json` under its numbered capture directory.
The audits retain conversation, helper-run, and request IDs plus authoritative checkpoint outcomes.
Run 1 submitted zero native questions; runs 2, 3, and 4 each submitted one.
Provider HTTP request counts were not instrumented and remain unknown.
One native question may include tool-loop requests or Pi-owned retries.
Runs 2, 3, and 4 all released the helper owner lock during exact-owned-child cleanup.
Run 4 uses its own console log, `target/ask-spaceport/native-provider-4.log`.
The earlier shared console log contains interleaved partial history; per-run artifacts are authoritative.

The fixture now reports a failed final turn before checking for missing terminal evidence.
Its diagnostic tests first failed twice because the old path returned success for failed turns.
All three tests now pass, including successful completion after an earlier recovered error.
The exact filter was `smoke::ask_provider::tests` with the `integration-test` feature.
Logs are `target/ask-spaceport/native-provider-report-{red,green}.log`.
Scoped formatting, source-size, sorting, and diff checks also passed after this correction.

Independent review found another fixture false-pass path after partial evidence and cancellation.
The final turn check now requires `TurnEnd::Done`.
A missing summary stays pending; Cancelled, Limit, Refused, Other, and Failed cannot pass.
The cancellation and missing-summary regressions first failed against the previous predicate.
The final native verdict filter passes all five tests.

The saved artifact now separates `native_view_pass` from `native_provider_pass`.
The final harness verdict starts false and becomes true only after all assertions pass.
Those assertions include helper ownership release, a Completed checkpoint, and private daemon survival.
The artifact regression first failed because a GUI PASS marker produced overall success.
The final artifact and opt-in guard filter passes two tests; the paid test remains ignored.
These checks made no provider calls.
Logs are `native-provider-verdict-{red,green}.log` and `native-provider-artifact-{red,green}.log` under `target/ask-spaceport/`.
