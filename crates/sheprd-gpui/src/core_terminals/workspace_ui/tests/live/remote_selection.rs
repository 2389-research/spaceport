use super::*;
use crate::browser::{CoreGroupKey, Location, Store};
use sheprd_client::ConnectTarget;

#[gpui::test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn local_member_opens_with_a_remote_legacy_endpoint_selected(cx: &mut gpui::TestAppContext) {
    let sandbox = Sandbox::start();
    let core = sandbox.connect();
    let core_id = core.hello().core.id.clone();
    let group = Uuid::from_u128(91);
    let created = edit::apply(
        &core,
        edit::read(&core).unwrap(),
        &Edit::Create {
            id: group,
            label: "Local".into(),
        },
    )
    .unwrap();
    let argv = ["sh".to_owned(), "-c".to_owned(), "sleep 120".to_owned()];
    let terminal = core
        .run(&Run {
            argv: &argv,
            ..Run::default()
        })
        .unwrap();
    edit::apply(
        &core,
        created,
        &Edit::AddMember {
            id: group,
            core_id: core_id.clone(),
            terminal_id: terminal.address.clone(),
        },
    )
    .unwrap();
    let backend = CoreBackend::new(sandbox.connect(), 1000);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|_, app| {
        view.update(app, |view, _| {
            view.core_terminals
                .hub
                .set_backend_for_test(backend.clone())
        });
        if !app.has_global::<observation::Observation>() {
            observation::Observation::install(app);
        }
        app.global_mut::<observation::Observation>().ensure(backend);
    });
    wait_document(cx, 2);
    cx.update(|window, app| {
        Store::update(app, |_| ());
        view.update(app, |view, app| {
            view.endpoints.push(crate::endpoint::Endpoint::new(
                "ssh:fixture".into(),
                "Fixture".into(),
                ConnectTarget::Ssh {
                    target: "fixture".into(),
                    session: "default".into(),
                },
                true,
            ));
            view.selected_endpoint = 1;
            view.select_core_workspace(group, app);
            assert!(view.selected_is_remote());
            view.open_core_workspace_member(group, &core_id, &terminal.address, window, app);
            let key = CoreGroupKey::new(
                &core_id,
                None,
                crate::core_terminals::workspace_document::NAME,
                group,
            )
            .unwrap();
            assert!(
                app.global::<Store>()
                    .in_core_group(&key)
                    .any(|tab| { matches!(tab.location, Some(Location::CoreTerminal { .. })) })
            );
        });
    });
}
