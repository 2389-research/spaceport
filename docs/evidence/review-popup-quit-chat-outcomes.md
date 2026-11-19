# Popup Quit and uncertain chat creation

Doctor Biz reported two P2 regressions in draft PR 125 on November 19, 2026.

## Popup Quit

The main window consumes catalog shortcuts before global bindings run.
Its generic command handler refuses commands while a popup is open.
Routing Quit through that handler therefore consumed the shortcut without quitting.

Quit now dispatches the existing app action through the window's deferred action path.
Other commands retain popup isolation, and held Quit events remain consumed.
The raw-key regression opens the actual shortcut popup and observes the app action.
It fails before the correction with zero Quit actions instead of one.
It passes after the correction, including ordinary-command isolation and held-key suppression.

The native Windows case opens the actual shortcut popup and sends Ctrl-Shift-Q.
It requires successful GUI exit, a live private daemon, and unchanged daemon tab IDs.
Its helper stops inspecting focus after sending Quit, since another app may then own focus.
Native execution remains pending CI.

## Chat creation

Herdr can accept tab creation but return a success envelope without usable tab or pane IDs.
The existing parser reports `StartError::Response` after that submitted request.
The control server previously converted that error into a generic failure.
A caller could retry despite the tab already existing.

The control response now uses the existing uncertain-outcome response:
“Chat creation outcome is unknown; inspect the window before retrying”.
Explicit daemon refusals retain their known-failure diagnostic.
No retry, protocol shape, or exit-code change is introduced.

The regression exercises the real framed daemon connection and private control socket.
Both missing-tab and missing-pane successes produce the uncertain response.
An ordered input fence proves no replay or launch input precedes completion.
The typed and framed cases fail before the fix with the generic missing-ID diagnostic.

## Verification

`cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- window::tests::shortcut_repeat:: control::server::chat::tests:: --nocapture` passes 28 tests, with none ignored.
Logs: `/tmp/spaceport-9s44-p2-quit-creation-{red,green}.log` and `/tmp/spaceport-9s44-p2-chat-framed-red.log`.
Independent review finds no material routing, outcome, ownership, or test gap.
Both host lint configurations pass.
Both Windows GNU lint configurations pass after importing the native test's Windows `CommandExt` trait.
Format, size, sorting, operations, localization, commit, and diff checks pass.
Full platform CI and native popup Quit execution remain pending for these fixes.
