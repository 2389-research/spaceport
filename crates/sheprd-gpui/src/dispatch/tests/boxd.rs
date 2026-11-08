use super::*;
use crate::{
    cloud::CloudProvider,
    endpoint::Endpoint,
    sidebar::layout_tests::fixture_window,
    teleport::{self, FreshOrigin, Place},
};
use sheprd_client::ConnectTarget;

#[gpui::test]
fn boxd_dispatch_has_no_offer_or_provision_callback(cx: &mut gpui::TestAppContext) {
    assert!(
        crate::cloud::provision(CloudProvider::Boxd).is_none(),
        "no Boxd callback may create a machine for dispatch"
    );

    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            view.endpoints[0] =
                Endpoint::new("local".into(), "Local".into(), ConnectTarget::Local, true);
            let provider = CloudProvider::Boxd;
            view.endpoints.push(Endpoint::new(
                format!("{}:saved", provider.key()),
                "Saved".into(),
                ConnectTarget::Cloud {
                    provider,
                    account: "fixture".into(),
                    id: "saved".into(),
                    machine: "saved".into(),
                    session: "default".into(),
                },
                true,
            ));
            #[cfg(feature = "container")]
            {
                let provider = CloudProvider::Container;
                view.endpoints.push(Endpoint::new(
                    format!("{}:saved", provider.key()),
                    "Saved".into(),
                    ConnectTarget::Cloud {
                        provider,
                        account: "fixture".into(),
                        id: "saved".into(),
                        machine: "saved".into(),
                        session: "default".into(),
                    },
                    true,
                ));
            }
            let offered = view.dispatch_sandboxes();
            assert!(!offered.contains(&CloudProvider::Boxd));
            #[cfg(all(feature = "container", any(target_os = "macos", target_os = "linux")))]
            assert!(offered.contains(&CloudProvider::Container));
        })
    });

    let origin = FreshOrigin {
        place: Place {
            endpoint_id: "local".into(),
            label: "Local".into(),
            host: teleport::Host::new(&ConnectTarget::Local).unwrap(),
        },
        workspace_id: "workspace".into(),
        repo_key: "repo".into(),
        repo_label: "Repo".into(),
    };
    assert!(matches!(
        Job::sandboxed(CloudProvider::Boxd, origin, Then::Workspace { label: None }),
        Err(crate::Error::Remote(sheprd_remote::Error::Cloud(
            crate::cloud::Error::Unavailable(_)
        )))
    ));
}
