# Build state isolation

Date: 2026-11-17. Scope: `bwm8` under `9s44`.
Doctor Biz authorized implementation and validation without commits or external writes.

## Cause and selected behavior

Every build previously used the same `spaceport` state directory.
Development runs could overwrite installed preview layouts and appearance stores.
Their control endpoints also collided, so CLI routing depended on launch order.

The existing compiled channel, profile, and worktree flag now select app state.
Installed previews retain the exact `spaceport` namespace.
Development builds use `spaceport-development-PROFILE`.
Linked worktrees append a stable hash of their compiled manifest directory.
Moving a linked worktree changes that development namespace.
Changing its branch or commit leaves the namespace stable.
The selector validates profile and namespace names before joining paths.
No existing state is moved, copied, or deleted.

All GUI stores and the app's control client/server use `preferences::state_dir`.
Diagnostics and crash handlers receive that selected path explicitly.
Local log and diagnostic commands use the same selection before GPUI starts.
The diagnostics crate no longer depends on the client crate for path discovery.
Configuration files retain their current configuration root.

Windows development builds hash the selected state path into their pipe names.
Installed previews retain their ordinary SID-only pipe when `XDG_STATE_HOME` is unset.
An explicit state root continues to isolate installed preview pipes as before.

## Verification

The executable log-path regression failed against the old implementation.
It reported the preview path from a development build.

- Client path tests: eight passed, including stable and distinct namespaces.
- Control endpoint identity tests: two passed, including the preview pipe contract.
- Diagnostics tests: 28 passed, with the crash subprocess fixture run by its parent test.
- Executable state tests: three passed, with the diagnostics producer fixture run by its parent test.
- The final executable rerun passed after the preview peer read one bounded request before replying.
- Headless native-harness diagnostic tests: two passed with the selected state-path helper.
- Native Clippy passed for client, control protocol, and diagnostics with all targets and features.
- Windows GNU Clippy passed for the same three crates with all targets and features.

The executable tests run the actual diagnostics writer in an isolated subprocess.
The actual CLI reads its serialized records from the selected build directory.
A separately written preview log remains byte-identical.
A real preview Unix listener remains unreachable from the development CLI.
Scripted CLI peers derive their expected path from the same compiled identity inputs.
No test contacts a personal app instance or daemon.

## Limits

Native Windows pipe execution remains pending Windows CI.
Windows GNU checks establish compilation and lint results, not Windows runtime behavior.
The pure namespace tests exercise preview selection without building a signed preview.
The existing full workspace and platform gates remain CI work.
The CLI build reports the existing `block 0.1.6` future-incompatibility notice.
