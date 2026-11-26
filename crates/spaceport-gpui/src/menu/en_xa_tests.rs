//! The `en-XA` leak test (`crate::i18n::leak_tests`) for the popup shell's
//! own pages: the in-app menu, the version mismatch
//! notice, and the release notes sheet. Each draws only translated
//! messages, except for commands to run, host names, and the daemon's notes.
#![allow(clippy::unwrap_used)]

use super::{Page, VersionNotice, WorkspaceAction, version_mismatch::UPDATE_COMMAND};
use crate::{
    SpaceportWindow,
    i18n::leak_tests::{Recorder, assert_translated, draw, uncut, under_en_xa},
    sidebar::layout_tests::fixture_window,
};
use gpui::{Context, Entity, TestAppContext, VisualTestContext, Window, px, size};
use spaceport_client::{VersionMismatch, protocol::ClientShellSnapshot};
use spaceport_i18n::pseudo::is_marked;
use spaceport_menu::page_title::name;
use std::sync::Arc;

fn window<'a>(
    cx: &'a mut TestAppContext,
    recorder: &Recorder,
) -> (Entity<SpaceportWindow>, &'a mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.simulate_resize(size(px(1000.), px(800.)));
    draw(cx);
    recorder.take();
    (view, cx)
}

/// Opens a page with `open`, draws it, and checks what it added.
fn check(
    view: &Entity<SpaceportWindow>,
    cx: &mut VisualTestContext,
    recorder: &Recorder,
    surface: &str,
    data: &[String],
    open: impl FnOnce(&mut SpaceportWindow, &mut Window, &mut Context<SpaceportWindow>),
) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.dismiss_menu(window, cx);
            open(view, window, cx);
        })
    });
    draw(cx);
    assert_translated(surface, &uncut(recorder.take()), data);
}

#[test]
fn the_menu_and_its_own_pages_show_only_messages_under_en_xa() {
    under_en_xa("menu en-XA leak", |cx, recorder| {
        let (view, cx) = window(cx, recorder);
        check(&view, cx, recorder, "Menu", &[], |view, window, cx| {
            view.open_menu(window, cx);
        });
        for (host, mismatch) in [
            (
                None,
                VersionMismatch::DaemonOutdated {
                    server_version: Some("0.8.2".into()),
                },
            ),
            (
                Some("build box".to_owned()),
                VersionMismatch::DaemonOutdated {
                    server_version: None,
                },
            ),
            (
                Some("build box".to_owned()),
                VersionMismatch::ClientOutdated {
                    server_version: Some("2.0.0".into()),
                },
            ),
        ] {
            let surface = format!("Version mismatch {mismatch:?}");
            check(
                &view,
                cx,
                recorder,
                &surface,
                &[UPDATE_COMMAND.to_owned()],
                |view, window, cx| {
                    if view.open_menu(window, cx) {
                        view.menu.version_notice = Some(VersionNotice { host, mismatch });
                        view.menu.page = Some(Page::VersionMismatch);
                    }
                },
            );
        }
    });
}

#[test]
fn the_release_notes_sheet_shows_only_messages_under_en_xa() {
    under_en_xa("release notes en-XA leak", |cx, recorder| {
        let snapshot: ClientShellSnapshot = serde_json::from_str(include_str!(
            "../../../spaceport-protocol/tests/fixtures/endpoint-snapshot-v1.json"
        ))
        .unwrap();
        let (view, cx) = window(cx, recorder);
        let data = [
            snapshot.update_install_command.clone(),
            snapshot.release_notes.as_ref().unwrap().body.clone(),
        ];
        // The window shows the session first, so the sidebar's names are
        // in the frame before the sheet, not counted as the sheet's.
        view.update(cx, |view, cx| {
            view.live.snapshot = Some(Arc::new(snapshot));
            cx.notify();
        });
        draw(cx);
        recorder.take();
        check(
            &view,
            cx,
            recorder,
            "Release notes",
            &data,
            |view, window, cx| {
                if view.open_menu(window, cx) {
                    view.menu.page = Some(Page::Update);
                }
            },
        );
    });
}

/// A page's accessible name is a translated message too. Screen readers read
/// it rather than the frame, so the leak test's text recorder cannot see it.
#[test]
fn every_page_title_is_a_message_under_en_xa() {
    under_en_xa("page titles en-XA", |_, _| {
        let dialogs = [
            WorkspaceAction::Rename,
            WorkspaceAction::NewTab,
            WorkspaceAction::NewWorkspace,
            WorkspaceAction::Note,
        ]
        .map(Page::Dialog);
        let pages = [
            Page::About,
            Page::AddDevice,
            Page::AddWsl,
            Page::AgentSkill,
            Page::Application,
            Page::ConfirmClose,
            Page::CoreWorkspaceActions,
            Page::CoreWorkspaceCreate,
            Page::CoreWorkspaceDelete,
            Page::CoreWorkspaceRename,
            Page::Devices,
            Page::DeviceTags,
            Page::DeviceTheme,
            Page::ForwardPort,
            Page::Group,
            Page::Host,
            Page::Keybinds,
            Page::KillProcesses,
            Page::Menu,
            Page::MovePane,
            Page::MoveTab,
            Page::OpenLink,
            Page::Palette,
            Page::Pane,
            Page::PaneProcesses,
            Page::RemoveDevice,
            Page::RemoveWsl,
            Page::RenameDevice,
            Page::RenamePane,
            Page::RenameTab,
            Page::Tab,
            Page::Themes,
            Page::Update,
            Page::VersionMismatch,
            Page::Workspace,
        ];
        let leaks: Vec<_> = pages
            .into_iter()
            .chain(dialogs)
            .chain(crate::usage::registry::all().map(Page::Usage))
            .map(name)
            .filter(|name| !is_marked(name))
            .collect();
        assert!(leaks.is_empty(), "{leaks:#?}");
    });
}
