// ABOUTME: Exercises native marker drawing without borrowing its root view twice.
// ABOUTME: Checks owned-window cleanup when a headless renderer cannot return pixels.
use super::verify_status_shapes;
use gpui::TestAppContext;

#[gpui::test]
fn drawing_reaches_readback_and_removes_the_owned_window(cx: &mut TestAppContext) {
    for light in [false, true] {
        cx.update(|cx| {
            let windows = cx.windows().len();
            let Err(error) = verify_status_shapes(light, cx) else {
                panic!("no headless renderer configured");
            };
            assert!(error.to_string().contains("no HeadlessRenderer configured"));
            assert_eq!(cx.windows().len(), windows);
        });
    }
}
