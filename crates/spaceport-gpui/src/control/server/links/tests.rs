use super::*;
use crate::{control::protocol::Caller, sidebar::layout_tests::fixture_window};
use gpui::TestAppContext;

fn link(url: &str) -> Call<LinkOp> {
    Call {
        caller: Caller::default(),
        op: LinkOp { url: url.into() },
    }
}

fn code(response: &Response) -> Option<ErrorCode> {
    match response {
        Response::Error { code, .. } => Some(*code),
        _ => None,
    }
}

#[gpui::test]
fn links_are_parsed_again_and_only_show_things(cx: &mut TestAppContext) {
    cx.add_window(fixture_window);
    cx.update(|cx| {
        // Any local process can send one, so the app checks it too.
        for url in [
            "spaceport://run?command=calc",
            "spaceport://open?path=/checkout",
            "https://example.com",
        ] {
            assert_eq!(
                code(&serve(link(url), cx)),
                Some(ErrorCode::InvalidRequest),
                "{url}"
            );
        }
        assert_eq!(
            code(&serve(link("spaceport://device?id=ssh:nowhere"), cx)),
            Some(ErrorCode::NotFound)
        );
        assert!(matches!(
            serve(link("spaceport://device?id=local"), cx),
            Response::Done { .. }
        ));
        // Posted by no window of this app.
        assert_eq!(
            code(&serve(
                link("spaceport://notification?tag=herdr:local:boot:w1:p1"),
                cx
            )),
            Some(ErrorCode::NotFound)
        );
    });
}

#[gpui::test]
fn a_device_link_keeps_the_caller_window_target(cx: &mut TestAppContext) {
    let first = cx.add_window(fixture_window);
    cx.add_window(fixture_window);
    let mut call = link("spaceport://device?id=local");
    call.caller.window = Some(first.window_id().as_u64());
    cx.update(|cx| assert!(matches!(serve(call, cx), Response::Done { .. })));
    let mut missing = link("spaceport://device?id=local");
    missing.caller.window = Some(u64::MAX);
    cx.update(|cx| assert_eq!(code(&serve(missing, cx)), Some(ErrorCode::NotFound)));
}
