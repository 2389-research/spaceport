<!-- ABOUTME: Records actual chat preparation work and a synthetic release benchmark. -->
<!-- ABOUTME: Keeps CPU parser measurements separate from pending native UI timing. -->

# Streaming parser costs

Scope: the chat parsing part of `gfde`, authorized by the existing all-findings task.
Root owns this slice.
No commit, publication, or native qualification is authorized by this work.

## Existing contract

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
