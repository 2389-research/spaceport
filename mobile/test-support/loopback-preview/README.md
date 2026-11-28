# Mobile browser preview trial

Debug builds can open one HTTP service on the connected SSH host. In the iOS agent picker, choose **Preview loopback service**. Enter the port. Spaceport opens the system browser. Return to the same menu and choose **Stop service preview** to close the listener and every browser connection.

The listener binds an ephemeral IPv4 loopback port. Each browser socket opens a direct TCP channel to the selected port on the remote IPv4 loopback address. The SSH server must permit TCP forwarding. The client limits each preview to eight browser sockets. Any local process that knows the port can reach it while the preview runs.

## Reproduce

Copy the two HTML files in this directory into a disposable SSH account. Serve them from that directory with `python3 -m http.server --bind 127.0.0.1 0`. Read the chosen port from its output. A production host needs only its existing web application and ordinary SSH.

Use the existing mobile SSH fixture variables. For the Rust live test, also set `SPACEPORT_TEST_SERVICE_PORT`. For the iOS workflow, set `SPACEPORT_TEST_SERVICE_PORT` and `SPACEPORT_TEST_SERVICE_SESSION` in the test runner. Finish system browser onboarding before running UI automation.

| Check | What it exercises |
| --- | --- |
| Rust live test `chosen_service_survives_park_and_closes_on_stop` | Reads the selected service before and after parking. Rejects a second preview. Checks that Stop closes an existing socket and immediately refuses new connections. |
| iOS `testLoopbackServiceInSafari` | Opens Safari, follows a link, returns to Spaceport, reopens the agent picker, and stops the preview. |

## Release decision

Keep the control in debug builds. The November 26 simulator and emulator trials confirmed short browser handoffs on iOS 26.2 (Safari) and Android 16 / API 36 (Chrome). They used an OpenSSH host with an HTTP service bound to loopback. The fixture returned HTTP 200 for two distinct pages while each browser was in front. Safari requested the second page one second after the first. Chrome also fetched it nine seconds after the first page during a manual tap trial. Stop closed both new and in-flight connections in the Rust live test.

These trials cover a quick switch to another app. Physical-device suspension, memory pressure, screen lock and sustained browsing still need qualification before offering this as a release feature. The prototype requests no background execution exemption. Relaunching Spaceport requires starting a fresh preview.
