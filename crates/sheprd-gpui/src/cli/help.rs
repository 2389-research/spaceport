// ABOUTME: Builds the shared top-level help for the GUI and control command entry points.
// ABOUTME: Keeps launch options and control options visible without starting the app.
pub(crate) fn usage() -> String {
    let mut text = String::from(
        "spaceport [--socket CLIENT_SOCKET]\nConnects to Local and saved SSH hosts; never installs remote software.\nStarts the local Herdr daemon if needed; never stops it.\nExplicit --socket targets are attach-only and isolate the GUI to one existing daemon.\n  --build-info        Print the executable's build identity without starting the GUI\n\n",
    );
    text.push_str(&crate::control::command_usage(None));
    #[cfg(feature = "integration-test")]
    text.push_str("\n\n  --integration-test  Run native GUI checks (requires explicit --socket)\n  --sidebar-test      Run native sidebar fixtures without connecting to a daemon\n  --performance-test  Measure native dense-terminal hover/scroll without a daemon (macOS/Windows)");
    text
}
