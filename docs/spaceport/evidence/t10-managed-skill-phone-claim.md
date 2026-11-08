# T10/RM-PHONE: managed skill command claim

At `f4a7edbb`, the app-embedded managed skill in
`skills/herdr-gpui-browser/SKILL.md` still instructed agents to run
`sheprd remote status` and `sheprd remote pair`. The CLI and control route
had already been removed; `crates/sheprd-gpui/src/agent_skill.rs` embeds
this source file for the app's managed copy. This was a reachable false
command claim even though it could not start the retired phone service.

The new `test_managed_skill_omits_removed_phone_commands` source policy test
failed on the live row. Removing that row made the test pass while the
retained `sheprd status` example remains. The other phone absence tests
still check the desktop startup/control and package inputs. This edit changes
the app-owned skill source only; it neither reads nor rewrites a copy already
installed in a person's home. Installed managed copies refresh under the
existing ownership rules when the app runs; no such refresh was exercised.

Source inventory found no `ios/` app files, iOS/watch workflow, `ios-*`
Justfile recipe, phone crate or helper in the current desktop release inputs;
the existing absence tests cover those paths. `docs/rebranding.md` labels
itself a historical upstream plan and notes removal. The inherited
`website/site/index.html`, `website/site/testflight/index.html` and
`website/site/assets/features.js` still advertise Sheprd phone remote
control, native iOS and TestFlight. `website/README.md` and
`website/netlify.toml` name the separate sheprd.org Netlify site; this is
not a qualified Spaceport release surface. Its ownership and current
publication state require a distinct T12 review, not a mobile build or
blind deletion in this cut. No site was deployed or contacted.

Checks: the focused new test failed before the row deletion, then the nine
phone/iOS/publication source tests passed. `just check-file-size`,
`just check-sorted`, the specification package integrity check and
`git diff --check` passed. No Cargo build, native desktop, mobile app,
package, signing or personal service check ran. A31 stays `not_run` until
native process/package/network and retained desktop IPC evidence exists;
A54 stays open. T10 and RM-PHONE remain in progress.

After integration at `0156081f`, the merged phone absence suite passed 6/6,
including the managed-skill regression. The package integrity check passed;
it does not inspect a user's installed skill copy or prove native startup.
