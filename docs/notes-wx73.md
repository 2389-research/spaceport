# wx73: the phone's upload staging script and search pacing

Branch `feat/wx73-mobile-shared-4`, cut from `cf94fd9856` (main after #237 gave
the phone the pane map and the navigation predicates). Two re-coded rules get
one home in `spaceport-client`: the shell that stages a private file on the
host, and the content revision a scrollback search may name. No UniFFI
signature changes — `MobileError`, `StagedImage`, `HistoryMatch` and
`HistoryCursor` are untouched, so the generated Swift and Kotlin are the same.

## 1. The staging script

`mobile/rust/src/ssh/images.rs` built its own `herdr-upload` script, and
`crates/spaceport-client/src/upload.rs` built another. Comparing them line by
line, three things matched (private umask, a twelve-character `mktemp`
directory, a byte count that must match), and four did not:

| Difference | Desktop | Phone (before) | Now |
| --- | --- | --- | --- |
| Staging base | `${TMPDIR:-/tmp}` resolved with `cd -P`, so macOS prints `/private/tmp` | hard-coded `/tmp`, and the reported path was validated against that prefix | both resolve the host's own temporary directory; the phone accepts what the host prints |
| Destination | exclusive create (`set -C`, `exec 3>`) after an `accept` line | plain `> "$file"` | both create exclusively on descriptor 3 |
| Byte-count guard | `bytes=$(wc -c < "$file" \| tr -d '[:space:]')` | `[ "$(wc -c < "$file")" -eq N ]` under `set -e` | one guard, `|| exit 1`, no `set -e` |
| Retention | file persists; removal is a separate `pane`-less cleanup script | `retain`/`retained`/`discard` handshake on the same channel | unchanged per client, sharing the removal |

The last one is a real protocol difference, not drift: the desktop half-closes
its socket pair to end `cat`, so it can no longer answer on that channel and
removes files later with `remove_uploaded_files`. The phone must keep its
channel open to say `discard` when Swift decides the paste will not happen, so
it reads exactly `N` bytes with `head -c` and keeps writing. Both handshakes
stay; only the rules inside them are shared.

`crates/spaceport-client/src/staging.rs` (188 lines, `pub mod staging`) now
holds:

| Item | Rule |
| --- | --- |
| `directory()` (private) | `umask 077`, `cleanup()`, the exit trap that runs it unless `$complete`, `${TMPDIR:-/tmp}` resolved through symlinks, `mktemp -d` of `herdr-upload.XXXXXXXXXXXX` |
| `create_exclusively(name)` (private) | `set -C`, `exec 3> "$destination"`, and `file=` only once the host's own create succeeded |
| `verify_size(len)` (private) | the byte count guard both scripts use |
| `receive_script(name, len)` | the desktop's ready/accept/created/complete script |
| `retain_script(name, len)` | the phone's path/retain/retained/discard script |
| `READY`, `CREATED`, `COMPLETE`, `ACCEPT`, `RETAIN`, `RETAINED`, `DISCARD` | the one-line directives in both directions, so a script and its parser cannot drift |
| `valid_directory(dir)`, `staged_directory_of(path)`, `is_staged_file(path, name)` | the shape a reported path may have: absolute, bounded, no control characters, no empty `.` or `..` component, a staging directory as the last directory, a plain file name |
| `LINE_LIMIT` | the bound on both a path and a directive line (upload's own copy is gone) |

`upload.rs` loses `valid_directory` and `uploaded_directory` (they were that
file's, now the module's) and its `receive_command` is one call into
`receive_script`. The POSIX I/O — spawning, polling, progress, rollback — is
untouched. `images.rs` keeps its russh channel, its three timeouts and its
`StagedImage` keep/discard oneshot; it calls `retain_script` and
`is_staged_file`, and reads the directives from the shared constants.

One behaviour change follows from sharing the base: the phone no longer refuses
a path spelled `/private/tmp/...`. That spelling is what a real macOS host
prints, so the old prefix check was rejecting the correct answer on exactly the
platform the phone runs on.

## 2. The search revision rule

Three desktop sites spelled "an odd content revision is a write in progress,
which the daemon refuses" — `find.rs`, `copy_mode.rs` (`% 2`, as
`is_multiple_of(2)`) and `links.rs` (`% 2 != 0`) — and the phone had no such
rule at all: it sent the search and relied on catching `stale_content`
afterwards. `crates/spaceport-client/src/scrollback.rs` now holds it once:

