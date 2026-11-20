// ABOUTME: Protects active Core edit targets and accepted completion ownership from passive paint.
// ABOUTME: Exercises detached controls with private framed Core peers and bounded channels.
use super::super::Pending;
#[cfg(unix)]
use super::super::WorkspacePaint;
#[cfg(unix)]
use super::super::paint::ActionsPaint;
use super::*;
#[cfg(unix)]
use crate::{SheprdWindow, dialog_input::DialogInput, fonts::StyledFont};
use gpui::TestAppContext;
#[cfg(unix)]
use gpui::{Context, Entity, Modifiers, Render, VisualTestContext, Window, div, prelude::*, px};

fn observe(cx: &mut gpui::App, id: Uuid, label: &str, version: u64) {
    observation::set_document_for_test_as(
        cx,
        "fixture-core",
        Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[{"id":id,"label":label}]}),
            version,
        },
    );
}

#[cfg(unix)]
fn open(
    source: &mut SheprdWindow,
    page: Page,
    id: Uuid,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) {
    match page {
        Page::CoreWorkspaceActions => {
            source.open_core_workspace_actions(id, Default::default(), window, cx)
        }
        Page::CoreWorkspaceCreate => {
            source.open_core_workspace_dialog(window, cx);
            source.menu.input = Some(DialogInput::new("Original draft".into()));
        }
        Page::CoreWorkspaceRename => source.open_core_workspace_rename(id, window, cx),
        Page::CoreWorkspaceDelete => source.open_core_workspace_delete(id, window, cx),
        _ => unreachable!(),
    }
}

#[cfg(unix)]
struct Frozen {
    source: Entity<SheprdWindow>,
    paint: WorkspacePaint,
    _peer: std::os::unix::net::UnixStream,
}

#[cfg(unix)]
impl Render for Frozen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.source.update(cx, |source, cx| {
            div()
                .w(px(460.))
                .h(px(700.))
                .text_font(&source.config.ui)
                .text_size(px(source.config.ui.size))
                .line_height(px(source.config.ui.line_height()))
                .child(source.render_core_workspace_paint(&self.paint, cx))
        })
    }
}

#[cfg(unix)]
#[gpui::test]
fn frozen_core_controls_cannot_edit_or_dismiss_a_replacement_without_a_parent_shield(
    cx: &mut TestAppContext,
) {
    for page in [
        Page::CoreWorkspaceActions,
        Page::CoreWorkspaceCreate,
        Page::CoreWorkspaceRename,
        Page::CoreWorkspaceDelete,
    ] {
        let (view, cx) = cx.add_window_view(|window, cx| {
            let source = cx.new(|cx| sidebar::layout_tests::fixture_core_window(window, cx));
            let (backend, peer) =
                crate::core_terminals::tests::scripted_core::connect("fixture-core");
            let paint = source.update(cx, |source, cx| {
                source.core_terminals.hub.set_backend_for_test(backend);
                observe(cx, Uuid::from_u128(71), "Original group", 3);
                open(source, page, Uuid::from_u128(71), window, cx);
                if page == Page::CoreWorkspaceActions {
                    assert!(matches!(
                        source.prepare_core_workspace_actions(cx),
                        ActionsPaint::Ready { .. }
                    ));
                } else {
                    assert!(
                        source.prepare_core_workspace_dialog(cx).armed,
                        "the original {page:?} submit is genuinely armed"
                    );
                }
                let paint = source.core_workspace_paint(cx).unwrap();
                source.dismiss_menu(window, cx);
                observe(cx, Uuid::from_u128(72), "Replacement group", 4);
                source.open_core_workspace_rename(Uuid::from_u128(72), window, cx);
                source.menu.input.as_mut().unwrap().select(2..5);
                window.focus(&source.focus, cx);
                paint
            });
            Frozen {
                source,
                paint,
                _peer: peer,
            }
        });
        draw(cx);
        assert!(cx.debug_bounds("menu-leaving-shield").is_none());
        let generation = view.read_with(cx, |view, cx| {
            view.source
                .read(cx)
                .core_terminals
                .workspace_ui
                .dialog_generation
        });
        let selectors: &[&str] = if page == Page::CoreWorkspaceActions {
            &[
                "core-workspace-action-rename",
                "core-workspace-action-archive",
                "core-workspace-action-delete",
            ]
        } else if page == Page::CoreWorkspaceDelete {
            &["core-workspace-create", "core-workspace-cancel"]
        } else {
            &[
                "dialog-input",
                "core-workspace-create",
                "core-workspace-cancel",
            ]
        };
        for &selector in selectors {
            let bounds = cx.debug_bounds(selector).expect(selector);
            cx.simulate_mouse_move(bounds.center(), None, Modifiers::default());
            cx.simulate_click(bounds.center(), Modifiers::default());
            cx.update(|window, cx| {
                view.read_with(cx, |view, cx| {
                    let source = view.source.read(cx);
                    let state = &source.core_terminals.workspace_ui;
                    assert_eq!(source.menu.page, Some(Page::CoreWorkspaceRename));
                    assert_eq!(state.dialog_generation, generation);
                    assert_eq!(state.target.unwrap().id, Uuid::from_u128(72));
                    assert!(state.pending.is_none());
                    assert!(source.menu.error.is_none());
                    let input = source.menu.input.as_ref().unwrap();
                    assert_eq!(input.text, "Replacement group");
                    assert_eq!(input.selection(), 2..5);
                    assert_eq!(input.bounds, gpui::Bounds::default());
                    assert!(source.focus.is_focused(window));
                })
            });
        }
    }
}

#[cfg(unix)]
fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

#[gpui::test]
fn capture_keeps_the_accepted_edit_receiver_and_new_dialog_generation(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            observe(cx, Uuid::from_u128(71), "Original group", 3);
            view.open_core_workspace_dialog(window, cx);
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            view.core_terminals.workspace_ui.pending = Some(Pending {
                dialog_generation: view.core_terminals.workspace_ui.dialog_generation,
                dismiss_page: Some(Page::CoreWorkspaceCreate),
                receiver,
            });
            let _paint = view.core_workspace_paint(cx).unwrap();
            view.dismiss_menu(window, cx);
            assert!(view.core_terminals.workspace_ui.pending.is_some());
            view.open_core_workspace_rename(Uuid::from_u128(71), window, cx);
            sender.send(Ok(())).unwrap();
            view.poll_core_workspace_edit(window, cx);
            assert!(view.core_terminals.workspace_ui.pending.is_none());
            assert_eq!(view.menu.page, Some(Page::CoreWorkspaceRename));
            assert_eq!(view.menu.input.as_ref().unwrap().text, "Original group");
        })
    });
}
