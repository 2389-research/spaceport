// ABOUTME: Checks retained native tab targets and reserved retired identifiers.
// ABOUTME: Opts into Core before restoring and targeting saved Core references.
//! A saved tab file mixes retained Core/Devices IDs with retired IDs.
use super::*;
use crate::{browser::CoreGroupKey, core_terminals::set_document_for_test_as};
use sheprd_core_client::wire::Doc;

#[gpui::test]
fn restored_tabs_keep_exact_cli_targets_and_reserve_retired_ids(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| crate::core_feature::install(true, cx));
    let group = uuid::Uuid::from_u128(71);
    let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
    let (view, cx) = window(cx);
    let (core, devices, retired) = cx.cx.update(|cx| {
        set_document_for_test_as(cx, "hub-a", Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[{"id":group,"label":"Core"}]}),
            version: 3,
        });
        let mut store = Store::default();
        let core = store.open_core(&key, None).unwrap();
        let scope = crate::browser::scope(&view.read(cx).endpoints[0]);
        let devices = store.open(scope.clone(), "w0", Some(Location::Devices), None).unwrap();
        // The file names this tab as an old Orchestrator tab.
        let retired = store.open(scope, "w0", None, None).unwrap();
        let mut saved = serde_json::json!({"tabs":store.tabs().collect::<Vec<_>>(),"next":0});
        saved["tabs"][2]["location"] = serde_json::json!({"kind":"orchestrator","repo":{"checkout":"/src/old"}});
        let bytes = serde_json::to_vec(&saved).unwrap();
        cx.set_global(Store::restored_for_test(&bytes).unwrap());
        (core, devices, retired)
    });
    cx.cx.update(|cx| {
        let Response::Tab {
            reply: Reply::Tabs { tabs },
        } = list(cx)
        else {
            panic!("not tabs")
        };
        assert_eq!(
            tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [u64::from(core), u64::from(devices)]
        );
        assert_eq!(tabs[0].kind, TabKind::CoreTerminal);
        assert_eq!(tabs[1].kind, TabKind::Devices);
        for id in [retired, TabId::test(u64::from(retired) + 1)] {
            assert!(matches!(
                focus(&Caller::default(), id.into(), cx),
                Response::Error {
                    code: ErrorCode::NotFound,
                    ..
                }
            ));
            assert!(matches!(
                close(&Caller::default(), id.into(), cx),
                Response::Error {
                    code: ErrorCode::NotFound,
                    ..
                }
            ));
        }
        assert!(matches!(
            focus(&Caller::default(), core.into(), cx),
            Response::Done { .. }
        ));
        assert_eq!(view.read(cx).active_browser_key(cx).unwrap(), key.parts());
        assert!(matches!(
            focus(&Caller::default(), devices.into(), cx),
            Response::Done { .. }
        ));
        assert_eq!(view.read(cx).active_browser_key(cx).unwrap().1, "w0");
        let shown = view.read(cx);
        assert_eq!(
            shown.group_pick(shown.active_group().unwrap()),
            Some(crate::browser::Pick::Page(devices))
        );
        assert!(matches!(
            close(&Caller::default(), devices.into(), cx),
            Response::Done { .. }
        ));
        assert!(cx.global::<Store>().get(devices).is_none());
        assert!(cx.global::<Store>().get(core).is_some());
        assert!(matches!(
            close(&Caller::default(), core.into(), cx),
            Response::Done { .. }
        ));
        assert!(cx.global::<Store>().get(core).is_none());
        let next = Store::update(cx, |store| {
            store.open(Scope::endpoint("next"), "w", None, None)
        })
        .unwrap();
        assert!(u64::from(next) > u64::from(retired));
    });
}