| Item | Rule |
| --- | --- |
| `content_settled(revision)` | the revision is even, so no write is in progress |
| `retryable_after(refused, revision)` | the pane shows content newer than the revision that was refused, and it is settled |

`find.rs`, `copy_mode.rs::due_retry` and `links.rs::LinkCell::at` call them;
their answers are unchanged and their tests pass untouched.

The phone (`mobile/rust/src/history.rs`) now refuses before sending, as the
window does. `MobileSession::settled_revision(target, refused)` reads the pane's
revision under the projection lock and, while it is mid-write (or still shows
the revision a `stale_content` answer already refused), waits on a condition
variable the drain signals for every event it takes in. So the phone waits for
a *frame*, not the clock: `SETTLE` (5 s) only bounds a run where nothing ever
settles, which returns `StaleView` — "wait for the current view" — instead of
telling the user the search timed out when no search was sent. The single
retry after a daemon refusal now uses the same wait, so it can no longer re-ask
at the revision it was just refused.

`MobileSession` gained one field, `frames: Arc<Condvar>`, which
`from_client` hands to the drain thread; `drain` notifies it where it already
bumps the view version. The mobile context window (the before, matched and
after reads, the `copy_motion` line-end probe, the `HistoryInbox` generation
fence and its 8-reply bound) is unchanged and stays the phone's.

The test fixture's pane carried `content_revision: 1`, an odd revision the
daemon would refuse; a phone that checks the rule before sending would send
nothing against it. `tests/mod.rs` now builds the pane at revision 2 and the
history tests follow, with the retry test moving on to 4.

## 3. Tests

| Added or re-pointed | Where |
| --- | --- |
| `an_odd_revision_is_a_write_in_progress_and_only_newer_settled_content_retries` | `crates/spaceport-client/src/scrollback/tests.rs` |
| `a_reported_path_is_only_a_staging_directory_and_a_plain_file`, `a_staged_file_is_the_name_the_script_reported`, `both_scripts_stage_under_the_hosts_own_temporary_directory`, `each_script_prints_only_the_directives_its_client_parses` | `crates/spaceport-client/src/staging/tests.rs` (new) |
| `only_generated_absolute_screenshot_paths_can_reach_the_terminal` — `/private/tmp` now accepted, a wrong or nested name still refused | `mobile/rust/src/ssh/images/tests.rs` |
| `a_search_waits_for_a_settled_frame_instead_of_asking_about_a_write_in_progress` (new); the other four history tests re-pointed to even revisions | `mobile/rust/src/tests/history_peer.rs` |
| `content_revision: 2` with the reason | `mobile/rust/src/tests/mod.rs` |
| `response_bounds_and_owned_directory_validation` — the moved path-shape loop is `staging`'s now | `crates/spaceport-client/src/upload_tests.rs` |

The two real-shell desktop tests (`remote_short_receive_trap_removes_file_and_directory`,
`remote_exclusive_create_never_overwrites_an_existing_file`) and the phone's
own run of `retain_script` (`confirmed_file_survives_disconnect_and_only_an_explicit_discard_removes_it`,
`partial_transfer_is_never_offered_for_paste`) all execute the shared scripts
against `/bin/sh` and pass, so the string construction is checked by a shell and
not only by `contains`.

## 4. What ran

```sh
cargo fmt --all -- --check                                    # clean
cd mobile/rust && cargo fmt -- --check                        # clean
cargo clippy -p spaceport-client --all-targets --locked -- -D warnings           # clean
cargo clippy -p spaceport-gpui --all-targets --features integration-test --locked -- -D warnings   # clean
cargo test   -p spaceport-client --locked                     # 410 passed, 2 + 4 ignored
cargo test   -p spaceport-gpui --bin spaceport -- find:: links:: copy_mode::   # 193 passed
cd mobile/rust && cargo check                                 # clean
cd mobile/rust && cargo clippy --all-targets -- -D warnings    # clean
cd mobile/rust && cargo test                                  # 56 passed, 2 live ignored
bash scripts/check-file-size.sh                               # clean
python3 scripts/check-sorted.py                               # clean
```

`CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` throughout, and
`SPARKLE_FRAMEWORK_PATH=$PWD/target/sparkle-sdk` for the app. Not run locally:
the full workspace suite, `just ci`, the Windows and Linux lint legs, the
mobile Xcode/Gradle suites, and native QA of the Find sheet and a screenshot
paste — deferred to CI. No UniFFI bindings were regenerated, which is safe
because no exported record, method or error changed.
