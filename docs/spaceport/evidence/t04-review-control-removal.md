# T04/RM-IDE: review control route removal

At `5df39947`, `sheprd review` still opened a native Git review from the
caller's checkout. The CLI sent `TabOp::Review` through the private control
socket; the server switched to the caller's workspace before opening the
review. This command was a separable excluded route. Native review tabs and
their listing remain reachable through the UI, so this cut does not finish
RM-IDE or A29.

The CLI regression first failed because `review` parsed. The wire regression
first failed because `{"op":"review"}` decoded. Both now pass: the command
returns the unknown-command error before control dispatch, and the old wire
operation fails strict decoding. Protocol version 21 records the shape
change. A retained Devices request still round trips with its exact socket,
workspace and window target. Existing review feedback and tab listing remain
while the native surface exists.

On macOS, the control protocol library passed 55/55; the focused GUI CLI
regression passed 1/1, server tab tests 8/8, and adjacent CLI parser tests
10/10. Default and all-feature production app checks passed. File size,
sorted-list and i18n checks passed (5,166 messages in 269 files), as did
`git diff --check`. The GUI test link emitted its existing macOS compact
unwind warning; Cargo reported the existing `block v0.1.6` future
compatibility notice. Native window use, Windows/Linux, a full workspace
test and real app IPC were not exercised. T04 remains in progress and A29
remains `not_run`.
