// ABOUTME: Runs the real rich-chat preview with isolated state and no agent.
// ABOUTME: Checks native rendering in light/dark themes and wide/narrow windows.
use super::*;

#[test]
#[ignore = "requires native display server; private preview, no daemon or focus assertion"]
fn native_preview_branding() {
    for (light, narrow) in [(false, false), (false, true), (true, false), (true, true)] {
        native_fixture(Fixture::Preview { light, narrow });
    }
}
