//! A saved tab file mixes live Core/page IDs with retired Orchestrator IDs.
use super::*;
use crate::{browser::CoreGroupKey, core_terminals::set_document_for_test_as};
use sheprd_core_client::wire::Doc;

#[gpui::test]
fn restored_tabs_keep_exact_cli_targets_and_reserve_retired_ids(cx: &mut gpui::TestAppContext) {
    let group = uuid::Uuid::from_u128(71);
    let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
    let (view, cx) = window(cx);
    let (core, page, retired) = cx.cx.update(|cx| {
        set_document_for_test_as(cx, "hub-a", Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[{"id":group,"label":"Core"}]}),
            version: 3,
        });
        let mut store = Store::default();
        let core = store.open_core(&key, None).unwrap();
        let scope = crate::browser::scope(&view.read(cx).endpoints[0]);
        let page = store.open(scope.clone(), "w0", Some(web("https://a.test/")), None).unwrap();
        let retired = store.open(
            scope,
            "w0",
            Some(Location::RetiredOrchestrator { repo: RetiredOrchestratorRepo::new("/src/old".into()).unwrap() }),
            None,
        ).unwrap();
        let bytes = serde_json::to_vec(&serde_json::json!({"tabs":store.tabs().collect::<Vec<_>>(),"next":0})).unwrap();
        cx.set_global(Store::restored_for_test(&bytes).unwrap());
        (core, page, retired)
    });
    cx.cx.update(|cx| {
        let Response::Tab {
            reply: Reply::Tabs { tabs },
        } = list(false, cx)
        else {
            panic!("not tabs")
        };
        assert_eq!(
            tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [u64::from(core), u64::from(page)]
        );
        let Response::Tab {
            reply: Reply::Tabs { tabs },
        } = list(true, cx)
        else {
            panic!("not pages")
        };
        assert_eq!(
            tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [u64::from(page)]
        );
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
            focus(&Caller::default(), page.into(), cx),
            Response::Done { .. }
        ));
        assert_eq!(view.read(cx).active_browser_key(cx).unwrap().1, "w0");
        let shown = view.read(cx);
        assert_eq!(
            shown.group_pick(shown.active_group().unwrap()),
            Some(crate::browser::Pick::Page(page))
        );
        assert!(matches!(
            close(&Caller::default(), page.into(), cx),
            Response::Done { .. }
        ));
        assert!(cx.global::<Store>().get(page).is_none());
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
