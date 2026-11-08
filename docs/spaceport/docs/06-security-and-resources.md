# Security, failure containment and resource ownership

Implements SP-005–SP-006, SP-012–SP-014, SP-018–SP-023 and SP-025.

## Boundaries

Tailscale supplies network connectivity; it is not application consent, a substitute for SSH host verification, or permission to trust content. Core controls runtime access; the OS protects the private local client endpoint. A capability advertises what can be done, not who is authorized to do it.

The retained local control socket/pipe must be restricted to the owning OS user and validate its peer/endpoint using the retained platform policy. This is a same-user control surface, not a sandbox against arbitrary code already running as that user. Do not claim stronger isolation. Never expose it through a phone server, generic TCP bind, browser bridge or tailnet listener.

## Untrusted data stays data

Terminal escape sequences, OSC metadata, chat Markdown, tool output, filenames, repository files, remote status, document labels and links are untrusted. They cannot set new executable paths, request provider credentials, grant confirmations, mutate command catalog definitions or alter update endpoints. Render text safely, bound lengths/images and preserve existing URL/clipboard restrictions. Tool-output instructions are not user requests.

Opening a file or URL is an explicit outward action. Do not fetch external images or follow local file paths simply because remote text references them. Preserve safer upstream behavior when adapting rich content. Generic protocol fields must not bypass typed argument validation or safe process invocation.

## Process lifecycle

Spaceport owns only its UI, workers and deliberately short-lived support processes. Core and its terminal/agent programs are not children to be swept up by application shutdown. On quit, cancel subscriptions and client-owned transfers, close IPC and exit; do not invoke Core stop/update or agent termination. Tests must prove actual platform process survival, not just that a kill function was not called.

A user-confirmed terminal kill is distinct and precisely targeted. Chat view conversion/resume code that manipulates an agent merely to achieve a layout is removed. A Core keep/recovery policy is backend behavior; Spaceport must not recreate it.

## Credentials and executable selection

Do not read cloud/GitHub/model tokens to populate a usage panel. Do not copy credentials between upstream and fork identities. Existing SSH agents/key stores remain the user's setup. Resolve the installed Core binary from a trusted configured/system location, present its identity/version, and do not pick `./hc` from an untrusted workspace. Never pass remote content through a local shell to construct commands.

Machine onboarding is non-installing and non-upgrading. Do not auto-accept changed host keys or downgrade authentication to make a test pass. Fixtures use separate known-host files and explicit test identities.

## Transfers and forwarding

Distinguish read, replace, append, attach and execute. Preserve overwrite confirmations and target-machine/path identity; a remote path is not a local path. Bound data, validate content type where rendering depends on it, and do not turn a received path into automatic execution. Verify symlink semantics against Core before advertising safe replacement; a client preflight alone cannot eliminate a backend race.

A service forward must have an explicit endpoint/lifetime/owner and a verified route. Do not expose a remote loopback service by rebinding it publicly. Quitting a client must not stop the service itself. If forwarding is Core-owned, accurately describe its actual lifetime; do not assume it ends with the client. No secret second remote connection manager.

## Logging and inspection

Default diagnostics contain operation IDs, qualified nonsecret identities, error classes, versions and bounded timing/state transitions—not prompts, full file contents, tokens, environment dumps or conversation text. Optional diagnostic export is explicit, inspectable and redacted. Never automatically upload it. Panic/crash artifacts require the same attention to secrets.

No offline content cache or durable action queue. Preferences and view references may persist. Logs rotate and bound storage. A bounded in-process receipt buffer is not a replay mechanism; after process loss, unknown effects are reconciled from backend truth rather than invented history.

## Resource accounting

Every queue, parser buffer, transcript page, image, document, request, subscription and retry loop has an owner, bound, cancellation path and overload outcome. Keep control and bulk-data capacity separate. Apply backpressure before allocation where possible. Coalesce repeated display invalidations, not arbitrary terminal bytes or permission requests.

A discarded view cancels its expensive subscriptions unless another view uses them. Multi-window metadata uses shared subscriptions. Permission and completion events are not silently dropped to save resources; on a stream gap, refresh state and report coverage limits. Retain stricter upstream parser/attachment bounds rather than replacing them with larger global defaults.

## Updates and supply chain

Use independent Spaceport update identities and verified signed manifests/artifacts. Reject tampered manifests, wrong app identity and untrusted keys. Updates must not replace Core or upstream app installations. Require explicit owner-controlled production configuration; do not ship development credentials/placeholders. Record retained licenses/dependencies and remove unused service/push bundles from installers.
