# Prompt for the Spaceport fork agent

You are implementing **Spaceport**, an independent, fully rebranded desktop fork of Sheprd/herdr-gpui. The specification package beside this prompt is authoritative. Keep it intact at `docs/spaceport/` in the fork and integrate a pointer into the repository's root agent instructions without erasing applicable upstream rules. Run its tools from the package root; candidate code paths refer to the application repository. Read `AGENTS.md`, `SPEC.md` and `SYSTEM.md`, then run the package checks and obtain the T00 context packet.

The user likes the existing desktop UX. Preserve it by default. Remove IDE/diff/Git/GitHub/embedded-browser/cloud-provisioning/Teleport/orchestration/mod/scheduler features, terminal annotations, legacy Herdr 1 and the mobile remote-control/push service completely. Keep terminals, rich chats, workspaces, existing UI customization, file interactions, required service forwarding, notifications, diagnostics and the local desktop CLI, subject to the exact capability/ownership contracts.

Use a separately installed **Herdr Core** as the only runtime and machine-connection owner. Tailscale is external. Do not bundle/install/update Core or edit its repository. Do not implement another transport manager, scheduler, session recovery engine, provider configuration system or mobile server. Do not change the terminal engine or start mobile/UniFFI work.

The spec corrects earlier assumptions: Core already owns remote machine connections; Core does not supply the old daemon's native workspace semantics; rich chat depends on a host-side adapter and remote native parity must be proved. Use Core documents for explicit presentation metadata as specified. Do not invent backend RPCs or claim that an untested adapter is supported.

Begin with T00: verify checkout safety and source revisions, build an isolated reference, inventory dependencies, inspect actual Core methods/features and record capability gaps. Then follow `plan/tasks.json`, using task-sized context packets. Do not start with a bulk rename or deletion of the legacy client crate. Reuse shared types and existing command/control infrastructure where they fit.

For each bounded change: reproduce, implement, test, inspect the diff, update task/acceptance evidence, and preserve one useful regression or contract clarification. Prefer source/schema/structured observations over screenshots or guessed state. Do not blindly retry uncertain side effects. Keep permissions and exact target identity intact.

Do not ask the user to repeat settled choices. If a required backend route is missing, record the smallest precise external dependency, mark the affected task blocked, and continue independent work. Do not silently ship a smaller product or cross the client/runtime boundary to hide a gap.

Deliver code commits, retained/excluded feature evidence, capability matrix, native-platform results, release identity audit, current documentation and a concise list of any unresolved gates. A passing specification validator or compilation is not evidence that the application works. Do not publish or mutate real user environments without explicit authorization.
