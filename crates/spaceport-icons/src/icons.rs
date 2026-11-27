//! The UI glyphs, agent marks and status sets, and the source that serves
//! them all by asset path.

use gpui::{AssetSource, SharedString};
use std::borrow::Cow;

/// The one `gpui::AssetSource`: every embedded icon, by asset path.
pub struct Icons;

/// Declares each agent mark once: its variant, Herdr's canonical
/// `agent_label` identity, and the embedded asset named after that identity.
macro_rules! agent_icons {
    ($($variant:ident => $label:literal, $extension:literal,)+) => {
        /// Canonical daemon identities, independent of editable display names.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum AgentIcon {
            $($variant,)+
            Generic,
        }

        impl AgentIcon {
            const ALL: &[Self] = &[$(Self::$variant,)+ Self::Generic];

            pub fn from_identity(identity: Option<&str>) -> Self {
                match identity {
                    $(Some($label) => Self::$variant,)+
                    _ => Self::Generic,
                }
            }

            pub fn path(self) -> &'static str {
                match self {
                    $(Self::$variant => concat!("icons/agent-", $label, ".", $extension),)+
                    Self::Generic => "icons/agent-generic.svg",
                }
            }

            fn bytes(self) -> &'static [u8] {
                match self {
                    $(Self::$variant => include_bytes!(concat!(
                        "../../../assets/icons/agent-", $label, ".", $extension
                    )),)+
                    Self::Generic => include_bytes!("../../../assets/icons/agent-generic.svg"),
                }
            }

            pub fn element(self, size: f32, color: u32) -> gpui::AnyElement {
                icon(self.path(), size, color)
            }
        }
    };
}

// Keyed on Herdr's `src/detect/mod.rs` `agent_label`, in its declaration order.
agent_icons! {
    Pi => "pi", "svg",
    Omo => "omo", "png",
    Claude => "claude", "svg",
    Codex => "codex", "svg",
    Gemini => "gemini", "svg",
    Cursor => "cursor", "svg",
    Devin => "devin", "svg",
    Antigravity => "agy", "svg",
    Cline => "cline", "svg",
    Omp => "omp", "svg",
    Mastracode => "mastracode", "svg",
    OpenCode => "opencode", "svg",
    Copilot => "copilot", "svg",
    Kimi => "kimi", "svg",
    Kiro => "kiro", "svg",
    Droid => "droid", "svg",
    Amp => "amp", "svg",
    Grok => "grok", "svg",
    Hermes => "hermes", "svg",
    Kilo => "kilo", "svg",
    Qodercli => "qodercli", "svg",
    Qwen => "qwen", "svg",
    Letta => "letta", "svg",
    Maki => "maki", "svg",
    Muse => "muse", "svg",
}

/// Raster marks retain their source colors; SVG glyphs use the caller's ink.
pub fn icon(path: impl Into<SharedString>, size: f32, color: u32) -> gpui::AnyElement {
    use gpui::{ObjectFit, img, prelude::*, px, rgb, svg};
    let path = path.into();
    if path.ends_with(".png") {
        img(path)
            .size(px(size))
            .flex_none()
            .object_fit(ObjectFit::Contain)
            .into_any_element()
    } else {
        svg()
            .path(path)
            .size(px(size))
            .flex_none()
            .text_color(rgb(color))
            .into_any_element()
    }
}

/// Embeds each `status_icons` set, one asset per status, in the order
/// `status_icon` indexes: sets as `StatusIcons` declares them, then statuses.
macro_rules! status_icons {
    ($($set:literal),+) => {
        const STATUS_ICONS: [(&str, &[u8]); 15] = [$(
            status_icons!(@one $set, "idle"),
            status_icons!(@one $set, "working"),
            status_icons!(@one $set, "done"),
            status_icons!(@one $set, "blocked"),
            status_icons!(@one $set, "unknown"),
        )+];
    };
    (@one $set:literal, $status:literal) => {
        (
            concat!("icons/status-", $set, "-", $status, ".svg"),
            include_bytes!(concat!("../../../assets/icons/status-", $set, "-", $status, ".svg")),
        )
    };
}

// Solid, then tiles' inner mark, then the outline set.
status_icons!("solid", "mark", "outline");

/// The asset that draws `status` in a `status_icons` set, or `None` for the
/// sets whose marks are drawn rather than loaded.
pub fn status_icon(
    icons: spaceport_settings::StatusIcons,
    status: spaceport_protocol::AgentStatus,
) -> Option<&'static str> {
    use spaceport_protocol::AgentStatus;
    use spaceport_settings::StatusIcons;
    let set = match icons {
        StatusIcons::Solid => 0,
        StatusIcons::Tiles => 1,
        StatusIcons::Outline => 2,
        _ => return None,
    };
    let status = match status {
        AgentStatus::Idle => 0,
        AgentStatus::Working => 1,
        AgentStatus::Done => 2,
        AgentStatus::Blocked => 3,
        AgentStatus::Unknown => 4,
    };
    Some(STATUS_ICONS[set * 5 + status].0)
}

