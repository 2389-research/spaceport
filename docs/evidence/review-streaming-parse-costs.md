<!-- ABOUTME: Records actual chat preparation work and a synthetic release benchmark. -->
<!-- ABOUTME: Keeps CPU parser measurements separate from pending native UI timing. -->

# Streaming parser costs

Scope: the chat parsing part of `gfde`, authorized by the existing all-findings task.
Root owns this slice.
The initial slice excluded publication; Doctor Biz later authorized the aggregate draft and qualification updates.

## Initial no-marker contract

Unchanged entry revisions already reuse prepared bodies.
Changed rich fragments always run math normalization and display protection before the final CommonMark parser.
The protection pass parses CommonMark to locate literal code, images, and HTML.
Normalization and display protection also create owned text copies.
Without dollar or backslash, neither pass can rewrite the source.

Keep the final parser, options, builder, source coordinates, and rendered semantics unchanged.
Keep existing math processing whenever either marker occurs.
Preserve the message byte, line, word, event, table, and highlight bounds.
Do not throttle updates, delay parsing, retain another transcript, or change transport.

## Acceptance

- [x] Inspect actual preparation, math, transcript, and selection consumers.
- [x] Add a test-only byte meter immediately before the real protection parser.
- [x] Prove ordinary rich Markdown enters that redundant parser before the fix.
- [x] Record the release-mode synthetic streaming baseline.
- [x] Add the small no-marker fast path after the valid red and baseline.
- [x] Require zero protection-parser bytes for ordinary Markdown.
- [x] Run rich, prepared-message, embedding, summary, and selection consumers.
- [x] Measure at least ten percent lower median synthetic preparation time.
- [x] Check that the math-bearing control has no material measured regression.
- [x] Obtain independent semantic review.
- [x] Run the relevant quick checks after the source settles.

## Runtime red

The ordinary `**bold** text` input submits 13 bytes to the protection parser.
The new regression expects zero and fails for that exact reason.
Log: `/tmp/spaceport-9s44-streaming-work-red.log`.
Executable: `/tmp/spaceport-9s44-gpui.z1NSZc/spaceport-tests`.
The actual rich parser accepts the input before the work assertion runs.
After recording that red and release baseline, Root added the no-marker fast path.
The final parser, options, builder, and limits remain unchanged.

## Benchmark protocol

Use only generated text and a fresh in-memory transcript.
Apply the actual Push and Append reducers.
Call Prepared::update with each changed revision and consume the resulting body through black_box.
Append 128 chunks of 128 bytes, ending at 16 KiB plus the optional math prefix.
Keep the final body rich, below the existing word limit.
Measure nine rounds after two warmups.
Report every sample, the median in nanoseconds, and actual protection-parser bytes as JSON.
Measure ordinary text and a math-bearing control.
Run both immutable release executables without concurrent test or Cargo work during timing.

The ignored benchmark is `chat::rich::tests::streaming_work::synthetic_stream_preparation_cost`.
Run it with `--exact --ignored --nocapture` from the pinned release executable.
Build that executable with the shared artifact helper's `--release` option under the Cargo lock.
The helper remains `/tmp/spaceport-9s44-build-gpui-artifact.sh`.
The baseline build log is `/tmp/spaceport-9s44-streaming-release-baseline-build.log`.
The baseline release executable is `/tmp/spaceport-9s44-gpui.39Bv32/spaceport-tests`.
Its compiler artifact records optimization level three and disabled debug assertions.
The initial baseline median is 14.740334 ms for ordinary input and 14.469542 ms for the math control.
Log: `/tmp/spaceport-9s44-streaming-release-baseline.log`.

The optimized release executable is `/tmp/spaceport-9s44-gpui.dylihV/spaceport-tests`.
The new work regression passes there.
Log: `/tmp/spaceport-9s44-streaming-work-green.log`.

An additional sequential old/new pair produced these nine-sample medians:

