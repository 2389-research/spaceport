// ABOUTME: Exercises frozen workspace controls without a parent input shield.
// ABOUTME: Protects replacement targets, drafts, and focus from departing content.
#![allow(clippy::expect_used)]
use super::*;
use crate::{
    fonts::StyledFont,
    menu::{
        Page,
        workspace::{DialogPaint, PopoverPaint},
    },
};
use gpui::{
    Bounds, Context, Entity, Modifiers, Point, Render, TestAppContext, VisualTestContext, Window,
    div, prelude::*, px,
};

enum Paint {
    Popover(PopoverPaint),
    Dialog(Box<DialogPaint>),
}

struct Preview {
    source: Entity<SheprdWindow>,
    paint: Option<Paint>,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.source.update(cx, |source, cx| {
            let body = match &self.paint {
                Some(Paint::Popover(paint)) => source.render_workspace_popover_paint(paint, cx),
                Some(Paint::Dialog(paint)) => source.render_workspace_dialog_paint(paint, cx),
                None => match source.menu.page {
                    Some(Page::Dialog(action)) => source.render_workspace_dialog(action, cx),
                    _ => source.render_workspace_popover(cx),
                },
            };
            div()
                .id("workspace-preview")
                .w(px(560.))
                .h(px(600.))
                .text_font(&source.config.ui)
                .text_size(px(source.config.ui.size))
                .line_height(px(source.config.ui.line_height()))
                .child(body)
        })
    }
}

fn preview(
    action: Option<WorkspaceAction>,
    cx: &mut TestAppContext,
) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| sidebar::layout_tests::fixture_window(window, cx));
        source.update(cx, |source, cx| {
            source.live.status = crate::state::ConnectionStatus::Connected;
            source.open_workspace_menu("w3", Point::default(), window, cx);
            if let Some(action) = action {
                source.open_workspace_dialog(action, window, cx);
            }
        });
        Preview {
            source,
            paint: None,
        }
    })
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn freeze(preview: &Entity<Preview>, dialog: bool, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            preview.paint = Some(preview.source.update(cx, |source, cx| {
                let paint = if dialog {
                    Paint::Dialog(Box::new(source.workspace_dialog_paint(cx).unwrap()))
                } else {
                    Paint::Popover(source.workspace_popover_paint().unwrap())
                };
                source.dismiss_menu(window, cx);
                source.open_workspace_menu("w4", Point::default(), window, cx);
                source.open_workspace_dialog(WorkspaceAction::Rename, window, cx);
                source.menu.input = Some(DialogInput::new("replacement draft".into()));
                source.menu.input.as_mut().unwrap().select(2..5);
                window.focus(&source.focus, cx);
                paint
            }));
        })
    });
    draw(cx);
}

fn unchanged(preview: &Entity<Preview>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        preview.read_with(cx, |preview, cx| {
            let source = preview.source.read(cx);
            assert_eq!(
                source.menu.page,
                Some(Page::Dialog(WorkspaceAction::Rename))
            );
            assert_eq!(source.menu.target.as_ref().unwrap().id, "w4");
            assert!(source.menu.workspace_selected.is_none());
            assert!(source.menu.error.is_none());
            assert!(source.menu.opening.is_none());
            let input = source.menu.input.as_ref().unwrap();
            assert_eq!(input.text, "replacement draft");
            assert_eq!(input.selection(), 2..5);
            assert_eq!(
                input.bounds,
                Bounds::default(),
                "frozen paint cannot overwrite the new field's IME geometry"
            );
            assert!(source.focus.is_focused(window));
        })
    });
}

#[gpui::test]
fn departing_workspace_actions_cannot_target_a_replacement(cx: &mut TestAppContext) {
    let (preview, cx) = preview(None, cx);
    draw(cx);
    freeze(&preview, false, cx);
    assert!(cx.debug_bounds("menu-leaving-shield").is_none());
    for selector in [
        "workspace-menu-Mute notifications",
        "workspace-menu-Rename",
        "workspace-menu-Close workspace",
    ] {
        let bounds = cx.debug_bounds(selector).expect(selector);
        cx.simulate_mouse_move(bounds.center(), None, Modifiers::default());
        cx.simulate_click(bounds.center(), Modifiers::default());
        unchanged(&preview, cx);
    }
}