/// A checkout the user keeps a note on, beside the working-tree marker and
/// in a hue of its own so the two never read as one.
pub fn note(theme: &spaceport_settings::Theme, size: f32) -> gpui::Div {
    use gpui::{div, prelude::*, px, rgb, rgba, svg};
    div()
        .size(px(size))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(spaceport_settings::corners::SMALL))
        .bg(rgba((theme.palette[4] << 8) | 0x30))
        .border_1()
        .border_color(rgba((theme.palette[4] << 8) | 0x90))
        .child(
            svg()
                .path("icons/note.svg")
                .size(px(size - 4.))
                .text_color(rgb(theme.ink(theme.palette[4]))),
        )
}

/// A theme the user starred, and the outline offering to star one.
pub const STARRED: &str = "icons/star-full.svg";
pub const UNSTARRED: &str = "icons/star.svg";

/// The mark of a workspace or tab whose notifications the user muted.
pub const MUTED: &str = "icons/bell-off.svg";

/// `mark` named "Notifications muted" for screen readers, which hear nothing
/// of an icon: for a muted mark in a row that carries no accessible name of
/// its own.
pub fn spoken_muted(
    id: impl Into<gpui::ElementId>,
    mark: impl gpui::IntoElement,
) -> gpui::Stateful<gpui::Div> {
    use gpui::{Role, div, prelude::*};
    div()
        .id(id)
        .role(Role::Image)
        .aria_label("Notifications muted")
        .flex_none()
        .child(mark)
}

/// A workspace whose notifications the user muted: the bell struck through,
/// in the theme's quiet greys rather than a hue, so it reads as silence
/// beside the other marks. The glyph fills more of its box than theirs do,
/// since a slash through a bell blurs sooner than a pencil.
pub fn muted(theme: &spaceport_settings::Theme, size: f32) -> gpui::Div {
    use gpui::{div, prelude::*, px, rgb, rgba, svg};
    div()
        .size(px(size))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(spaceport_settings::corners::SMALL))
        .bg(rgba((theme.muted << 8) | 0x24))
        .border_1()
        .border_color(rgba((theme.muted << 8) | 0x70))
        .child(
            svg()
                .path(MUTED)
                .size(px(size - 2.))
                .text_color(rgb(theme.subtext())),
        )
}

/// Declares each embedded UI icon once, so loading and listing cannot drift.
macro_rules! embedded_icons {
    ($($path:literal,)+) => {
        const EMBEDDED: &[(&str, &[u8])] = &[$((
            concat!("icons/", $path),
            include_bytes!(concat!("../../../assets/icons/", $path)),
        ),)+];
    };
}

// Embedded UI icons under assets/icons, one per line; agent marks declare
// their own. Keep the list sorted so parallel additions land
// on different lines.
embedded_icons! {
    // keep-sorted start
    "arrow-left.svg",
    "arrow-right.svg",
    "arrow-up.svg",
    "azure-devops.svg",
    "bell-off.svg",
    "bell.svg",
    "brain.svg",
    "chart.svg",
    "check.svg",
    "chevron-down.svg",
    "chevron-left.svg",
    "chevron-right.svg",
    "chevron-up.svg",
    "close.svg",
    "code.svg",
    "coffee-full.svg",
    "coffee.svg",
    "computer.svg",
    "copy.svg",
    "cpu.svg",
    "devices.svg",
    "error.svg",
    "external.svg",
    "fan-out.svg",
    "file.svg",
    "folder.svg",
    "git-branch.svg",
    "globe.svg",
    "keyboard.svg",
    "lock.svg",
    "more.svg",
    "network/tailscale.svg",
    "note.svg",
    "panel-left.svg",
    "panel-right.svg",
    "pass.svg",
    "pencil.svg",
    "play.svg",
    "plug.svg",
    "plus.svg",
    "providers/factory.svg",
    "pull-request.svg",
    "pulse.svg",
    "refresh.svg",
    "search.svg",
    "sessions.svg",
    "settings.svg",
    "shield.svg",
    "split.svg",
    "star-full.svg",
    "star.svg",
    "status-bar.svg",
    "status-orbit.svg",
    "status-orca-blocked.svg",
    "status-orca-working.svg",
    "tabs.svg",
    "terminal.svg",
    "theme.svg",
    "trash.svg",
    "user.svg",
    "whitespace.svg",
    "window-maximize.svg",
    "window-minimize.svg",
    "window-restore.svg",
    "x.svg",
    "zoom.svg",
    // keep-sorted end
}

impl AssetSource for Icons {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        if let Some(icon) = AgentIcon::ALL.iter().find(|icon| icon.path() == path) {
            return Ok(Some(Cow::Borrowed(icon.bytes())));
        }
        let bytes = EMBEDDED
            .iter()
            .chain(&STATUS_ICONS)
            .copied()
            .chain(crate::brand::assets())
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| bytes);
        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(EMBEDDED
            .iter()
            .map(|(name, _)| *name)
            .chain(AgentIcon::ALL.iter().map(|icon| icon.path()))
            .chain(STATUS_ICONS.iter().map(|(name, _)| *name))
            .chain(crate::brand::assets().map(|(name, _)| name))
            .filter(|name| name.starts_with(path))
            .map(Into::into)
            .collect())
    }
}

#[cfg(test)]
mod tests;
