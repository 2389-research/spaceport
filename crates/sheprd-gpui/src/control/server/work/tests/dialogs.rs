use super::*;
use crate::control::protocol::work::Script;

#[gpui::test]
fn scripts_refuse_what_the_menu_does_not_offer(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            connected(view);
            let run =
                |view: &mut SheprdWindow, workspace, kind, cx: &mut gpui::Context<SheprdWindow>| {
                    error_code(&flows::script(view, &caller(Some(workspace)), kind, cx))
                };
            assert_eq!(
                run(view, "w3", Script::Setup, cx),
                Some(ErrorCode::InvalidRequest)
            );
            assert_eq!(
                run(view, "w0", Script::Run, cx),
                Some(ErrorCode::InvalidRequest)
            );
            assert!(view.worktree_script.is_none());
        })
    });
}
