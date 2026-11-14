//! What the Settings window writes for every `Preference`, pinned as exact
//! text, and read back through the parser loading uses. The text is the
//! compatibility contract of the settings writer: a change here changes
//! users' files.
use super::*;
use crate::config::preferences::Preference;
use crate::config::status_bar::{AwakeScope, Button, Detail, Edit};
use sheprd_client::protocol::ToastHerdrPosition;

/// A file that already holds unrelated keys, comments, and tables.
const HEAD: &str = "# personal\ntheme = 'Nord' # keep\n";
const NOTIFICATIONS: &str = "\n[notifications]\n";
const NOTIFICATIONS_BODY: &str = "enabled = false # mine\nposition = \"top-left\" # corner\n";
const REST: &str = "\n[bell]\nsound = true # ding\n";

/// Where the populated file differs after an edit; the rest is unchanged.
enum Full {
    /// A top-level key, after the ones already there.
    Root(&'static str),
    /// The `[notifications]` keys, which the file already sets.
    Notifications(&'static str),
    /// A table the file did not have, appended.
    Tail(&'static str),
}

impl Full {
    fn text(&self) -> String {
        let (root, notifications, tail) = match self {
            Self::Root(root) => (*root, NOTIFICATIONS_BODY, ""),
            Self::Notifications(body) => ("", *body, ""),
            Self::Tail(tail) => ("", NOTIFICATIONS_BODY, *tail),
        };
        format!("{HEAD}{root}{NOTIFICATIONS}{notifications}{REST}{tail}")
    }
}

struct Row {
    variant: &'static str,
    edit: Preference,
    /// The whole file after the edit, starting from an empty one.
    empty: &'static str,
    full: Full,
    /// Whether a parsed config holds the value the edit wrote.
    read: fn(&Config) -> bool,
}

/// Declares each variant's rows once, with a match over the same names: a
/// `Preference` variant without rows here does not compile.
macro_rules! rows {
    ($($variant:ident { $($edit:expr => $empty:literal, $full:expr, $read:expr;)+ })+) => {
        fn rows() -> Vec<Row> {
            vec![$($(Row {
                variant: stringify!($variant),
                edit: $edit,
                empty: $empty,
                full: $full,
                read: $read,
            },)+)+]
        }

        fn variant(edit: &Preference) -> &'static str {
            match edit {
                $(Preference::$variant { .. } => stringify!($variant),)+
            }
        }
    };
}

fn sides(start: f32, end: f32) -> Sides {
    Sides::new(start, end).expect("padding in band")
}

fn language(tag: &str) -> sheprd_i18n::Language {
    tag.parse().expect("a known language")
}

fn host(host: &str, color: Option<u32>) -> Preference {
    Preference::HostColor {
        host: host.into(),
        color,
    }
}

use Full::{Notifications as InNotifications, Root, Tail};

rows! {
    AgentsPlacement {
        Preference::AgentsPlacement(AgentsPlacement::Nested) =>
            "agents_placement = \"nested\"\n", Root("agents_placement = \"nested\"\n"),
            |c| c.agents_placement == AgentsPlacement::Nested;
        Preference::AgentsPlacement(AgentsPlacement::Panel) =>
            "agents_placement = \"panel\"\n", Root("agents_placement = \"panel\"\n"),
            |c| c.agents_placement == AgentsPlacement::Panel;
    }
    ChatGlide {
        Preference::ChatGlide(true) =>
            "[chat]\nglide = true\n", Tail("\n[chat]\nglide = true\n"), |c| c.chat.glide;
    }
    ChatLauncher {
        Preference::ChatLauncher(ChatLauncher::Trailing) =>
            "[chat]\nlauncher = \"trailing\"\n", Tail("\n[chat]\nlauncher = \"trailing\"\n"),
            |c| c.chat.launcher == ChatLauncher::Trailing;
    }
    ChatLayout {
        Preference::ChatLayout(ChatLayoutMode::Bubbles) =>
            "[chat]\nlayout = \"bubbles\"\n", Tail("\n[chat]\nlayout = \"bubbles\"\n"),
            |c| c.chat.layout == ChatLayoutMode::Bubbles;
    }
    ChatMaxWidth {
        Preference::ChatMaxWidth(640) =>
            "[chat]\nmax_width = 640\n", Tail("\n[chat]\nmax_width = 640\n"),
            |c| c.chat.max_width == 640;
        Preference::ChatMaxWidth(0) =>
            "[chat]\nmax_width = 0\n", Tail("\n[chat]\nmax_width = 0\n"),
            |c| c.chat.max_width == 0;
    }
    ClipboardEnabled {
        Preference::ClipboardEnabled(Some(false)) =>
            "[clipboard_toast]\nenabled = false\n", Tail("\n[clipboard_toast]\nenabled = false\n"),
            |c| c.clipboard_overrides.enabled() == Some(false);
        Preference::ClipboardEnabled(None) =>
            "[clipboard_toast]\n", Tail("\n[clipboard_toast]\n"),
            |c| c.clipboard_overrides.enabled().is_none();
    }
    ClipboardPosition {
        Preference::ClipboardPosition(Some(ClipboardToastPosition::TopLeft)) =>
            "[clipboard_toast]\nposition = \"top-left\"\n",
            Tail("\n[clipboard_toast]\nposition = \"top-left\"\n"),
            |c| c.clipboard_overrides.position() == Some(ClipboardToastPosition::TopLeft);
        Preference::ClipboardPosition(Some(ClipboardToastPosition::TopCenter)) =>
            "[clipboard_toast]\nposition = \"top-center\"\n",
            Tail("\n[clipboard_toast]\nposition = \"top-center\"\n"),
            |c| c.clipboard_overrides.position() == Some(ClipboardToastPosition::TopCenter);
        Preference::ClipboardPosition(Some(ClipboardToastPosition::TopRight)) =>
            "[clipboard_toast]\nposition = \"top-right\"\n",
            Tail("\n[clipboard_toast]\nposition = \"top-right\"\n"),
            |c| c.clipboard_overrides.position() == Some(ClipboardToastPosition::TopRight);
        Preference::ClipboardPosition(Some(ClipboardToastPosition::BottomLeft)) =>
            "[clipboard_toast]\nposition = \"bottom-left\"\n",
            Tail("\n[clipboard_toast]\nposition = \"bottom-left\"\n"),
            |c| c.clipboard_overrides.position() == Some(ClipboardToastPosition::BottomLeft);
        Preference::ClipboardPosition(Some(ClipboardToastPosition::BottomCenter)) =>
            "[clipboard_toast]\nposition = \"bottom-center\"\n",
            Tail("\n[clipboard_toast]\nposition = \"bottom-center\"\n"),
            |c| c.clipboard_overrides.position() == Some(ClipboardToastPosition::BottomCenter);
        Preference::ClipboardPosition(Some(ClipboardToastPosition::BottomRight)) =>
            "[clipboard_toast]\nposition = \"bottom-right\"\n",
            Tail("\n[clipboard_toast]\nposition = \"bottom-right\"\n"),
            |c| c.clipboard_overrides.position() == Some(ClipboardToastPosition::BottomRight);
        Preference::ClipboardPosition(None) =>
            "[clipboard_toast]\n", Tail("\n[clipboard_toast]\n"),
            |c| c.clipboard_overrides.position().is_none();
    }
    ConfirmClosePane {
        Preference::ConfirmClosePane(false) =>
            "confirm_close_pane = false\n", Root("confirm_close_pane = false\n"),
            |c| !c.confirm_close_pane;
    }
    ConfirmCloseTab {
        Preference::ConfirmCloseTab(false) =>
            "confirm_close_tab = false\n", Root("confirm_close_tab = false\n"),
            |c| !c.confirm_close_tab;
    }
    EdgeRail {
        Preference::EdgeRail(true) => "edge_rail = true\n", Root("edge_rail = true\n"),
            |c| c.edge_rail;
    }
    Feature {
        Preference::Feature(Feature::QaMenu, true) =>
            "[features]\nqa_menu = true\n", Tail("\n[features]\nqa_menu = true\n"),
            |c| c.features.enabled(Feature::QaMenu);
        Preference::Feature(Feature::AcpChats, false) =>
            "[features]\nacp_chats = false\n", Tail("\n[features]\nacp_chats = false\n"),
            |c| !c.features.enabled(Feature::AcpChats);
    }
    FocusFollowsMouse {
        Preference::FocusFollowsMouse(false) =>
            "focus_follows_mouse = false\n", Root("focus_follows_mouse = false\n"),
            |c| !c.focus_follows_mouse;
    }
    Herdr2Terminals {
        Preference::Herdr2Terminals(true) =>
            "herdr_2_terminals = true\n", Root("herdr_2_terminals = true\n"),
            |c| c.herdr_2_terminals;
    }
    HerdrCoreEnabled {
        Preference::HerdrCoreEnabled(true) =>
            "herdr_core_enabled = true\n", Root("herdr_core_enabled = true\n"),
            |c| c.herdr_core_enabled;
    }
    HostColor {
        host("Work", Some(0x5a3a3a)) =>
            "[sidebar]\n\n[sidebar.hosts]\nWork = \"#5a3a3a\"\n",
            Tail("\n[sidebar]\n\n[sidebar.hosts]\nWork = \"#5a3a3a\"\n"),
            |c| c.sidebar_style.hosts.get("Work") == Some(&0x5a3a3a);
        // A dot in a host's name is part of the name, not a table.
        host("dev.example.com", Some(0xff)) =>
            "[sidebar]\n\n[sidebar.hosts]\n\"dev.example.com\" = \"#0000ff\"\n",
            Tail("\n[sidebar]\n\n[sidebar.hosts]\n\"dev.example.com\" = \"#0000ff\"\n"),
            |c| c.sidebar_style.hosts.get("dev.example.com") == Some(&0xff);
        host("Work", None) =>
            "[sidebar]\n\n[sidebar.hosts]\n", Tail("\n[sidebar]\n\n[sidebar.hosts]\n"),
            |c| c.sidebar_style.hosts.is_empty();
    }
    Language {
        Preference::Language(language("en-XA")) =>
            "language = \"en-XA\"\n", Root("language = \"en-XA\"\n"),
            |c| c.language == language("en-XA");
        Preference::Language(sheprd_i18n::Language::System) =>
            "language = \"system\"\n", Root("language = \"system\"\n"),
            |c| c.language == sheprd_i18n::Language::System;
    }
    NotificationDelay {
        Preference::NotificationDelay(Some(3600)) =>
            "[notifications]\ndelay_seconds = 3600\n",
            InNotifications(
                "enabled = false # mine\nposition = \"top-left\" # corner\ndelay_seconds = 3600\n"
            ),
            |c| c.notification_overrides.delay_seconds() == Some(3600);
        Preference::NotificationDelay(Some(0)) =>
            "[notifications]\ndelay_seconds = 0\n",
            InNotifications(
                "enabled = false # mine\nposition = \"top-left\" # corner\ndelay_seconds = 0\n"
            ),
            |c| c.notification_overrides.delay_seconds() == Some(0);
        Preference::NotificationDelay(None) =>
            "[notifications]\n", InNotifications(NOTIFICATIONS_BODY),
            |c| c.notification_overrides.delay_seconds().is_none();
    }
    NotificationEnabled {
        // A changed value keeps the comment beside it.
        Preference::NotificationEnabled(Some(true)) =>
            "[notifications]\nenabled = true\n",
            InNotifications("enabled = true # mine\nposition = \"top-left\" # corner\n"),
            |c| c.notification_overrides.enabled() == Some(true);
        Preference::NotificationEnabled(None) =>
            "[notifications]\n", InNotifications("position = \"top-left\" # corner\n"),
            |c| c.notification_overrides.enabled().is_none();
    }
    NotificationFinished {
        Preference::NotificationFinished(false) =>
            "[notifications]\nfinished = false\n",
            InNotifications(
                "enabled = false # mine\nposition = \"top-left\" # corner\nfinished = false\n"
            ),
            |c| !c.notifications.finished;
    }
    NotificationPosition {
        Preference::NotificationPosition(Some(ToastHerdrPosition::TopLeft)) =>
            "[notifications]\nposition = \"top-left\"\n",
            InNotifications("enabled = false # mine\nposition = \"top-left\" # corner\n"),
            |c| c.notification_overrides.position() == Some(ToastHerdrPosition::TopLeft);
        Preference::NotificationPosition(Some(ToastHerdrPosition::TopRight)) =>
            "[notifications]\nposition = \"top-right\"\n",
            InNotifications("enabled = false # mine\nposition = \"top-right\" # corner\n"),
            |c| c.notification_overrides.position() == Some(ToastHerdrPosition::TopRight);
        Preference::NotificationPosition(Some(ToastHerdrPosition::BottomLeft)) =>
            "[notifications]\nposition = \"bottom-left\"\n",
            InNotifications("enabled = false # mine\nposition = \"bottom-left\" # corner\n"),
            |c| c.notification_overrides.position() == Some(ToastHerdrPosition::BottomLeft);
        Preference::NotificationPosition(Some(ToastHerdrPosition::BottomRight)) =>
            "[notifications]\nposition = \"bottom-right\"\n",
            InNotifications("enabled = false # mine\nposition = \"bottom-right\" # corner\n"),
            |c| c.notification_overrides.position() == Some(ToastHerdrPosition::BottomRight);
        Preference::NotificationPosition(None) =>
            "[notifications]\n", InNotifications("enabled = false # mine\n"),
            |c| c.notification_overrides.position().is_none();
    }
    ShowAgentsHeader {
        Preference::ShowAgentsHeader(false) =>
            "show_agents_header = false\n", Root("show_agents_header = false\n"),
            |c| !c.show_agents_header;
    }
    ShowDeviceIcons {
        Preference::ShowDeviceIcons(false) =>
            "show_device_icons = false\n", Root("show_device_icons = false\n"),
            |c| !c.show_device_icons;
    }
    ShowListeningPorts {
        Preference::ShowListeningPorts(false) =>
            "show_listening_ports = false\n", Root("show_listening_ports = false\n"),
            |c| !c.show_listening_ports;
    }
    ShowNetworkQuality {
        Preference::ShowNetworkQuality(true) =>
            "show_network_quality = true\n", Root("show_network_quality = true\n"),
            |c| c.show_network_quality;
    }
    ShowRepositoryIcons {
        Preference::ShowRepositoryIcons(false) =>
            "show_repository_icons = false\n", Root("show_repository_icons = false\n"),
            |c| !c.show_repository_icons;
    }
    ShowSidebarSearch {
        Preference::ShowSidebarSearch(false) =>
            "show_sidebar_search = false\n", Root("show_sidebar_search = false\n"),
            |c| !c.show_sidebar_search;
    }
    ShowWindowControls {
        Preference::ShowWindowControls(true) =>
            "show_window_controls = true\n", Root("show_window_controls = true\n"),
            |c| c.show_window_controls;
        // Linux shows the controls by default, so only this row differs there.
        Preference::ShowWindowControls(false) =>
            "show_window_controls = false\n", Root("show_window_controls = false\n"),
            |c| !c.show_window_controls;
    }
    SidebarActivityBar {
        Preference::SidebarActivityBar(false) =>
            "[sidebar]\nactivity_bar = false\n", Tail("\n[sidebar]\nactivity_bar = false\n"),
            |c| !c.sidebar_style.activity_bar;
    }
    SidebarAutomations {
        Preference::SidebarAutomations(false) =>
            "[sidebar]\nautomations = false\n", Tail("\n[sidebar]\nautomations = false\n"),
            |c| !c.sidebar_style.automations;
    }
    SidebarDeviceFooter {
        Preference::SidebarDeviceFooter(DeviceFooter::Always) =>
            "[sidebar]\ndevice_footer = \"always\"\n",
            Tail("\n[sidebar]\ndevice_footer = \"always\"\n"),
            |c| c.sidebar_style.device_footer == DeviceFooter::Always;
        Preference::SidebarDeviceFooter(DeviceFooter::Auto) =>
            "[sidebar]\ndevice_footer = \"auto\"\n",
            Tail("\n[sidebar]\ndevice_footer = \"auto\"\n"),
            |c| c.sidebar_style.device_footer == DeviceFooter::Auto;
        Preference::SidebarDeviceFooter(DeviceFooter::Never) =>
            "[sidebar]\ndevice_footer = \"never\"\n",
            Tail("\n[sidebar]\ndevice_footer = \"never\"\n"),
            |c| c.sidebar_style.device_footer == DeviceFooter::Never;
    }
    SidebarHostGap {
        Preference::SidebarHostGap(Some(48.)) =>
            "[sidebar]\nhost_gap = 48.0\n", Tail("\n[sidebar]\nhost_gap = 48.0\n"),
            |c| c.sidebar_style.overrides.host_gap == Some(48.);
        Preference::SidebarHostGap(None) =>
            "[sidebar]\n", Tail("\n[sidebar]\n"),
            |c| c.sidebar_style.overrides.host_gap.is_none();
    }
    SidebarIndent {
        Preference::SidebarIndent(Some(12.5)) =>
            "[sidebar]\nindent = 12.5\n", Tail("\n[sidebar]\nindent = 12.5\n"),
            |c| c.sidebar_style.overrides.indent == Some(12.5);
        Preference::SidebarIndent(None) =>
            "[sidebar]\n", Tail("\n[sidebar]\n"),
            |c| c.sidebar_style.overrides.indent.is_none();
    }
    SidebarRowGap {
        Preference::SidebarRowGap(Some(0.)) =>
            "[sidebar]\ngap = 0.0\n", Tail("\n[sidebar]\ngap = 0.0\n"),
            |c| c.sidebar_style.overrides.gap == Some(0.);
        Preference::SidebarRowGap(None) =>
            "[sidebar]\n", Tail("\n[sidebar]\n"),
            |c| c.sidebar_style.overrides.gap.is_none();
    }
    SidebarRowPadding {
        Preference::SidebarRowPadding(Some(16.)) =>
            "[sidebar]\nrow_padding = 16.0\n", Tail("\n[sidebar]\nrow_padding = 16.0\n"),
            |c| c.sidebar_style.overrides.row_padding == Some(16.);
        Preference::SidebarRowPadding(None) =>
            "[sidebar]\n", Tail("\n[sidebar]\n"),
            |c| c.sidebar_style.overrides.row_padding.is_none();
    }
    SidebarSelect {
        Preference::SidebarSelect(SelectMode::Row) =>
            "[sidebar]\nselect = \"row\"\n", Tail("\n[sidebar]\nselect = \"row\"\n"),
            |c| c.sidebar_style.select == SelectMode::Row;
        Preference::SidebarSelect(SelectMode::Group) =>
            "[sidebar]\nselect = \"group\"\n", Tail("\n[sidebar]\nselect = \"group\"\n"),
            |c| c.sidebar_style.select == SelectMode::Group;
        Preference::SidebarSelect(SelectMode::GroupDim) =>
            "[sidebar]\nselect = \"group-dim\"\n", Tail("\n[sidebar]\nselect = \"group-dim\"\n"),
            |c| c.sidebar_style.select == SelectMode::GroupDim;
    }
    SmoothScroll {
        Preference::SmoothScroll(false) =>
            "smooth_scroll = false\n", Root("smooth_scroll = false\n"), |c| !c.smooth_scroll;
    }
    SoundVolume {
        Preference::SoundVolume(100) =>
            "[sound]\nvolume = 100\n", Tail("\n[sound]\nvolume = 100\n"),
            |c| c.sound.volume == 100;
        Preference::SoundVolume(0) =>
            "[sound]\nvolume = 0\n", Tail("\n[sound]\nvolume = 0\n"), |c| c.sound.volume == 0;
    }
    StatusBar {
        Preference::StatusBar(Edit::Usage(Detail::Detailed)) =>
            "[status_bar]\nusage = \"detailed\"\n", Tail("\n[status_bar]\nusage = \"detailed\"\n"),
            |c| c.status_bar.usage == Detail::Detailed;
        Preference::StatusBar(Edit::Usage(Detail::Compact)) =>
            "[status_bar]\nusage = \"compact\"\n", Tail("\n[status_bar]\nusage = \"compact\"\n"),
            |c| c.status_bar.usage == Detail::Compact;
        Preference::StatusBar(Edit::KeepAwake(false)) =>
            "[status_bar]\nkeep_awake = false\n", Tail("\n[status_bar]\nkeep_awake = false\n"),
            |c| !c.status_bar.keep_awake;
        Preference::StatusBar(Edit::KeepAwakeScope(AwakeScope::Display)) =>
            "[status_bar]\nkeep_awake_scope = \"display\"\n",
            Tail("\n[status_bar]\nkeep_awake_scope = \"display\"\n"),
            |c| c.status_bar.keep_awake_scope == AwakeScope::Display;
        Preference::StatusBar(Edit::KeepAwakeScope(AwakeScope::System)) =>
            "[status_bar]\nkeep_awake_scope = \"system\"\n",
            Tail("\n[status_bar]\nkeep_awake_scope = \"system\"\n"),
            |c| c.status_bar.keep_awake_scope == AwakeScope::System;
        Preference::StatusBar(Edit::Theme(Button::Label)) =>
            "[status_bar]\ntheme = \"label\"\n", Tail("\n[status_bar]\ntheme = \"label\"\n"),
            |c| c.status_bar.theme == Button::Label;
        Preference::StatusBar(Edit::Theme(Button::Icon)) =>
            "[status_bar]\ntheme = \"icon\"\n", Tail("\n[status_bar]\ntheme = \"icon\"\n"),
            |c| c.status_bar.theme == Button::Icon;
        Preference::StatusBar(Edit::Theme(Button::Hidden)) =>
            "[status_bar]\ntheme = \"hidden\"\n", Tail("\n[status_bar]\ntheme = \"hidden\"\n"),
            |c| c.status_bar.theme == Button::Hidden;
        Preference::StatusBar(Edit::Shortcuts(Button::Icon)) =>
            "[status_bar]\nshortcuts = \"icon\"\n", Tail("\n[status_bar]\nshortcuts = \"icon\"\n"),
            |c| c.status_bar.shortcuts == Button::Icon;
        Preference::StatusBar(Edit::ReportIssue(Button::Hidden)) =>
            "[status_bar]\nreport_issue = \"hidden\"\n",
            Tail("\n[status_bar]\nreport_issue = \"hidden\"\n"),
            |c| c.status_bar.report_issue == Button::Hidden;
    }
    StatusIcons {
        Preference::StatusIcons(StatusIcons::Braille) =>
            "status_icons = \"braille\"\n", Root("status_icons = \"braille\"\n"),
            |c| c.status_icons == StatusIcons::Braille;
    }
    TabAgentTitle {
        Preference::TabAgentTitle(true) =>
            "[tab_labels]\nagent_title = true\n", Tail("\n[tab_labels]\nagent_title = true\n"),
            |c| c.tab_labels.agent_title;
        Preference::TabAgentTitle(false) =>
            "[tab_labels]\nagent_title = false\n", Tail("\n[tab_labels]\nagent_title = false\n"),
            |c| !c.tab_labels.agent_title;
    }
    TabLayout {
        Preference::TabLayout(TabLayoutMode::Pills) =>
            "tab_layout = \"pills\"\n", Root("tab_layout = \"pills\"\n"),
            |c| c.tab_layout == TabLayoutMode::Pills;
    }
    TerminalPadding {
        // Sides that agree are one number, whole pixels an integer.
        Preference::TerminalPadding(Axis::X, sides(8., 8.)) =>
            "[terminal]\npadding_x = 8\n", Tail("\n[terminal]\npadding_x = 8\n"),
            |c| c.terminal_padding.x == sides(8., 8.);
        Preference::TerminalPadding(Axis::X, sides(7.5, 7.5)) =>
            "[terminal]\npadding_x = 7.5\n", Tail("\n[terminal]\npadding_x = 7.5\n"),
            |c| c.terminal_padding.x == sides(7.5, 7.5);
        Preference::TerminalPadding(Axis::Y, sides(4., 6.5)) =>
            "[terminal]\npadding_y = [4, 6.5]\n", Tail("\n[terminal]\npadding_y = [4, 6.5]\n"),
            |c| c.terminal_padding.y == sides(4., 6.5);
        Preference::TerminalPadding(Axis::Y, sides(0., 0.)) =>
            "[terminal]\npadding_y = 0\n", Tail("\n[terminal]\npadding_y = 0\n"),
            |c| c.terminal_padding.y == sides(0., 0.);
    }
}

fn written(base: &str, edit: Preference) -> anyhow::Result<String> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("local.toml");
    fs::write(&path, base)?;
    Config::save_preference_path(edit, &path)?;
    Ok(fs::read_to_string(&path)?)
}

#[test]
fn every_preference_writes_pinned_text_that_loads_back() -> anyhow::Result<()> {
    let populated = Root("").text();
    let defaults = Config::parse("")?;
    let mut failures = Vec::new();
    // A variant whose every row matches the defaults proves nothing by
    // reading back; each needs a row that the defaults do not satisfy.
    let mut distinct = std::collections::BTreeSet::new();
    let rows = rows();
    for row in &rows {
        assert_eq!(
            variant(&row.edit),
            row.variant,
            "row under the wrong variant"
        );
        if !(row.read)(&defaults) {
            distinct.insert(row.variant);
        }
        for (base, expected) in [("", row.empty.to_owned()), (&*populated, row.full.text())] {
            let text = written(base, row.edit.clone())?;
            if text != expected {
                failures.push(format!(
                    "{:?} over {base:?}\n  wrote    {text:?}\n  expected {expected:?}",
                    row.edit
                ));
            }
            match Config::parse(&text) {
                Ok(config) if (row.read)(&config) => {}
                Ok(_) => failures.push(format!("{:?}: read back differently: {text:?}", row.edit)),
                Err(error) => {
                    failures.push(format!("{:?}: refused by the loader: {error}", row.edit))
                }
            }
        }
    }
    for row in &rows {
        if !distinct.contains(row.variant) {
            failures.push(format!("{}: no row differs from the defaults", row.variant));
        }
    }
    failures.dedup();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// The cause under the path every writer failure is wrapped in.
fn cause(error: &crate::Error) -> &crate::Error {
    match error {
        crate::Error::Path { source, .. } => source,
        other => other,
    }
}

/// The shared-settings failure the config boundary boxes.
fn shared(error: &crate::Error) -> Option<&crate::herdr_settings::Error> {
    match error {
        crate::Error::ConfigFile { source, .. } => source.downcast_ref(),
        _ => None,
    }
}

#[test]
fn out_of_range_preferences_keep_their_typed_errors_and_write_nothing() -> anyhow::Result<()> {
    use crate::herdr_settings::Error as Shared;
    type Check = fn(&crate::Error) -> bool;
    let cases: [(Preference, Check); 9] = [
        (Preference::NotificationDelay(Some(3601)), |error| {
            matches!(shared(error), Some(Shared::ToastDelay))
        }),
        (Preference::SoundVolume(101), |error| {
            matches!(error, crate::Error::Settings(Error::InvalidSoundVolume))
        }),
        (
            Preference::SidebarIndent(Some(48.5)),
            |error| matches!(error, crate::Error::Settings(Error::InvalidSidebarMetric { key: "indent", max }) if *max == 48.),
        ),
        (
            Preference::SidebarRowPadding(Some(-0.5)),
            |error| matches!(error, crate::Error::Settings(Error::InvalidSidebarMetric { key: "row_padding", max }) if *max == 16.),
        ),
        (
            Preference::SidebarRowGap(Some(f32::NAN)),
            |error| matches!(error, crate::Error::Settings(Error::InvalidSidebarMetric { key: "gap", max }) if *max == 32.),
        ),
        (
            Preference::SidebarHostGap(Some(f32::INFINITY)),
            |error| matches!(error, crate::Error::Settings(Error::InvalidSidebarMetric { key: "host_gap", max }) if *max == 48.),
        ),
        (
            host("Work", Some(0x100_0000)),
            |error| matches!(error, crate::Error::Settings(Error::InvalidHostColor { host, value }) if host == "Work" && value == "#1000000"),
        ),
        (Preference::ChatMaxWidth(319), |error| {
            matches!(error, crate::Error::Settings(Error::InvalidChatWidth))
        }),
        (Preference::ChatMaxWidth(4001), |error| {
            matches!(error, crate::Error::Settings(Error::InvalidChatWidth))
        }),
    ];
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("local.toml");
    let original = Root("").text();
    for (edit, expected) in cases {
        fs::write(&path, &original)?;
        let shown = format!("{edit:?}");
        let error = Config::save_preference_path(edit, &path)
            .err()
            .with_context(|| format!("{shown} was accepted"))?;
        assert!(
            matches!(&error, crate::Error::Path { path: failed, .. } if failed == &path),
            "{shown}: {error:?}"
        );
        assert!(expected(cause(&error)), "{shown}: {error:?}");
        assert_eq!(fs::read_to_string(&path)?, original, "{shown}");
    }
    Ok(())
}

#[test]
fn a_table_name_that_holds_a_value_is_refused_by_name() -> anyhow::Result<()> {
    use crate::herdr_settings::Error as Shared;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("local.toml");
    for (original, edit, table) in [
        (
            "notifications = false\n",
            Preference::NotificationEnabled(Some(true)),
            "notifications",
        ),
        ("sidebar = 1\n", host("Work", Some(1)), "sidebar"),
        ("sidebar = 1\n", host("Work", None), "sidebar"),
        (
            "[sidebar]\nhosts = 1\n",
            host("Work", Some(1)),
            "sidebar.hosts",
        ),
    ] {
        fs::write(&path, original)?;
        let error = Config::save_preference_path(edit, &path)
            .err()
            .context("a value was edited as a table")?;
        assert!(
            matches!(shared(cause(&error)), Some(Shared::Table(name)) if *name == table),
            "{error:?}"
        );
        assert_eq!(fs::read_to_string(&path)?, original);
    }
    Ok(())
}

/// A file the loader already refuses can still be edited from the window:
/// the edit that fixes it must land, and so must one beside the mistake.
#[test]
fn a_file_that_does_not_load_can_still_be_edited() -> anyhow::Result<()> {
    let broken = "[sound]\nvolume = 500 # too loud\n";
    assert!(Config::parse(broken).is_err());
    let fixed = written(broken, Preference::SoundVolume(40))?;
    assert_eq!(fixed, "[sound]\nvolume = 40 # too loud\n");
    assert_eq!(Config::parse(&fixed)?.sound.volume, 40);

    let beside = written(broken, Preference::ConfirmCloseTab(false))?;
    assert_eq!(
        beside,
        "confirm_close_tab = false\n[sound]\nvolume = 500 # too loud\n"
    );
    assert!(Config::parse(&beside).is_err());
    Ok(())
}

#[test]
fn a_missing_file_starts_from_the_seeded_overrides() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("local.toml");
    Config::save_preference_path(Preference::EdgeRail(true), &path)?;
    let text = fs::read_to_string(&path)?;
    assert_eq!(text, format!("{LOCAL_CONFIG}edge_rail = true\n"));
    assert!(Config::parse(&text)?.edge_rail);
    Ok(())
}

#[test]
fn horizontal_padding_removes_the_deprecated_gap_it_replaces() -> anyhow::Result<()> {
    let original = "[layout]\nmode = 'compact'\nsidebar_gap = 3 # old\n";
    let text = written(
        original,
        Preference::TerminalPadding(Axis::X, sides(2., 4.)),
    )?;
    assert_eq!(
        text,
        "[layout]\nmode = 'compact'\n\n[terminal]\npadding_x = [2, 4]\n"
    );
    assert_eq!(Config::parse(&text)?.terminal_padding.x, sides(2., 4.));
    // The vertical axis never stood in for the gap, so it leaves it alone.
    let text = written(
        original,
        Preference::TerminalPadding(Axis::Y, sides(1., 1.)),
    )?;
    assert_eq!(text, format!("{original}\n[terminal]\npadding_y = 1\n"));
    Ok(())
}

#[test]
fn inline_tables_take_edits_in_place() -> anyhow::Result<()> {
    let text = written(
        "notifications = { enabled = false } # one line\n",
        Preference::NotificationDelay(Some(5)),
    )?;
    assert_eq!(
        text,
        "notifications = { enabled = false , delay_seconds = 5 } # one line\n"
    );
    assert_eq!(
        Config::parse(&text)?.notification_overrides.delay_seconds(),
        Some(5)
    );
    let text = written(
        "sidebar = { indent = 4.0 } # one line\n",
        host("Work", Some(0x5a3a3a)),
    )?;
    assert_eq!(
        text,
        "sidebar = { indent = 4.0 , hosts.Work = \"#5a3a3a\" } # one line\n"
    );
    let config = Config::parse(&text)?;
    assert_eq!(config.sidebar_style.hosts.get("Work"), Some(&0x5a3a3a));
    assert_eq!(config.sidebar_style.overrides.indent, Some(4.));
    let text = written(&text, host("Work", None))?;
    assert_eq!(text, "sidebar = { indent = 4.0 } # one line\n");
    assert!(Config::parse(&text)?.sidebar_style.hosts.is_empty());
    Ok(())
}
