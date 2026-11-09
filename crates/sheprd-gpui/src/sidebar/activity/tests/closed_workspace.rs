use super::sections::{local, show};
use super::*;

#[gpui::test]
fn a_run_whose_workspace_closed_says_so_in_the_activity_view(cx: &mut gpui::TestAppContext) {
    let (view, cx) = local(cx);
    show(&view, cx, ActivityPanel::Issues);
    let id = cx.update(|_, cx| view.read(cx).activity.section_view.unwrap());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.orchestrator_events.push((
                id,
                crate::orchestrator::Event::OpenWorkspace {
                    host: None,
                    session: None,
                    workspace_id: "w404".into(),
                },
            ));
            view.poll_orchestrators(window, cx);
        })
    });
    let text = cx.update(|_, cx| {
        view.read(cx).orchestrators[&id]
            .view()
            .read(cx)
            .notice_text()
    });
    assert_eq!(text.as_deref(), Some("This run's workspace is closed"));
}
