# Remaining Spaceport display copy

This covers the remaining display strings listed in GitHub issue #16.
PR #8 already changed the chat-startup diagnostics.
This pass changes helper help, attachment errors, agent-facing messages, Core version guidance, and the preview.
Runtime text uses the existing `sheprd_client::APP_NAME` identity.
The static settings seed retains its public constant interface.
Only newly created overrides receive the Spaceport heading.
Existing overrides remain byte-for-byte unchanged, including historical comments.

Codex keeps the `herdr-acp` wire client name.
Both bridges retain their request IDs, refusal codes, and response envelopes.
Helper executable names, options, socket paths, and runtime lifecycle stay unchanged.
The preview removes its upstream product-guide sample link.
Its title and the private Linux X11 selector both name Spaceport.
The helper package description and icon-decode diagnostic also name Spaceport.
Legal credits, compatibility search paths, and icon filenames retain their original identities.

## Verification

Logs live under `.worktrees/spaceport-display-copy/target/`.

- ACP helper tests: 382 passed.
- Actual helper `--help` executable: passed with an empty inherited environment.
- Client unit tests: 396 passed.
- Core client unit tests: 67 passed.
- Additional integration and documentation tests passed; live cases remain opt-in.
- Headless GUI branding checks: 15 passed.
- Headless GUI chat checks: 459 passed.
- Native preview: four light/dark and wide/narrow cases passed.
- Python native-input helpers: four unit tests passed.
- Strict targeted Clippy, quick canonical checks, and specification integrity passed.

The red logs fail on Sheprd before the text changes.
They check typed errors, serialized bridge messages, actual help output, and generated settings.
The preview test reads the real installer's window-title setter and installed transcript.
It reports both the old title and upstream link before the change.
The settings regression loads a personal file and compares its bytes afterward.
Protocol tests use scripted peers; they do not establish real-provider session behavior.
The helper and native preview checks execute actual isolated binaries.
No real agent or personal daemon was started.

## Native capture correction

The first native captures exposed stale layout after resizing an inactive window.
The conversation viewport remained 919 pixels wide inside a 650-pixel window.
A bounds assertion reproduced that failure in `branding-native-geometry-red.log`.
The fixture now refreshes and draws before reading its GPU image.
The narrow viewport then measures 489 pixels at x=160.
Its right edge fits the 650-pixel window.
The assertion covers the conversation case without adding requirements to other sample cases.

Before captures restore the original title and link with the same corrected draw fence.
The after source was restored byte-for-byte afterward.
Images show real native window content, not OS titlebar pixels.
The headless installer test separately checks the title passed to GPUI.
Actual Linux X11 pointer/title matching was not run on this Mac.
The selector changed with its producer, and Python helper unit tests passed.

| Theme and width | Before | After |
| --- | --- | --- |
| Dark, wide | [Before](display-copy/before-dark-wide.png) | [After](display-copy/after-dark-wide.png) |
| Dark, narrow | [Before](display-copy/before-dark-narrow.png) | [After](display-copy/after-dark-narrow.png) |
| Light, wide | [Before](display-copy/before-light-wide.png) | [After](display-copy/after-light-wide.png) |
| Light, narrow | [Before](display-copy/before-light-narrow.png) | [After](display-copy/after-light-narrow.png) |

## Limits

This does not complete package identity or broad T01/T12 acceptance.
Windows resource ProductName and FileDescription still name Sheprd in `windows_resource.rs`.
That remaining work is recorded in Kata `d4z8`.
Windows checks remain paused under D66 pending native qualification.
No signing identity, release channel, installed bundle, or user configuration was migrated.
No tag or release was created.
Existing Darwin linker and `block 0.1.6` warnings remain.
The first native build failed when the disk filled; that was not a test pass.
Only this task's disposable GUI compiler cache was cleaned before the successful rebuild.
