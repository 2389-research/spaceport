//! The `en-XA` leak test (`crate::i18n::leak_tests`) for the workspace
//! popover and the dialogs it opens: each draws only translated messages,
//! except for names: the workspace's and its branch.
use super::*;
use crate::i18n::leak_tests::{Recorder, assert_translated, draw, under_en_xa};
use gpui::{Entity, VisualTestContext, px, size};

/// The window with the sidebar fixture's workspaces, connected.
fn window(cx: &mut gpui::TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = sidebar::layout_tests::fixture_window(window, cx);
        let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        snapshot.workspaces = sidebar::layout_tests::snapshot(7).workspaces;
        view.live.status = crate::state::ConnectionStatus::Connected;
        view
    });
    cx.simulate_resize(size(px(1200.), px(900.)));
    draw(cx);
    (view, cx)
}

/// What the dialogs show that is not a message: every workspace's name and
/// branch, and what the dialog's own fields and previews hold.
fn names(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    cx.update(|_, cx| {
        let view = view.read(cx);
        let snapshot = view.live.snapshot.as_ref().unwrap();
        let workspaces = snapshot.workspaces.iter().flat_map(|workspace| {
            [Some(workspace.label.clone()), workspace.branch.clone()]
                .into_iter()
                .flatten()
        });
        let menu = &view.menu;
        let fields = [
            menu.input.as_ref().map(|input| input.text.clone()),
            // Errors keep their English until they are localized.
            menu.error.clone(),
            // The open worktree dialog's key hint.
            Some("ESC".to_owned()),
        ];
        workspaces.chain(fields.into_iter().flatten()).collect()
    })
}

fn popover(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext, workspace: &str) {
    let workspace = workspace.to_owned();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.reset();
            view.dismiss_menu(window, cx);
            view.open_workspace_menu(&workspace, Default::default(), window, cx);
        })
    });
    draw(cx);
}

fn check_popover(
    view: &Entity<SheprdWindow>,
    cx: &mut VisualTestContext,
    recorder: &Recorder,
    workspace: &str,
) {
    popover(view, cx, workspace);
    for retained in ["workspace-menu-Rename", "workspace-menu-Note..."] {
        assert!(
            cx.debug_bounds(retained).is_some(),
            "{workspace}: {retained}"
        );
    }
    for removed in [
        "workspace-menu-New worktree...",
        "workspace-menu-Close",
        "workspace-menu-Close group",
        "workspace-menu-Delete worktree checkout",
    ] {
        assert!(cx.debug_bounds(removed).is_none(), "{workspace}: {removed}");
    }
    let data = names(view, cx);
    assert_translated(
        &format!("Workspace popover on {workspace}"),
        &recorder.take(),
        &data,
    );
}

#[test]
fn the_workspace_popover_and_its_dialogs_show_only_messages_under_en_xa() {
    under_en_xa("workspace popover en-XA leak", |cx, recorder| {
        let (view, cx) = window(cx);
        recorder.take();
        check_popover(&view, cx, recorder, "w3");
        for (workspace, action) in [
            ("w3", WorkspaceAction::Rename),
            ("w3", WorkspaceAction::OpenWorktree),
            ("w3", WorkspaceAction::NewTab),
            ("w3", WorkspaceAction::NewWorkspace),
            ("w3", WorkspaceAction::Note),
        ] {
            popover(&view, cx, workspace);
            recorder.take();
            cx.update(|window, cx| {
                view.update(cx, |view, cx| {
                    view.open_workspace_dialog(action, window, cx)
                })
            });
            draw(cx);
            let data = names(&view, cx);
            assert_translated(&format!("{action:?} dialog"), &recorder.take(), &data);
        }
    });
}

// A separate test gives the linked checkout a fresh text cache. Its remaining
// translated labels are all shared with the main checkout's popover.
#[test]
fn the_linked_checkout_popover_shows_only_messages_under_en_xa() {
    under_en_xa("linked checkout popover en-XA leak", |cx, recorder| {
        let (view, cx) = window(cx);
        recorder.take();
        check_popover(&view, cx, recorder, "w4");
    });
}

#[test]
fn opening_an_external_link_asks_only_in_messages_under_en_xa() {
    under_en_xa("open link en-XA leak", |cx, recorder| {
        let (view, cx) = window(cx);
        recorder.take();
        let url = "zed://file/tmp/checkout";
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                let url = crate::terminal::ExternalUrl::try_from(url).unwrap();
                view.ask_to_open_link(url, window, cx);
            })
        });
        draw(cx);
        assert_translated("Open link", &recorder.take(), &[url.into()]);
    });
}
