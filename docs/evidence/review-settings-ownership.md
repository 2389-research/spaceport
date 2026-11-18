# Settings ownership

Date: 2026-11-17. Scope: `n6a1` under `9s44`.

Spaceport keeps its theme, sound, toast, sidebar, and prefix settings in its private `view.toml`.
Labels, source tags, and documentation now name that file and its owner.
Herdr's runtime preferences are not imported into private view settings.
The Ctrl+A regression checks the real private-view keymap consumer.

The local pane-history switch now edits Herdr's actual runtime configuration.
Its path honors nonempty `HERDR_CONFIG_PATH`, then the existing release configuration root.
The default remains `herdr/config.toml`.
The typed history owner carries either a guarded local snapshot or the original SSH target and snapshot.
Generation fences reject stale results. Busy hosts cannot accept another write.
Successful writes reload only the matching runtime targets.
Explicit sockets and development sessions do not receive guessed local reloads.

The first regression used the real rendered control and private subprocess files.
It showed the view file changing while the runtime file stayed unchanged.
The final regression preserves the view file and edits the selected runtime file.
Review then found that closing Settings canceled accepted local writes.
The final policy lets accepted local writes finish with their captured guarded snapshot.
Abandoned reads and SSH writes still cancel.
Both the real close-after-click fixture and paired local/SSH owner tests pass.

Settings tests passed 246. Guarded persistence tests passed 48. Host-history tests passed seven.
Runtime path tests passed eight, and the Ctrl+A consumer test passed.
GPUI bin/test Clippy, scoped formatting, file-size, sorted-region, and whitespace checks passed.
Root independently reviewed the actual model, callers, file guards, cancellation, and reload targeting without remaining findings.

Native keyboard and performance checks remain pending at Doctor Biz's request.
Native glyph/layout checks and real-daemon scrollback replay are not claimed.
Windows retains the guarded writer's existing read-only limitation and explains it in the UI.

Detailed logs and report: `/tmp/spaceport-qnz9/n6a1-report.md`.
Regression logs: `n6a1-red.log` and `n6a1-close-red.log` in that directory.
