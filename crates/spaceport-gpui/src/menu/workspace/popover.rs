//! The workspace popover's words and window half: the label each action says,
//! how wide the popover's captions shape, and what resting on an action or
//! pressing it do. The grid, the rows and the header's toggle are
//! `spaceport_menu::workspace::popover`'s.

// i18n: migrated
use crate::{
    SpaceportWindow,
    menu::{WorkspaceMenuAction, accent, workspace_action_icon},
};
use gpui::{prelude::*, *};
use spaceport_menu::workspace::popover;

pub(crate) use spaceport_menu::workspace::popover::Item;

/// An action's menu label: a message ID, shown in the current language. Its
/// English names the action's elements, so tests find them in any language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Label(&'static str);

impl Label {
    // keep-sorted start
    pub(crate) const COLLAPSE_GROUP: Self = Self::new("worktree-popover-collapse-group");
    pub(crate) const EXPAND_GROUP: Self = Self::new("worktree-popover-expand-group");
    pub(crate) const MUTE: Self = Self::new("worktree-popover-mute");
    /// The word the header's toggle shows, where "notifications" is the row's
    /// to say.
    pub(crate) const MUTE_BUTTON: Self = Self::new("worktree-popover-mute-button");
    pub(crate) const NOTE: Self = Self::new("worktree-popover-note");
    pub(crate) const RENAME: Self = Self::new("worktree-popover-rename");
    /// The caption on the rename tile, where its icon already says "workspace".
    pub(crate) const RENAME_TILE: Self = Self::new("worktree-popover-tile-rename");
    pub(crate) const UNMUTE: Self = Self::new("worktree-popover-unmute");
    pub(crate) const UNMUTE_BUTTON: Self = Self::new("worktree-popover-unmute-button");
    // keep-sorted end
    /// Checked against the English files as it compiles, as `fl!` is.
    const fn new(id: &'static str) -> Self {
        crate::i18n::check(id, &[]);
        Self(id)
    }

    /// The label in the current language.
    pub(crate) fn text(self) -> String {
        crate::i18n::message(self.0, &[])
    }

    /// The English label, which names the action's elements in tests.
    pub(crate) fn english(self) -> String {
        crate::i18n::english(self.0, &[])
    }
}

/// One action the popover offers, `label` as its row says it and, where its
/// tile or the header's toggle shortens it, that `caption`.
pub(in crate::menu) fn item(
    action: WorkspaceMenuAction,
    label: Label,
    caption: Option<Label>,
) -> Item {
    Item {
        action,
        caption: caption.unwrap_or(label).text(),
        icon: workspace_action_icon(action),
        label: label.text(),
        name: label.english(),
    }
}

/// A word painted as this app paints it: the string itself, or under the layout
/// tests the probe element they read the laid-out text back from.
fn label_element(text: &str) -> AnyElement {
    crate::sidebar::label_text(text).into_any_element()
}

impl SpaceportWindow {
    /// The popover's width: 340, or as wide as the longest caption needs in a
    /// longer language, and never wider than `room`. Captions are short and
    /// few, and GPUI caches each shaped line, so measuring them as the popover
    /// draws costs no more than drawing them.
    pub(in crate::menu) fn workspace_popover_width(&self, window: &Window, room: Pixels) -> Pixels {
        let size = px((self.config.ui.size - 1.).max(8.));
        let font = crate::config::font_for(&self.config.ui);
        popover::width(&self.workspace_items(), room, |caption| {
            let mut run = window.text_style().to_run(caption.len());
            run.font = font.clone();
            window
                .text_system()
                .shape_line(caption.into(), size, &[run], None)
                .width
        })
    }

    pub(in crate::menu) fn render_workspace_popover(&self, cx: &mut Context<Self>) -> Div {
        let Some(target) = &self.menu.target else {
            return div();
        };
        let text = crate::menu::chrome::text(&self.config.ui);
        let items = self.workspace_items();
        let hosts = popover::Hosts {
            accent: accent(&self.theme),
            branch: target.branch.as_deref(),
            items: &items,
            name: &target.label,
            paint: label_element,
            selected: self.menu.workspace_selected,
            text: &text,
            theme: &self.theme,
        };
        popover::panel(&hosts, |action, element| {
            self.workspace_interact(action, element, cx)
        })
    }

    /// Wires one action's element with what resting on it and pressing it do.
    fn workspace_interact(
        &self,
        action: WorkspaceMenuAction,
        element: Stateful<Div>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        element
            .on_hover(Self::workspace_hover(action, cx))
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                this.activate_workspace_menu(action, window, cx);
            }))
    }

    /// Hover selects, as the arrow keys do, so both highlight the same item.
    fn workspace_hover(
        action: WorkspaceMenuAction,
        cx: &mut Context<Self>,
    ) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
        cx.listener(move |this, hovered: &bool, _, cx| {
            if *hovered {
                this.menu.workspace_selected = Some(action);
            } else if this.menu.workspace_selected == Some(action) {
                this.menu.workspace_selected = None;
            }
            cx.notify();
        })
    }
}
