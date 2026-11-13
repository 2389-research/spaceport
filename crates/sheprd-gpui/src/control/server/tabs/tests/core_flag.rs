// ABOUTME: Hides saved Core tabs without discarding their persisted references.
// ABOUTME: Refuses direct Core tab actions while ordinary Devices tabs remain available.
use super::*;
use crate::browser::CoreGroupKey;

#[gpui::test]
fn disabled_core_tabs_are_hidden_and_direct_actions_preserve_them(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let key = CoreGroupKey::new("hub-a", None, "spaceport", uuid::Uuid::from_u128(71)).unwrap();
        let mut store = Store::default();
        let core = store.open_core(&key, None).unwrap();
        let scope = Scope::local(Path::new("/tmp/private-core-flag.sock"));
        let bare = store
            .open(
                scope.clone(),
                "w0",
                Some(Location::CoreTerminal {
                    panes: crate::core_terminals::panes::Panes::single(
                        sheprd_backend::TerminalId::new("fixture-shell").unwrap(),
                    ),
                    observe: true,
                }),
                None,
            )
            .unwrap();
        let devices = store
            .open(scope, "w0", Some(Location::Devices), None)
            .unwrap();
        cx.set_global(store);
        let Response::Tab {
            reply: Reply::Tabs { tabs },
        } = list(cx)
        else {
            panic!("expected tabs");
        };
        assert_eq!(
            tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [u64::from(devices)]
        );
        for id in [core, bare] {
            for response in [
                focus(&Caller::default(), id.into(), cx),
                close(&Caller::default(), id.into(), cx),
            ] {
                assert!(
                    matches!(response, Response::Error { code: ErrorCode::Unsupported, message }
                    if message == crate::core_feature::disabled_message())
                );
            }
            assert!(cx.global::<Store>().get(id).is_some());
        }
        crate::core_feature::install(true, cx);
        let Response::Tab {
            reply: Reply::Tabs { tabs },
        } = list(cx)
        else {
            panic!("expected tabs");
        };
        assert!(tabs.iter().any(|tab| tab.id == u64::from(core)));
        assert!(tabs.iter().any(|tab| tab.id == u64::from(bare)));
    });
}
