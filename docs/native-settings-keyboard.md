# Native Settings keyboard qualification

Run `just test-settings-keyboard` on an unlocked macOS desktop.
Select the window titled **Spaceport keyboard check** when it appears.
Leave that window in front while the fixture runs.
The fixture observes the exact window's key state; no chat reply is needed.

The foreground gate waits at most 60 seconds.
The parent allows 120 seconds for its one child process.
The test does not retry a timeout or an assertion failure.
It restores the Settings title after the gate succeeds or fails.

The ignored `native_settings_keyboard` test launches the actual app directly.
It keeps the child's exit status and kills/reaps only that owned child during cleanup.
Its cleared environment selects private HOME, XDG, preferences, and socket paths.
It starts no daemon and uses no provider or personal clipboard.
Only this fixture sets the test-only `HERDR_TEST_SETTINGS_KEYBOARD` selector.
The ordinary `native_sidebar` test keeps its existing checks and resource requirements.

The additional matrix sends AppKit key-down and key-up events to the retained Settings window.
It checks Tab and Shift-Tab traversal, painted focus rings, and Enter/Space activation.
It also checks dropdown arrows and focus restoration, the font modal, and invalid numeric edits.
Changed private preferences return to their original values.
Every key requires that exact Settings window to remain key.

These are native AppKit event checks, not global physical keyboard or IME qualification.
The fixture does not change activation policies, OS preferences, or security controls.
It does not claim a pass unless the foreground gate and every assertion pass.
