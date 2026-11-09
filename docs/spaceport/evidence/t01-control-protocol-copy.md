# T01 control protocol product copy

Source base: integration `78dc8177c`. The local desktop control protocol
still emitted `Sheprd` in typed connection errors and several socket replies
even though the app's English catalog says `Spaceport`. This is user-facing
product copy on a retained private CLI route; it does not change wire fields,
error codes, endpoint ownership, or protocol framing.

The existing `every_control_variant_has_a_message` test failed during the
T04 control suite: `NoResponse` displayed `Sheprd` in `thiserror` and
`Spaceport` in Fluent. A new exact oversized-reply assertion failed with
`Sheprd's answer is larger...` on the wire. The typed errors and socket
responses now use Spaceport, including busy, shutdown and timeout replies.
The error comment names `spaceport <command>`.

The control protocol suite passed 61/61 after the fix. The GPUI control
filter passed 108/108 after a scoped rebuild of the changed protocol crate.
An all-target, all-feature workspace check passed with no new source warning.
The first real CLI integration run found one stale `Sheprd is not running`
fixture; updating that expected text made the full suite pass 20/20.
Native product-copy and side-by-side install checks have not run;
A42 remains `not_run` and T01 remains `in_progress`. Production identity and
signing stay gated by G-RELEASE. No installed app, personal socket, or user
state was changed.