| Input | Baseline | Optimized | Protection bytes, baseline → optimized |
| --- | ---: | ---: | ---: |
| Ordinary | 14.953500 ms | 12.335750 ms | 1,056,768 → 0 |
| Math control | 14.810250 ms | 14.786125 ms | 1,057,536 → 1,057,536 |

Ordinary preparation improved 17.5 percent in that paired workload.
The math control stayed effectively unchanged.
Logs: `/tmp/spaceport-9s44-streaming-release-{baseline,green}-paired.log`.
Each log retains every sample.
The benchmark asserts that each final body remains rich.
Independent review approved source identity, offsets, literal handling, bounds, and benchmark validity.

## UI verification mode

The first broad chat run used the release executable and encountered UI fixture timeouts.
Root stopped that run after reproducing the same composer timeout in the baseline release executable.
Log: `/tmp/spaceport-9s44-chat-ui-release-baseline.log`.
The pinned GPUI dependency disables debug_selector without debug assertions.
See gpui-pre 0.3.10 `src/elements/div.rs`, lines 908–924.
Those fixtures wait on debug_bounds and cannot qualify in that build mode.
No timeout, assertion, or test ignore was relaxed.
The full debug chat suite now passes 545 cases, with zero failures.
Its one ignored release benchmark was run separately and passed.
Executable: `/tmp/spaceport-9s44-gpui.ZtQ5tx/spaceport-tests`.
Log: `/tmp/spaceport-9s44-streaming-chat-debug-qualified.log`.
The interrupted release UI run is not a passing consumer gate.

## Final quick checks

Formatting, file size, sorted regions, i18n, and diff checks pass.
The i18n check validates 2,174 messages in 114 files.
Host and Windows GNU all-target, all-feature Clippy pass with warnings denied.
Logs: `/tmp/spaceport-9s44-performance-{host,windows}-clippy-final.log`.
The first host lint caught a constant assertion in the ignored benchmark's release guard.
The guard now passes its build flag through black_box and retains runtime refusal in debug.
No production parser or measured workload changed after the qualified runs.
The host retains the existing external `block` future-incompatibility notice.
Native keyboard, glyph, accessibility, and timing checks remain pending.

These measurements concern CPU preparation of synthetic transcripts.
They do not establish native frame latency, glyph behavior, or whole-window performance acceptance.

## Remaining full-prefix work, November 19

On source `f904651ac`, the original-scope audit confirms changed Agent answers enter the final CommonMark parser in full.
The earlier counter observed only math protection; it could not qualify that remaining cost.
The test-only meter now counts protection and final-parser input separately at their actual call sites.
It restores nested scopes and isolates concurrent test threads.
Production builds contain neither counter.

The final-parser measurement regression fails first with zero bytes instead of thirteen.
Log: `/tmp/spaceport-9s44-final-parser-meter-red.log`.
After wiring the final call site, thirty focused rich and summary cases pass, including the real Prepared append consumer.
The existing release-only benchmark remains ignored in this debug run.
The new real Append-to-Prepared workload appends 128 generated chunks.
It verifies the complete displayed prose, including CommonMark's removal of trailing source spaces.
Its 16,398-byte source submits 1,058,560 bytes to the final parser, with zero protection-parser bytes.
Log: `/tmp/spaceport-9s44-final-parser-meter-final.log`.
The first consumer run incorrectly expected the trailing raw space and failed; the corrected display assertion passes.
These byte counts establish remaining work, rather than an optimization or CPU timing result.

The `f904651ac` meter checkpoint receives an independent read-only review with no material finding.
Its host and Windows GNU lints pass with all features and with default features disabled.
Its canonical static checks and diff check pass.
Logs: `/tmp/spaceport-9s44-final-parser-meter-lint.log` and `final-parser-meter-static.log`.

## Validated streaming append preparation

