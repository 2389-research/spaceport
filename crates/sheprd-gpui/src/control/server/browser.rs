//! Browser tab control actions.
use super::super::{
    main_windows, plain,
    protocol::{Caller, ErrorCode, Response},
};
use gpui::App;

/// Reloads every page the calling pane opened, in every window.
pub(super) fn reload(caller: &Caller, cx: &mut App) -> Response {
    let Some(pane) = caller.pane_id.as_deref().filter(|pane| plain(pane, 256)) else {
        return Response::error(
            ErrorCode::InvalidRequest,
            "browser reload runs in the Herdr pane that opened the page",
        );
    };
    let tabs: Vec<_> = cx
        .try_global::<crate::browser::Store>()
        .map(|store| store.opened_by(pane).map(|tab| tab.id).collect())
        .unwrap_or_default();
    for handle in main_windows(cx) {
        let _ = handle.update(cx, |view, _, _| view.reload_browser_tabs(&tabs));
    }
    Response::Reloaded { tabs: tabs.len() }
}
