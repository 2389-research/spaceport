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

1. Open **Add host**. Enter the SSH hostname, port, username, and Herdr session. Tap **Connect**.
2. Compare the displayed SSH fingerprint with the host administrator’s copy before trusting it.
3. For password sign-in, enter the account password. Open **Session → Phone key → Install key** to enroll the device.
   Spaceport installs the public key through the password session and verifies a fresh key-only SSH login.
4. For an account that requires a key, open **Phone key → Copy public key** at the password prompt.
   Authorize that public key for the SSH account, then tap **Use phone key**. The P-256 private key stays in Android Keystore.
   Key login requires Android 9 or later and secure hardware. The phone must be unlocked to sign.
   Android 8 and devices without a hardware-backed key support password sign-in.
5. Tap **Go To** to select an agent or workspace. Android draws Herdr’s terminal cells on a native Canvas.
   The iOS Ghostty experiment uses a separate renderer.

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
4. Sign in. Tap the current location in the bottom toolbar to browse workspaces, tabs, and panes.
5. Tap the terminal to type. Input goes to the displayed pane or popup.

Home puts pinned agents first, followed by **Waiting for you** and **Working**. Idle agents follow in three approximate age bands: under an hour, one to five hours, and five hours or longer.
Agents first found idle appear under **Idle**, newest Herdr sequence first within each session. Once Spaceport confirms the same idle streak for an hour, its row shows a lower bound such as `2h+`.
Home measures idle time through the last confirmed observation. A cached session keeps that age until it reconnects.
Home and Go To show each agent’s terminal title above its name and location. Home keeps the last title observed for that session.
Tap an agent to reopen its session. Spaceport checks the daemon boot, workspace, tab, pane, and reported agent name before enabling input.
Choose **Workspaces** underneath the agent list to browse a saved session’s live workspaces.
Saved sessions appear next, grouped by host.
The time beside an activity count tells you when the phone last observed it.
Open Host Detail using the info button to add sessions or manage the host’s sign-in and trust.

Opening a named session runs the installed Herdr bridge. Herdr starts the session if needed.
Disconnecting closes the client's SSH channel. Herdr continues to own its terminals and agents.
Native chat is deferred. Agent prompts and approvals use their original terminal.

## Try Ghostty on iOS

Turn on **Settings → Terminal Experiment → Use Ghostty**, then open an agent or terminal.
The native Agents and Workspaces lists still use the existing session connection.
Spaceport resolves the selected pane in that named session and runs Herdr’s direct attach on a second SSH channel.
Switching panes releases the old channel. Herdr handles input ownership without a forced takeover.
Popups keep the existing cell display. Turn off the setting to use that display for every pane.

