// ABOUTME: Groups tests of the real synthetic rich-chat installer.
// ABOUTME: Exercises the same title and transcript producers as native previews.
use super::*;
mod accessibility;
mod branding;
mod composer;

fn conversation_options() -> Options {
    Options {
        attach: vec![],
        case: "conversation".into(),
        capture: None,
        draft: None,
        picture: None,
        hold: Duration::ZERO,
        layout: ChatLayoutMode::default(),
        light: false,
        max_width: None,
        narrow: true,
        read_only: false,
        view: false,
        working: false,
    }
}
