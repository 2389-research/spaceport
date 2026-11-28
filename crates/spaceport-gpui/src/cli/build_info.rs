// ABOUTME: Embeds the signed-channel identity that packaging reads without running the app.
// ABOUTME: Returns the worktree record that the command line prints.
pub fn build_info() -> &'static str {
    // Keep packaging metadata in the bytes without changing --build-info output.
    std::hint::black_box(concat!(
        "\0SPACEPORT_PREVIEW_V1\nid=",
        env!("SPACEPORT_APP_ID"),
        "\nversion=",
        env!("SPACEPORT_APP_VERSION"),
        "\nprofile=",
        env!("SPACEPORT_BUILD_PROFILE"),
        "\nchannel=",
        env!("SPACEPORT_BUILD_CHANNEL"),
        "\n\0"
    ));
    const PREFIX_LEN: usize = "\0HERDR_BUILD_IDENTITY_V1\n".len();
    const RECORD: &str = concat!(
        "\0HERDR_BUILD_IDENTITY_V1\n",
        "worktree=",
        env!("HERDR_BUILD_WORKTREE"),
        "\n",
        "branch=",
        env!("HERDR_BUILD_BRANCH"),
        "\n",
        "pr=",
        env!("HERDR_BUILD_PR"),
        "\n\0"
    );
    let record = std::hint::black_box(RECORD);
    &record[PREFIX_LEN..record.len() - 1]
}
