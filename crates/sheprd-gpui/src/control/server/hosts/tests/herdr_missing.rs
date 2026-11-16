//! `spaceport status` and `hosts` say why Local is down when this computer
//! has no Herdr to start, rather than a bare "reconnecting".
use super::*;

#[gpui::test]
fn a_local_host_without_herdr_reports_it_is_not_installed(cx: &mut TestAppContext) {
    let (view, cx) = window(cx);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.endpoints[0].never_connect();
            view.endpoints[0].lose_herdr_for_test("Could not start herdr server");
            assert_eq!(view.control_hosts(cx)[0].status, "Herdr not installed");
        });
    });
}
