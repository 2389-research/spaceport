#![allow(clippy::unwrap_used)]
use super::*;
use crate::endpoint::tests::host;
use spaceport_client::ConnectTarget;

#[gpui::test]
fn old_container_catalog_stays_foreign_to_startup_devices(cx: &mut gpui::TestAppContext) {
    let state = tempfile::tempdir().unwrap();
    let path = state.path().join("cloud-devices.json");
    let bytes =
        br#"{"version":1,"devices":[{"provider":"container","account":"local","id":"old","machine":"saved-machine","session":"saved-session","label":"Old Container","unknown":{"keep":true}}]}"#;
    std::fs::write(&path, bytes).unwrap();
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    view.update(cx, |view, cx| {
        let ids = |view: &SpaceportWindow| {
            view.endpoints
                .iter()
                .map(|e| e.id.clone())
                .collect::<Vec<_>>()
        };
        view.reconcile_devices(vec![host("a", false)], Vec::new(), cx);
        assert_eq!(ids(view), [LOCAL, "ssh:a"]);
        assert_eq!(
            view.endpoints[1].connection.target,
            ConnectTarget::Ssh {
                target: "user@a".into(),
                session: "default".into()
            }
        );
        view.reconcile_devices(vec![], Vec::new(), cx);
        assert_eq!(ids(view), [LOCAL]);
    });
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
