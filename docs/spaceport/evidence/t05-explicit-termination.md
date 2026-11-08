# T05 private Core exact-address termination

An opt-in client test starts two `sleep 60` fixture terminals under a private
Core socket and home. Both receive the same non-authoritative
`fixture.display` label, while Core gives each a distinct terminal address
and PID. Two observer views attach to the selected terminal and a third to
the other. `terminal.kill` uses the selected address. Its reply reports that
address finished; `terminal.get` reports the other address still live with
the same PID, and its observer stream remains open. The test then kills the
survivor and stops only its own Core child through the shared sandbox. No
personal daemon or user state is read.

The first fixture tried duplicate Core terminal *names*. Core rejected the
second `terminal.run` with typed `name_taken`; live names are unique. The
revised fixture uses duplicate labels, which model the ambiguous display
case without contradicting Core's name contract. It passed 1/1 against the
pinned `hc 0.1.18` binary at `/tmp/spaceport-t00-target/debug/herdr-core`.
The default `sheprd-core-client --lib` suite passed 63 active tests with 11
opt-in tests ignored. `just format-check` passed after correcting one line
wrap; `just check-file-size` and `git diff --check` passed.

This proves the Core client's exact-address kill path and observer isolation
for local fixtures. It does not exercise a native Spaceport terminate action,
remote machine, or OS process inspection, so A23 remains `not_run` and T05
remains in progress.
