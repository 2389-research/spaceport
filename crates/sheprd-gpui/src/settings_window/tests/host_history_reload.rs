//! Reload on the Tabs page re-reads every SSH host, so a failed host retries.
use super::*;
use crate::herdr_settings::RemotePaneHistory;
use crate::settings_window::host_history::{History, Host, HostState, Target};
use sheprd_client::ConnectTarget;

#[gpui::test]
fn reload_on_tabs_rereads_a_failed_host(cx: &mut TestAppContext) {
    let source = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    let weak = cx.update(|cx| source.update(cx, |_, _, cx| cx.weak_entity()).unwrap());
    let reads = Arc::new(AtomicUsize::new(0));
    let counted = reads.clone();
    let (view, cx) = cx.add_window_view(move |_, cx| {
        let mut view = fixture_new(weak, cx);
        view.section = Section::Tabs;
        view.host_history.read = Arc::new(move |target, _| {
            let Target::Ssh(target) = target else {
                return Err(crate::Error::MissingHome);
            };
            // The first read fails like an unreachable host; later ones work.
            if counted.fetch_add(1, Ordering::SeqCst) == 0 {
                return Err(herdr_settings::Error::RemoteConflict.into());
            }
            Ok(History::Ssh {
                target: target.clone(),
                history: RemotePaneHistory::parse_text(Some(
                    "[experimental]\npane_history = true\n".into(),
                ))?,
            })
        });
        view
    });
    cx.update(|_, cx| {
        source
            .update(cx, |window, _, cx| {
                let endpoint = &mut window.endpoints[0];
                endpoint.connection.target = ConnectTarget::Ssh {
                    target: "box".into(),
                    session: "default".into(),
                };
                endpoint.live.status = crate::state::ConnectionStatus::Connected;
                cx.notify();
            })
            .unwrap()
    });
    view.update(cx, |view, cx| view.sync_host_history(false, cx));
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        let hosts = view.host_history.hosts();
        assert!(matches!(ssh_host(hosts).state, HostState::Failed(_)));
    });

    view.update(cx, |view, cx| view.reload_everything(fixture_load, cx));
    view.read_with(cx, |view, _| {
        assert!(matches!(
            ssh_host(view.host_history.hosts()).state,
            HostState::Loading
        ));
    });
    cx.run_until_parked();
    assert_eq!(reads.load(Ordering::SeqCst), 2);
    view.read_with(cx, |view, _| {
        assert!(matches!(
            &ssh_host(view.host_history.hosts()).state,
            HostState::Ready(history) if history.enabled()
        ));
    });
}

fn ssh_host(hosts: &[Host]) -> &Host {
    hosts
        .iter()
        .find(|host| host.target == Target::Ssh("box".into()))
        .unwrap()
}
