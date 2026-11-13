// ABOUTME: Checks the installed preview title and conversation sample.
// ABOUTME: Keeps the synthetic preview free of upstream product guidance.
#![allow(clippy::unwrap_used)]
use super::*;

#[gpui::test]
fn the_installed_preview_names_spaceport(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let options = Options {
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
    };
    cx.update(|window, cx| {
        view.update(cx, |view, cx| install(view, window, cx, &options).unwrap())
    });
    let text = view.read_with(cx, |view, _| {
        let pane = view.chats.pane(PANE).unwrap();
        pane.session
            .transcript()
            .entries
            .iter()
            .filter_map(|entry| match &entry.kind {
                EntryKind::Agent { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    });
    assert!(text.contains("Bubbles"));
    assert_eq!(
        (cx.window_title(), text.contains("sheprd-labs/sheprd")),
        (Some("Spaceport · Rich chat preview".into()), false)
    );
}