Session now records the last replacement alongside each entry revision.
A snapshot fences all earlier prepared revisions.
Only successfully applied Append events can prove that an existing prefix stays unchanged.
The transcript remains the sole raw-text owner; prepared caches retain bounded source positions only.

Literal appends reuse eligible prose, lists, quotes, tables, and unfinished code.
The parser records whether a final text event reaches its actual input end.
Paragraphs and tables require that proof, including their ordinary or verbatim space behavior.
Hidden reference definitions and ignored table cells cannot extend an earlier visible word.
Code verifies its current raw line and keeps the pinned parser's actual EOF behavior.
An already clipped message reuses its immutable visible body without scanning the growing hidden suffix.

New syntax and unsupported boundaries use the existing full preparation path immediately.
Replacement, snapshot, truncation, and locale resets invalidate reuse.
The existing message, word, event, table, highlight, and visual limits remain authoritative.
No update is throttled, delayed, or rendered with stale formatting.

The real Session-to-Prepared work regression fails first with 1,059,968 parser bytes for 16,409 source bytes.
The same generated rich answer now submits 153 bytes.
Its tests compare every update with fresh full preparation.
They compare complete bodies, selectable text, cells, atomic ranges, and source offsets.
They also retain old bodies to verify immutable ownership.
The full chat scope passes 581 cases, with the existing release-only benchmark ignored.
Log: `/tmp/spaceport-9s44-append-chat-final.log`.

The expanded list and clipped-source regressions fail before their fixes, with 12 and 262,144 unnecessary parser bytes.
The first code attempt incorrectly assumes an EOF newline; pinned pulldown-cmark 0.13.6 preserves the raw EOF.
Raw HTML also preserves spaces that ordinary prose removes.
The corrected mappings pass the full-versus-append boundary checks.
Independent review finds a hidden-reference footer regression before publication.
That exact case fails before parser-owned EOF coverage and passes afterward.
The narrow follow-up review finds no remaining material issue.
Logs: `/tmp/spaceport-9s44-append-container-cut-red.log`, `append-code-eof-diagnostic.log`, `append-reference-footer-red.log`, and `append-parser-eof-green.log`.

## Release preparation measurement

One immutable test executable compares full preparation and validated append using the same real Session workload.
Each case appends 128 generated 128-byte chunks.
Each mode records nine rounds after two warmups.
The compiler artifact confirms optimization level three and disabled debug assertions.
Root runs no concurrent Cargo or test process during timing.

| Source prefix | Full preparation median | Validated append median | Reduction |
| --- | ---: | ---: | ---: |
| Plain | 12.103 ms | 1.211 ms | 90.0% |
| Inline math | 14.762 ms | 1.223 ms | 91.7% |
| Open code | 4.081 ms | 2.694 ms | 34.0% |
| Heading and bold prose | 12.244 ms | 1.240 ms | 89.9% |

Full preparation submits about one MiB to the final parser per generated answer.
Validated append submits only the first 128–153 bytes in these cases.
The math case also reduces protection-parser input from 1,057,536 bytes to 134 bytes.
These are elapsed synthetic release preparation measurements, including test counters.
They do not measure process CPU counters or foreground frame timing.

Artifact: `/tmp/spaceport-9s44-gpui.BOQSp2/spaceport-tests`.
Log: `/tmp/spaceport-9s44-append-release-benchmark.log`.
Receipt: `/tmp/spaceport-9s44-append-release-receipt.json`.
The receipt retains every measured round, compiler profile, executable hash, and qualified source-file hashes.
Current-head platform CI and authored native image qualification remain pending.
Kata `gfde` stays open for its remaining original-scope qualification.

The final streaming implementation passes host and Windows GNU lints with all features and default features disabled.
The canonical formatting, size, sorted-region, operations, localization, commit-message, and diff checks pass.
Logs: `/tmp/spaceport-9s44-append-final-lint.log` and `append-final-static.log`.
The existing debug-link unwind warning and external `block 0.1.6` notice remain unchanged.