#[gpui::test]
fn departing_dialog_cannot_submit_cancel_or_take_replacement_input(cx: &mut TestAppContext) {
    let (preview, cx) = preview(Some(WorkspaceAction::Rename), cx);
    draw(cx);
    freeze(&preview, true, cx);
    assert!(cx.debug_bounds("menu-leaving-shield").is_none());
    unchanged(&preview, cx);
    for selector in ["dialog-input", "dialog-submit", "dialog-cancel"] {
        let bounds = cx.debug_bounds(selector).expect(selector);
        cx.simulate_click(bounds.center(), Modifiers::default());
        unchanged(&preview, cx);
    }
}

fn seed_picker(source: &mut SheprdWindow, cx: &mut Context<SheprdWindow>, name: &str) {
    let mut picker =
        crate::menu::worktree_open::Picker::new(cx.new(crate::search_input::SearchInput::new));
    picker.pending = Some("fixture-list".into());
    source.menu.worktree_open = Some(picker);
    source.menu.apply_worktree_list_response("fixture-list", Ok(serde_json::json!({"result": {
        "type": "worktree_list", "source": {"repo_key": "/synthetic/.git", "repo_name":"synthetic", "source_workspace_id":"w3"},
        "worktrees": (0..3).map(|index| serde_json::json!({"path":format!("/synthetic/{name}-{index}"), "label":format!("{name}-{index}"), "is_bare":false, "is_prunable":false, "is_detached":true})).collect::<Vec<_>>()
    }})));
}

#[gpui::test]
fn departing_worktree_controls_cannot_act_on_a_replacement_picker(cx: &mut TestAppContext) {
    let (preview, cx) = preview(Some(WorkspaceAction::OpenWorktree), cx);
    preview.update(cx, |preview, cx| {
        preview
            .source
            .update(cx, |source, cx| seed_picker(source, cx, "original"))
    });
    draw(cx);
    let search = cx.update(|window, cx| {
        preview.update(cx, |preview, cx| {
            let (paint, search) = preview.source.update(cx, |source, cx| {
                let search = source
                    .menu
                    .worktree_open
                    .as_ref()
                    .unwrap()
                    .search
                    .downgrade();
                let paint = source.workspace_dialog_paint(cx).unwrap();
                source.dismiss_menu(window, cx);
                source.open_workspace_menu("w3", Point::default(), window, cx);
                source.open_workspace_dialog(WorkspaceAction::OpenWorktree, window, cx);
                seed_picker(source, cx, "replacement");
                source.menu.worktree_open.as_mut().unwrap().selected = 1;
                window.focus(&source.focus, cx);
                (paint, search)
            });
            preview.paint = Some(Paint::Dialog(Box::new(paint)));
            search
        })
    });
    draw(cx);
    assert!(search.upgrade().is_none());
    assert!(cx.debug_bounds("menu-leaving-shield").is_none());
    for selector in [
        "open-worktree-row-0",
        "open-worktree-escape",
        "dialog-submit",
        "dialog-cancel",
    ] {
        let bounds = cx.debug_bounds(selector).expect(selector);
        cx.simulate_click(bounds.center(), Modifiers::default());
        cx.update(|window, cx| {
            preview.read_with(cx, |preview, cx| {
                let source = preview.source.read(cx);
                assert_eq!(
                    source.menu.page,
                    Some(Page::Dialog(WorkspaceAction::OpenWorktree))
                );
                let picker = source.menu.worktree_open.as_ref().unwrap();
                assert_eq!(picker.selected, 1);
                assert_eq!(picker.entry(0).unwrap().label, "replacement-0");
                assert!(source.menu.opening.is_none());
                assert!(source.menu.error.is_none());
                assert!(source.focus.is_focused(window));
            })
        });
    }
}
