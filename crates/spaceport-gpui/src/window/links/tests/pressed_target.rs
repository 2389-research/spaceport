//! Pins what a press holds and what a click on it does: one typed target,
//! and the refusal each unopened kind explains.

use super::*;

const WEB: &str = "https://ex.com/a";
const OTHER: &str = "https://ex.com/b";

fn web(url: &str) -> PressedTarget {
    PressedTarget::Web(WebUrl::try_from(url).unwrap())
}

fn file(path: &str) -> PressedTarget {
    PressedTarget::File(FileLink {
        path: path.into(),
        line: None,
        pane_id: "w1:p1".into(),
        cwd: None,
        explicit: true,
    })
}

fn press(target: PressedTarget) -> PressedLink {
    PressedLink {
        target,
        position: point(px(1.), px(1.)),
    }
}

/// A press and its release compare the one typed target, so another address,
/// path, or kind is another link, and the same one is the same link.
#[test]
fn a_press_and_its_release_compare_the_typed_target() {
    assert!(press(web(WEB)).same_link(&press(web(WEB))));
    assert!(!press(web(WEB)).same_link(&press(web(OTHER))));
    assert!(press(file("notes.md")).same_link(&press(file("notes.md"))));
    assert!(!press(file("notes.md")).same_link(&press(file("other.md"))));
    // A target kind is never another kind, whatever the two read.
    assert!(!press(web(WEB)).same_link(&press(PressedTarget::Unopened(RowTarget::Malformed))));
    assert!(!press(file(WEB)).same_link(&press(web(WEB))));
}

/// Every typed target opens or refuses exactly what the press read, and a
/// kind a press files elsewhere opens nothing.
#[test]
fn a_click_opens_or_refuses_what_the_press_read() {
    assert!(matches!(
        web(WEB).click(),
        Some(ClickTarget::Web(url)) if url.as_str() == WEB
    ));
    assert!(matches!(
        file("notes.md").click(),
        Some(ClickTarget::File(link)) if link.path == "notes.md"
    ));
    assert!(matches!(
        PressedTarget::Unopened(RowTarget::External(
            ExternalUrl::try_from("mailto:name@example.com").unwrap()
        ))
        .click(),
        Some(ClickTarget::External(url)) if url.as_str() == "mailto:name@example.com"
    ));
    assert!(matches!(
        PressedTarget::Unopened(RowTarget::OtherHost("devbox".into())).click(),
        Some(ClickTarget::Unopened(
            crate::Error::LinkOnAnotherHost { .. }
        ))
    ));
    assert!(matches!(
        PressedTarget::Unopened(RowTarget::Malformed).click(),
        Some(ClickTarget::Unopened(crate::Error::MalformedLink))
    ));
    for never in [
        RowTarget::Web(WEB.into()),
        RowTarget::Path {
            path: "notes.md".into(),
            line: None,
            explicit: true,
        },
    ] {
        assert!(PressedTarget::Unopened(never).click().is_none());
    }
}
