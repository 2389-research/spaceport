# Spaceport Mobile

Open a saved SSH host and named Herdr TUI session from Android, iPhone, or iPad.
The host needs an ordinary SSH account with a POSIX shell and a compatible installed Herdr 1.
Tailscale is optional. The desktop app can be closed.

## Build and run on Android

Use macOS or Linux with Bash, JDK 17, and Rustup. Use the Rust toolchain pinned in this repository.
Install Android SDK Platform 36, Build Tools 35.0.0, Platform Tools, and NDK 28.2.13676358 with Android Studio’s SDK Manager.
Set `ANDROID_HOME` if your SDK lives outside the default location.

Run these commands from the repository root to install a debug build on a connected device.

```sh
./mobile/build-android
adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk
```

The Kotlin app supports Android 8.0 and newer on ARM64 and x86-64.
Gradle builds the shared Rust controller for both ABIs and generates matching Kotlin bindings with UniFFI.
The debug application ID is `ai.research2389.spaceport.mobile.debug`.
The APK uses Android debug signing. `./mobile/build-android --release` builds an unsigned release APK.

Signed release APKs come from `.github/workflows/android-release.yml`.
It runs after the macOS preview workflow publishes a `v*` release.
If the APK's sources changed since the last release that has an APK, it attaches `Spaceport-<version>-android.apk` and its `.sha256` to that release.
Those sources are `mobile/android`, `mobile/rust`, every shared crate it reaches by path, the root `Cargo.toml` and `rust-toolchain.toml`.
The release application ID is `ai.research2389.spaceport.mobile`, and its `versionCode` is `YYYYMMDD×100+N` from the tag, so each release installs over the last.
The `android-release` environment holds the key as secrets `SPACEPORT_ANDROID_KEYSTORE_BASE64`, `SPACEPORT_ANDROID_KEYSTORE_PASSWORD`, `SPACEPORT_ANDROID_KEY_ALIAS`, and `SPACEPORT_ANDROID_KEY_PASSWORD`.
Its variable `SPACEPORT_ANDROID_CERT_SHA256` pins the certificate; the workflow refuses an APK signed by any other key.
The same run keeps a signed Play bundle, `Spaceport-<version>-android.aab`, as the workflow artifact `android-play-bundle-<version>` for 90 days.
Play registers this key as the upload key.
Play signs installs with its own Google-generated key (certificate SHA-256 `A4:79:B9:98:53:07:0C:BC:4D:00:45:93:69:1A:71:68:18:B6:45:83:75:4D:3B:B2:7A:D2:EF:FC:81:71:BF:F2`).
A phone therefore cannot switch between a Play install and a GitHub APK without uninstalling first.
A `play` job then sends that bundle to the Play internal testing track with `scripts/release/play-upload.py`.
Its `play-internal` environment holds only `SPACEPORT_PLAY_SERVICE_ACCOUNT_JSON`, the service account that may release to testing tracks.
Production stays a person's decision in the Play Console.
By hand, `scripts/release/play-upload.py --credentials KEY.json tracks` lists what each track holds.
Even `tracks` opens a Play edit, and Play keeps one open edit per account and app.
Do not run it, or a manual upload, while the workflow's `play` job is uploading; the newer edit makes that upload fail.

1. Open **Add host**. Enter the SSH hostname, port, username, and Herdr session. Tap **Connect**.
2. Compare the displayed SSH fingerprint with the host administrator’s copy before trusting it.
3. For password sign-in, enter the account password. Leave **Authorize this phone for future sign-ins** checked to enroll the device.
   Spaceport installs the public key through the password session and verifies a fresh key-only SSH login.
   You can also enroll later from **Session → Phone key → Install key**.
4. For an account that requires a key, open **Phone key → Copy public key** at the password prompt.
   Authorize that public key for the SSH account, then tap **Use phone key**. The P-256 private key stays in Android Keystore.
   Key login requires Android 9 or later and secure hardware. The phone must be unlocked to sign.
   Android 8 and devices without a hardware-backed key support password sign-in.
5. Tap **Go To** to select an agent or workspace. For a split tab, tap a tile on the pane map.
   Android draws Herdr’s terminal cells on a native Canvas.