The app embeds the pinned Ghostty Apple package from [libghostty-spm](https://github.com/Lakr233/libghostty-spm).
Xcode resolves its Swift sources and binary framework when building the app. The remote host still needs only SSH and Herdr.
The app hosts the package’s UIKit view directly. Its SwiftUI state wrapper crashed during synchronous teardown on the iOS 26.2 simulator.
When signing a manual archive, pass the app profile with `SPACEPORT_MOBILE_PROFILE`. The Ghostty resource bundle has its own signing settings.

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

On Home, swipe an agent right to pin it or left to snooze it for an hour.
Touch and hold for other snooze lengths or to wake it early.
Pinned agents lead the list. Snoozed agents appear at the bottom until the deadline,
or until Spaceport observes them change to working or waiting for you.
These settings stay on this phone and belong to the observed agent in its current Herdr boot.

On the open iPhone Duo, a wide session shows an agent sidebar beside the terminal.
**Waiting for you** comes first, then working and idle agents. Workspaces follow the agents.
Use the sidebar button to hide the list, or tap the terminal’s location to bring it back.
Use the magnifying glass to search panes or open terminal history.
A narrow window gives the terminal the full width. Folding or resizing keeps the selected pane and font size.
On the narrowest layouts, Escape stays visible and **More terminal keys** holds the other controls.

In the compact layout, tap the location in the terminal toolbar to open Go To.
In the Go To sheet, working agents appear first, followed by agents that need you. Idle and done agents follow, most recently changed first.
Go To uses Herdr’s own change order within the connected session. Workspace, tab, and pane navigation follow the list.
Search Go To by agent, terminal title, pane, workspace, or tab. Words can match across fields.
The search uses the connected session’s roster. Recent visits break ties between equally strong matches.
Touch and hold an agent or search result to peek without moving focus.
Peek shows the current screen, a dated excerpt from a previous visit, or the pane’s details if no excerpt is available.
These excerpts stay in memory and clear when the session parks or disconnects.
Tap a waiting pane to jump to it. Input resumes when Herdr confirms the destination and its terminal surface.
For split tabs, tap a tile on the pane map or choose a pane by name.
Opening Go To leaves focus in place. Dismissing it restores the keyboard if it was open.
Clients on the destination tab share its selected pane.

In Go To, tap **Find in Terminal** to search the focused pane’s retained Herdr history.
Find opens the newest match with its surrounding text. **Older** and **Newer** wrap between matches.
**Copy match** copies the terminal’s original text. **Done** returns to the live terminal at its current position.
Ghostty and the cell renderer use the same host history.

Tap **Compose** to edit a multiline draft on the phone. Paste it into the named terminal when ready.
Copy a screenshot, then tap **Paste** beside **Screenshot**. Review its thumbnail, add an optional note, and confirm the destination.
Spaceport stages the PNG in a private temporary directory on that SSH host. The terminal receives its path followed by your note.
The host confirms the file before Spaceport pastes its path. Accepted files stay there until you or the host clears them.
On a fresh install, **Send** pastes and presses Return. Turn off **Press Return After Pasting** to use **Paste**.
The switch shares its saved value with **Settings → Press Return After Pasting**.
Send queues the paste and Return together for the same destination.
Each retained session keeps its unsent draft in memory.
If the destination changes, choose the new destination explicitly or switch back to the original pane.

Set the terminal text size and choose your toolbar keys in Settings. Swipe the keys for more controls. Drag vertically on the terminal to scroll.
Each window keeps its two most recently opened sessions in memory. Go Home or switch agents to reuse the SSH login.
On return to a parked session, Spaceport checks the existing SSH connection and waits for Herdr before enabling input.
If the active session loses its SSH transport or the host stops answering the resume check, Spaceport tries one fresh connection.
It checks the host key again. A password cleared by leaving for Home must be entered again.
A refused resume or an interrupted agent switch requires a fresh selection. Spaceport never replays terminal input.
Choose **Disconnect** in Go To to close the connection and discard its local draft.

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
| `SPACEPORT_TEST_ATTENTION_SESSION` | Separate seeded session for the cross-workspace UI test |
| `SPACEPORT_TEST_LIFECYCLE_SESSION` | Fresh disposable session name for testing drafts and transport recovery |
| `SPACEPORT_TEST_SSH_WITNESS` | Set to `1` when running the fixture monitor `mobile/test-support/watch-ssh-continuity` |
| `SPACEPORT_TEST_GHOSTTY_SESSION` | Fresh disposable session from `mobile/test-support/seed-attention` for the Ghostty agent-switching test |
| `SPACEPORT_TEST_ORDER_SESSION` | Separate session created by `mobile/test-support/seed-agent-order` for the live and saved agent ordering test |
| `SPACEPORT_TEST_DUO_SESSION` | Fresh session created by `mobile/test-support/seed-agent-order` for the iPhone Duo fold test |
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
For the fold test, open an iPhone Duo simulator in Device Hub and unfold it.
Run `just test-ios-duo SIMULATOR_UDID` with the five SSH fixture variables and the Duo session set.
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