Home groups saved sessions under their SSH account. Tap **New session** to add a Herdr session on that account.
Tap **Details** for the address, trusted host key, phone key, and sessions. Hold a session there to rename or remove it.
Forgetting a host key closes that host's open connections. You verify the fingerprint again on the next connection.
On Home, swipe an agent toward the end to pin it, or toward the start to snooze it for an hour.
Touch and hold for other snooze lengths, including until 9 AM tomorrow. TalkBack offers the same choices as actions.
The terminal's **Home** button shows a dot when an agent in another pane needs you.

**Settings** shows the phone's public key with **Copy public key** and **Share**.
**Install on a host** asks for the account's SSH password once. It installs the key and checks a fresh key-only login without starting Herdr.
Later connections to that account skip the password. Settings also sets the terminal text size, the toolbar keys, and **Press Return after pasting**.
The toolbar keys are one ordered list of up to 16; Shift, Control, and Alt apply to the next key.
Hold the terminal and drag to move the cursor with arrow keys. Hold still to copy a line or the screen, or to paste.

On Android 12–14, set a PIN, password, or pattern before creating a phone key.
Removing the screen lock can delete that key. If it does, install the replacement key on the host.
Some biometric unlocks on these versions require a subsequent PIN, password, or pattern unlock before signing.
Android 15 fixes these screen-lock issues. [Android Keystore documentation](https://developer.android.com/reference/android/security/keystore/KeyGenParameterSpec.Builder#setUnlockedDeviceRequired(boolean))

For Gradle commands outside Android Studio, set `ANDROID_HOME` to the installed SDK or set `sdk.dir` in `mobile/android/local.properties`.
Run unit tests with the first command. Connect a device or emulator for the second.

```sh
./mobile/android/gradlew -p mobile/android :app:testDebugUnitTest
./mobile/android/gradlew -p mobile/android :app:connectedDebugAndroidTest
```

The device suite checks the actual Rust library packaged in the APK.
To run [LiveSessionTest](android/app/src/androidTest/java/ai/spaceport/mobile/LiveSessionTest.kt), supply a fresh disposable SSH endpoint.
Pass `herdrHost`, `herdrPort`, `herdrUser`, `herdrPassword`, `herdrFingerprint`, and `herdrSession` as Android instrumentation arguments.
For the [terminal UI test](android/app/src/androidTest/java/ai/spaceport/mobile/LiveTerminalTest.kt), seed a separate session with `mobile/test-support/seed-agent-order`.
Pass its name as `herdrTerminalSession`. The test replies to the disposable Reviewer pane.
The [Home UI test](android/app/src/androidTest/java/ai/spaceport/mobile/LiveHomeTest.kt) accepts a separately seeded session through `herdrHomeSession`.
These tests skip themselves when their fixture is absent.

## Build and run on iOS

Use an Apple silicon Mac with Xcode, Rustup, the repository's pinned Rust toolchain, and XcodeGen.
Run these commands from the repository root.

```sh
./mobile/build-apple
open mobile/apple/SpaceportMobile.xcodeproj
```

Select the SpaceportMobile scheme and an iOS 17 or newer simulator or device.
Select your signing team in Xcode when deploying to a device.
Re-run the script after changing Rust or adding Swift source files.
The script builds both ARM64 static libraries and generates Swift bindings with the locked UniFFI version.
It keeps the build tools and XCFramework inside the worktree.

For TestFlight, run `./mobile/build-apple --release` before archiving the Release scheme configuration.
This packages optimized Rust libraries for both platforms. Archive for a generic iOS device.
Use the 2389 Research distribution team and the `ai.2389.spaceport.mobile` App Store Connect record.
Debug builds use `ai.2389.spaceport.mobile.development`.
For device builds, enable App Groups on the app, Agent Status extension, and Share File extension.
Include `group.ai.2389.spaceport.mobile.widget` in all three distribution profiles.
Debug profiles use `group.ai.2389.spaceport.mobile.development.widget`.

### TestFlight export compliance

The Release target merges [ReleaseInfo.plist](apple/ReleaseInfo.plist) into the signed app.
The plist carries the same Boolean `false` recorded in App Store Connect for version 0.1, build 1.
Spaceport ships SSH encryption in its Rust library. Include that library when answering Apple's questions.

If the determination is exempt, set `ITSAppUsesNonExemptEncryption` to the Boolean `false`.
For non-exempt encryption, set it to the Boolean `true`.
When Apple issues a code for Spaceport, set `ITSEncryptionExportComplianceCode` to that exact string.
Revisit the values if the app's cryptography or distribution changes.
For an existing upload, use **Manage** on the build in TestFlight.
[Apple's export compliance instructions](https://developer.apple.com/help/app-store-connect/manage-app-information/determine-and-upload-app-encryption-documentation)

## First connection on iOS

1. Open **Add Host** and enter `user@host[:port]`. Choose Password or Phone Key and name the first Herdr session.
2. Tap **Connect**. Compare the complete SHA-256 fingerprint with the host administrator’s copy before granting trust.
3. For password sign-in, leave **Authorize This Phone** on to install the phone key, or turn it off for password-only access.
4. Sign in. Tap the pane name in the terminal header to browse workspaces, tabs, and panes.
5. Tap the terminal to type. Input goes to the displayed pane or popup.

The iOS agent list shows the hostname. A shell title such as `pilot@box: ~/src` appears as `box`.
Saved connections show a custom SSH port when one is set. Edit the host to change its SSH username.

Home puts pinned agents first, followed by **Waiting for you** and **Working**. Idle agents follow in three approximate age bands: under an hour, one to five hours, and five hours or longer.
Agents first found idle appear under **Idle**, newest Herdr sequence first within each session. Once Spaceport confirms the same idle streak for an hour, its row shows a lower bound such as `2h+`.
While Spaceport is active, Home connects directly to the hosts and named sessions saved on this iPhone or iPad.
A trusted host configured for Phone Key can report agents from a running session before you open its terminal.
Home observes the running session through its existing socket. If the session stops, Home keeps its last observation.
For password sign-in, open the session once. Returning Home keeps its SSH login available for roster updates.
Home shows **Live** beside a session while it receives updates. Other sessions show their last observation.
The phone can observe four Phone Key sessions at once and rotates through additional saved sessions.
Tap an agent to open its terminal on the host shown in its row.
Backgrounding the app pauses observation. Choosing **Disconnect** on a session pauses it until you open it again.

Home measures idle time through the last confirmed observation. A cached session keeps that age until it reconnects.
Home and Go To show each agent’s terminal title above its name and location. Home keeps the last title observed for that session.
Spaceport checks the daemon boot, workspace, tab, pane, and reported agent name before enabling input.
Choose **Workspaces** underneath the agent list to browse a saved session’s live workspaces.
Saved sessions appear next, grouped by host.
The time beside an activity count tells you when the phone last observed it.
Open Host Detail using the info button to add sessions or manage the host’s sign-in and trust.

Opening a named session runs the installed Herdr bridge. Herdr starts the session if needed.
Disconnecting closes the client's SSH channel. Herdr continues to own its terminals and agents.
Native chat is deferred. Agent prompts and approvals use their original terminal.

## Siri, Shortcuts, and the Action Button

In Shortcuts, choose **Spaceport** to see its five actions.

| Action | What happens |
| --- | --- |
| **Next Waiting Agent** | Refreshes the selected session over SSH and opens a waiting agent. If nobody is waiting, opens Agents. |
| **Open Agent** | Opens the chosen agent after Herdr confirms its identity and pane. |
| **Resume Session** | Returns to the terminal in the current session, or the last observed session on a cold launch. |
| **Get Agent Status** | Returns the last saved status and observation time. Works offline. |
| **Compose Prompt** | Opens an editable draft. Supplied text follows any existing draft text. Review the destination and tap **Send** or **Paste** when ready. |

On an iPhone with an Action Button, open **Settings → Action Button → Shortcut → Choose a Shortcut**.
Select Spaceport’s **Next Waiting Agent**. Leave the session unset to use the foreground session or your last observed session.
To target a particular session, save a shortcut with that session selected and assign it to the button.
Open a session in Spaceport once to populate its cached agents. Spaceport uses the usual SSH sign-in and host verification when connecting.

## Home Screen widget

Touch and hold the iPhone Home Screen. Choose **Edit → Add Widget → Spaceport → Agent Status**.
The small and medium sizes show waiting, working, and idle counts for the last observed session.
The time below the counts tells you when Herdr supplied them. Open a session to record the first observation.

Tap the widget to refresh that session over SSH and open its next waiting agent.
If nobody is waiting, Spaceport opens the Agents list. The widget itself makes no network requests.
It works independently of the Live Activities setting.

## Review agents and filter hosts

On iOS, **Finished for review** shows completed agents. Hold a completed row and choose **Mark reviewed** to move it into **Quiet**. A new working or waiting observation clears the review mark. Quiet also folds idle agents with at least 24 hours of confirmed inactivity. Open its heading to see those agents.

In Host Details, choose **Edit tags**. Separate tags with commas. The Home filter applies the selected tag to hosts, sessions and agents. Tags and review marks stay on the phone.

Choose **Copy connection diagnostics** from the connected session menu or the connection failure screen. The report includes the build, connection stage, most recent failure stage, known negotiated capabilities and last roster observation.

Debug builds also include a [loopback HTTP service preview](test-support/loopback-preview/README.md).

## Terminal rendering on iOS

On iOS, Ghostty renders every terminal pane.
The native Agents and Workspaces lists still use the existing session connection.
Spaceport resolves the selected pane in that named session and runs Herdr’s direct attach on a second SSH channel.
Herdr 0.9.5 gives its startup terminal probe 250 ms. Spaceport lets that probe expire because late replies can land in the shell.
This uses Herdr’s cell-based mouse coordinates for that attachment. Queries from programs in the pane still reach Ghostty.
Spaceport prepares the next terminal channel while Herdr looks up the pane.
Switching panes releases the old channel. Herdr handles input ownership without a forced takeover.
Herdr supplies popups as cell snapshots. Spaceport draws them in a separate Ghostty surface and sends popup input to Herdr.

The app embeds the pinned Ghostty Apple package from [libghostty-spm](https://github.com/Lakr233/libghostty-spm).
Xcode resolves its Swift sources and binary framework when building the app. The remote host still needs only SSH and Herdr.
The app hosts the package’s UIKit view directly. Its SwiftUI state wrapper crashed during synchronous teardown on the iOS 26.2 simulator.
When signing a manual archive, pass the app profile with `SPACEPORT_MOBILE_PROFILE`.
Pass the separate `ai.2389.spaceport.mobile.agent-status` extension profile with `SPACEPORT_ACTIVITY_PROFILE`.
Pass the `ai.2389.spaceport.mobile.share` extension profile with `SPACEPORT_SHARE_PROFILE`.
Include all three bundle IDs and their respective profiles in a manual export. The Ghostty resource bundle has its own signing settings.

## Install the iPhone key

When you sign in with a password, **Authorize This Phone** can install the public key using the open SSH connection.
Spaceport opens a separate key-only login to verify it. If verification fails, you can keep using the password session.

To install the key separately, open **Settings → Phone Key → Install on a Host…** and choose the destination account.
For a saved password host, you can also open the installer from Host Detail.
Enter the SSH password and tap **Install key**. Verify the fingerprint on first use.
Spaceport appends the public key on the POSIX host, then checks a fresh key-only SSH login.
After verification, Spaceport selects phone-key sign-in for the saved host.
If a separate saved host already uses that account with Phone Key, both keep their sign-in settings.
The installer works over regular SSH without opening or starting a Herdr session.

New phone keys use P-256 ECDSA. Apple's Secure Enclave signs SSH challenges while the device is unlocked.
The Keychain holds a device-only key reference. Rust receives the public key and signatures.
Settings can copy or share the OpenSSH public key for accounts with password sign-in disabled.
If an account trusted the Ed25519 key from build 6 or earlier, install the new public key there.
Spaceport leaves existing authorized_keys entries in place.

## Working with a desktop client

Each connection selects its own tab. Clients viewing the same tab share its focused pane.
Their screens can show different numbers of cells while the shell has one PTY size.
In the Herdr 0.9.5 live test, the smaller client claimed a 21-column split pane.
The shell reported 21 columns to both clients, including the one with a 37-column pane.
Mobile uses the desktop client's recent-input rule when reclaiming a frame.

On iOS, choose **Settings → Terminal → Appearance** to set the terminal’s light or dark palette.
The app starts in dark mode. The window also follows default background changes delivered to Ghostty by the active terminal stream.
The first-host screen uses the rocket artwork from the Spaceport landing page.

On Home, swipe an agent right to pin it or left to snooze it for an hour.
Touch and hold for other snooze lengths or to wake it early.
Pinned agents lead the list. Snoozed agents appear at the bottom until the deadline,
or until Spaceport observes them change to working or waiting for you.
These settings stay on this phone and belong to the observed agent in its current Herdr boot.

Swipe right across the terminal or tap the pane title to return to **Agents**.
On a standard iPhone this opens **Go To**. On an open Duo it expands the agent sidebar.
**Agents** puts waiting agents first.
Choose **Workspaces** to reach tabs and panes. Sessions with only shells open on Workspaces.
Tap **+** beside **Home** to create and focus a terminal tab in the current workspace, even from inside an agent.
From a shell prompt, `herdr tab create --focus` does the same thing.

On the open iPhone Duo, a wide session shows an agent sidebar beside the terminal.
When the Duo displays vertical toolbars, Home and New Terminal appear below the system status indicators.
Find in Terminal and Compose appear farther down the same side. Tap the agent title to open Agents.
The terminal header shows the focused agent’s reported title when available. The sidebar footer counts agents by status across the session.
**Waiting for you** comes first, then working and idle agents. Workspaces follow the agents.
Agent rows show the same agent artwork as the desktop app. The gold pane title leads each row, followed by the terminal title. An unnamed pane uses its tab number or title.
The workspace appears in teal and the agent name in purple.
When Home needs to distinguish sessions or hosts, it shows the session in blue and the host in rose.
Working dots pulse slowly. Waiting dots pulse twice, then pause. Idle dots stay still. Reduce Motion keeps every dot still.
Choose **Agents** or **Workspaces** at the top of the sidebar. Browsing a workspace leaves the live terminal beside it.
On the wide display, opening the terminal keyboard slides the agent list into a rail. Closing the keyboard restores the list unless you collapsed it yourself.
Tap another agent in the rail to switch with the keyboard open. Spaceport discards keystrokes during the switch and accepts input once Herdr confirms the new pane.
Turn on Reduce Motion in iOS to switch layouts immediately. Tap an initial to switch agents or expand the rail to see names.
If another agent starts waiting while you type, **Later** dismisses the notice and **Jump** switches to that agent.
Your half-typed line stays in the original pane.
Use the terminal header to identify the current pane, find text in its history, or compose a draft beside it.
On either display, tap the house beside Compose to return to Home.
A narrow window gives the terminal the full width. Folding or resizing keeps the selected pane and font size.
On the narrowest layouts, Escape stays visible and **More terminal keys** holds the other controls.

In the compact layout, tap the pane name in the header to open Go To. Tap the waiting agent’s name to jump directly.
In **Agents**, waiting agents appear first, followed by working and idle agents.
Search by agent, terminal title, pane, workspace, or tab. Recent visits break ties between equally strong matches.
In **Workspaces**, search for a workspace by name. Open it to see its panes grouped by tab.
Search within that workspace by tab, pane, terminal title, or agent. Back returns to the workspace list.
Choose a pane to switch terminals. Browsing, searching, and going back keep Herdr’s focus on the current terminal.
In **Agents**, touch and hold a row or search result to peek without moving focus.
Peek shows the current screen, a dated excerpt from a previous visit, or the pane’s details if no excerpt is available.
When you switch back, **Last screen** shows the saved text while Ghostty attaches.
iOS keeps up to two recent screens in memory and clears them with the excerpts when the session parks or disconnects.
Tap a waiting pane to jump to it. Input resumes when Herdr confirms the destination and its terminal surface.
For split tabs, tap a tile on the pane map or choose a pane by name.
Opening Go To leaves focus in place. Dismissing it restores the keyboard if it was open.
Clients on the destination tab share its selected pane.
Use **Previous pane** or **Next pane in history** on the terminal toolbar to revisit a confirmed focus.
The trail skips closed panes and starts over when Herdr restarts. A swipe still opens Agents.
Long-press a pane in Go To for **Split right**, **Split down**, **Zoom pane** or **Rename pane**.
The menu offers actions supported by the connected Herdr. **Restore panes** ends zoom.

In Go To, tap **Find in Terminal** to search the focused pane’s retained Herdr history.
Find opens the newest match with its surrounding text. **Older** and **Newer** wrap between matches.
**Copy match** copies the terminal’s original text. **Done** returns to the live terminal at its current position.

Tap **Compose** to edit a multiline draft on the phone. Paste it into the named terminal when ready.
Copy a screenshot, then tap the image paste button in Compose. Review its thumbnail, add an optional note, and confirm the destination.
Spaceport stages the PNG in a private temporary directory on that SSH host. The terminal receives its path followed by your note.
The host confirms the file before Spaceport pastes its path. Accepted files stay there until you or the host clears them.
On a fresh install, **Send** pastes and presses Return. Turn off **Press Return After Pasting** to use **Paste**.
The switch shares its saved value with **Settings → Press Return After Pasting**.
Send queues the paste and Return together for the same destination.
Each retained session keeps its unsent draft in memory.
If the destination changes, choose the new destination explicitly or switch back to the original pane.

### Attach a file to a terminal

Open Compose in the destination terminal and tap File on iOS.
Select a document up to 16 MiB. Check the host, session and pane, then tap Paste or Send.
Spaceport uploads the file over the active SSH connection and pastes its remote path.

From another app, share a file to Spaceport and save it locally.
Open Compose in the terminal and select Attach shared file.
One incoming file can wait on each device. Attach or discard it to make room for another.
Closing the composer during an upload cancels delivery and preserves the draft.

Set the terminal text size and choose your toolbar keys in Settings. Swipe the keys for more controls. Drag vertically on the terminal to scroll.
Each window keeps its two most recently opened sessions in memory. Go Home or switch agents to reuse the SSH login.
On return to a parked session, Spaceport checks the existing SSH connection and waits for Herdr before enabling input.
If the active session loses its SSH transport or the host stops answering the resume check, Spaceport tries one fresh connection.
It checks the host key again. A password cleared by leaving for Home must be entered again.
A refused resume or an interrupted agent switch requires a fresh selection. Spaceport never replays terminal input.
Choose **Disconnect** in Go To to close the connection and discard its local draft.

In Spaceport, **Settings → Agents → Live Activities** controls whether the selected session shows agent counts on the Lock Screen.
The switch defaults to on. Turning it off removes the current activity and saves your choice on this device.
When iOS allows Live Activities, the activity shows waiting, working, and idle counts.
On supported iPhones the Dynamic Island shows the highest-priority count. Expand it to see all three.
Locking the phone parks SSH. The activity shows **Last seen** with the time of the last observation.
It also shows **Last seen** after two minutes without another observation.
Reopening the session refreshes the counts after Herdr responds. Going Home or disconnecting ends the activity.
The widget receives the counts, observation time, and whether updates paused. It makes no network requests.

## Verify

```sh
cargo test --locked --manifest-path mobile/rust/Cargo.toml
cargo test --locked -p spaceport-client
```

Use Product → Test in the generated Xcode project for Keychain, profile and UI coverage.
The live tests require a **disposable SSH account and named session with a POSIX shell**.
Choose a fresh session name for each run. Both live tests authorize a new phone key in that account.
The Rust test creates a tab and splits a pane. The UI test splits a single-pane session to exercise the map.
XCTest records simulated typing in its local logs.
Use an expendable fixture password.
Configure the following test environment variables explicitly.

| Variable | Value |
| --- | --- |
| `SPACEPORT_TEST_SSH_HOST` | Fixture host reachable from the test client |
| `SPACEPORT_TEST_SSH_PORT` | Fixture SSH port |
| `SPACEPORT_TEST_SSH_USER` | Disposable account |
| `SPACEPORT_TEST_SSH_PASSWORD` | Disposable account's password |
| `SPACEPORT_TEST_SSH_FINGERPRINT` | Independently verified SSH host-key fingerprint |
| `SPACEPORT_TEST_HERDR_SESSION` | Named Herdr fixture session |
| `SPACEPORT_UI_FILE_SESSION`, `SPACEPORT_UI_SHARE_SESSION` | Fresh shell sessions for the picker and incoming share tests. Put `SpaceportFixture.txt` containing `verified-native-file-content` in On My iPhone. |
| `SPACEPORT_TEST_SHORTCUT_SESSION` | Fresh session from `mobile/test-support/seed-agent-order` for warm and cold entry through Apple Shortcuts |
| `SPACEPORT_TEST_HOME_ROSTER_SESSION` | Fresh session from `mobile/test-support/seed-attention` for refreshing a parked agent from Home |
| `SPACEPORT_TEST_HOME_WIDGET_SESSION` | Fresh session from `mobile/test-support/seed-agent-order` for Home Screen gallery placement and tap-to-open testing |
| `SPACEPORT_TEST_COMPOSE_SHORTCUT_SESSION` | Separate session from `mobile/test-support/seed-attention` for the draft and empty waiting queue workflows |
| `SPACEPORT_TEST_SHELL_SESSION` | Disposable workspace with only shell panes for the new-terminal workflow |
| `SPACEPORT_TEST_APPEARANCE_SESSION` | Disposable session focused on a shell for the light SSH terminal and Home appearance test |
| `SPACEPORT_TEST_ACTIVITY_SESSION` | Fresh session from `mobile/test-support/seed-agent-order` for the Lock Screen and Dynamic Island test on a supported iPhone |
| `SPACEPORT_TEST_ATTENTION_SESSION` | Separate seeded session for the cross-workspace UI test |
| `SPACEPORT_TEST_WORKSPACE_DUO_SESSION` | Fresh session from `mobile/test-support/seed-attention` for browsing beside the live terminal on an unfolded Duo |
| `SPACEPORT_TEST_WORKSPACE_PHONE_SESSION` | Fresh session from `mobile/test-support/seed-attention` for workspace search and keyboard restoration on iPhone |
| `SPACEPORT_TEST_NEW_TERMINAL_SESSION` | Separate session from `mobile/test-support/seed-attention` for creating a terminal beside an agent |
| `SPACEPORT_TEST_LIFECYCLE_SESSION` | Fresh disposable session name for testing drafts and transport recovery |
| `SPACEPORT_TEST_SSH_WITNESS` | Set to `1` when running the fixture monitor `mobile/test-support/watch-ssh-continuity` |
| `SPACEPORT_TEST_GHOSTTY_SESSION` | Fresh disposable session from `mobile/test-support/seed-attention` for the Ghostty agent-switching test |
| `SPACEPORT_TEST_ORDER_SESSION` | Separate session created by `mobile/test-support/seed-agent-order` for the live and saved agent ordering test |
| `SPACEPORT_TEST_DUO_SESSION` | Fresh session created by `mobile/test-support/seed-agent-order` for the iPhone Duo fold test |
| `SPACEPORT_TEST_GUTTER_SESSION` | Disposable SSH shell for testing the four side toolbar actions on an unfolded Duo |
| `SPACEPORT_TEST_PHONE_HEADER_SESSION` | Disposable SSH shell for testing terminal header actions on a standard iPhone |
| `SPACEPORT_TEST_OVERVIEW_SESSION` | Separate session from `mobile/test-support/seed-agent-order` for the Duo header and live sidebar counts |
| `SPACEPORT_TEST_PICKER_SESSION` | Fresh session from `mobile/test-support/seed-agent-order` for Go To search and Ghostty pane preview |
| `SPACEPORT_UI_IMAGE_SESSION` | Fresh disposable session with a POSIX shell for the screenshot paste workflow |
| `SPACEPORT_TEST_MARKS_SESSION` | Separate session from `mobile/test-support/seed-agent-order` for Home pin and snooze actions |
| `SPACEPORT_TEST_KEY_INSTALL_HOST` | Untrusted SSH fixture address for the standalone installer test |

```sh
cargo test --locked --manifest-path mobile/rust/Cargo.toml --test live -- --ignored
```

Give the UI test runner these same variables and a separate session for its keyboard round trip.
To exercise **Agents**, run `mobile/test-support/seed-attention NEW_DISPOSABLE_SESSION` on the disposable host with Herdr and jq installed.
Set `SPACEPORT_TEST_ATTENTION_SESSION` to that name. The test starts in Mission and replies to Review in Flight.
When running xcodebuild from a shell, prefix the variables with `TEST_RUNNER_`.

For the two-host checks, seed two disposable SSH endpoints with `mobile/test-support/seed-attention`.
Provide the second endpoint and its fingerprint using the same SSH variables with the prefix `SPACEPORT_TEST_SECOND_SSH_`.
Set `SPACEPORT_TEST_MULTIHOST_SESSION` and `SPACEPORT_TEST_SECOND_MULTIHOST_SESSION` to the two distinct session names.
The Rust live test `two_rosters_observe_separate_hosts_with_one_phone_key` enrolls one disposable signing key on both accounts.
For the simulator workflow, run `mobile/test-support/serve-multihost-roster CONTAINER USER SECOND_SESSION` for the second fixture.
Export the URL it prints to the XCTest runner. The workflow publishes an agent while both terminals are parked and checks its appearance on Home.
For the fold test, open an iPhone Duo simulator in Device Hub and unfold it.
Run `just test-ios-duo SIMULATOR_UDID` with the five SSH fixture variables and the Duo session set.
At the three Scout prompts, report the requested state from a second shell on the fixture host.
Set `session` to the disposable Duo session. Run one report per prompt, in the order shown:

```sh
scout=$(herdr --session "$session" agent list | jq -er '.result.agents[] | select(.agent == "Scout") | .pane_id')
report_scout() {
    herdr --session "$session" pane report-agent "$scout" --source mobile-qa --agent Scout --state "$1" --seq "$2"
}
report_scout blocked 2
report_scout working 3
report_scout blocked 4
```

Close the Duo when XCTest requests it, then open it at the next prompt.
The test sends a single reply to the fixture’s Reviewer pane across both transitions.
Close and reopen the Duo again when XCTest prompts with an unsent draft in the composer.
The test checks that the draft text and recipient survive both transitions.

Remove the disposable host after testing.

To check a changed SSH host key on the iOS simulator, start `mobile/test-support/serve-rotating-host` on the Mac.
Set the three `TEST_RUNNER_` variables it prints in the shell you use for Xcode.
Run the UI test `testChangedHostOpensDetailsWithoutReplacingPin`. Stop the fixture after the test.

### Verified development configuration

The iPhone 17 simulator on iOS 27.0 and iPad mini simulator on iOS 26.2 drove a real Ubuntu SSH host running Herdr 0.9.5.
Both entered shell input, read its output, enrolled a phone key and recovered the same shell after backgrounding.
A separate two-client test exercised tab selection, shared pane focus, terminal dimensions and stale-input rejection.
Version 0.1 build 1 ran on a physical device through TestFlight.
Build 2's iPhone simulator tests cover the split map, shared pane focus and multiline paste over SSH.
Build 3's iPhone SE simulator test jumped between workspaces and delivered a reply only to the waiting pane.

Build 5 passed 37 unit and integration tests plus four native workflows on the iPhone 17 simulator.
The Home agent jump and workspace picker also passed on the iPhone SE simulator.
The live SSH fixture confirmed a cross-workspace reply reached the chosen reviewer and neither neighboring terminal.
A separate OpenSSH fixture verified public-key installation without Herdr.

Build 8’s iPhone SE simulator switched agents with the keyboard open and delivered input over the same SSH connection.
A Mac Secure Enclave signed a fresh key-only login after enrollment on the open password connection.
The original terminal continued accepting input. A second process reopened the same device key and rejoined Herdr.

Build 12’s iPhone Duo simulator on iOS 27.2 folded and unfolded while Ghostty held a partially typed reply.
The selected agent received one complete reply. The keyboard stayed open and the SSH server recorded no new login.
A second fold and unfold preserved the unsent composer draft and its destination.
