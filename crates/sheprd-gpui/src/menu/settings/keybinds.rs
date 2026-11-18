// ABOUTME: Presents one prepared shortcut reference through active and inert templates.
// ABOUTME: Captures filtered rows and input paint without retaining focus or action owners.
use super::{command_name, keycaps, paste_label, reach_note, shortcut_matches};
use crate::{
    SheprdWindow,
    controls::{COMMANDS, Command, CommandInfo},
    i18n::fl,
    menu::accent,
};
use gpui::{prelude::*, *};
use std::borrow::Cow;

struct Row {
    keys: Vec<(SharedString, Option<SharedString>)>,
    description: SharedString,
    info: Option<&'static CommandInfo>,
}

pub(in crate::menu) struct Paint {
    sections: Vec<(SharedString, Vec<Row>)>,
    search: Option<crate::search_input::Paint>,
    editor: Option<crate::menu::shortcut_editor::Paint>,
    count: usize,
    total: usize,
    offset: Point<Pixels>,
}

impl SheprdWindow {
    pub(in crate::menu) fn keybinds_paint(&self, cx: &App) -> Paint {
        let query = self
            .menu
            .keybinds_search
            .as_ref()
            .map(|search| search.read(cx).text())
            .unwrap_or("");
        let paste = paste_label();
        let mut groups = [
            (fl!("menu-settings-workspaces"), Vec::new()),
            (fl!("menu-settings-navigation"), Vec::new()),
            (
                fl!("menu-settings-application"),
                vec![(
                    vec![(Cow::Borrowed(crate::actions::EDIT_KEYS[2]), None)],
                    Cow::Owned(paste.clone()),
                    None,
                )],
            ),
            (fl!("menu-settings-commands"), Vec::new()),
        ];
        for info in COMMANDS {
            if !self.core_enabled() && crate::core_feature::command_requires_core(info.command) {
                continue;
            }
            let mut keys: Vec<&str> = self.keymap().shortcuts(info.command).collect();
            if info.command == Command::Palette && self.config.palette.double_shift {
                keys.push("shift shift"); // i18n: data
            }
            let group = match info.command {
                Command::Workspace
                | Command::OpenWorktree
                | Command::EditWorktreeNote
                | Command::Tab
                | Command::SplitRight
                | Command::SplitDown
                | Command::Zoom
                | Command::ClearPane
                | Command::Find
                | Command::FindNext
                | Command::FindPrevious
                | Command::CopyMode
                | Command::EditScrollback
                | Command::ClosePane
                | Command::CloseTab
                | Command::CloseWorkspace
                | Command::ReopenClosedTab
                | Command::SplitEditor
                | Command::MoveTabPrevious
                | Command::MoveTabNext
                | Command::RenameTab
                | Command::SwapLeft
                | Command::SwapRight
                | Command::SwapUp
                | Command::SwapDown
                | Command::ResizeLeft
                | Command::ResizeRight
                | Command::ResizeUp
                | Command::ResizeDown
                | Command::ResizeMode
                | Command::EqualizeSplits
                | Command::RenamePane
                | Command::MovePane
                | Command::RenameWorkspace => 0,
                Command::NextTab
                | Command::PreviousTab
                | Command::FocusLeft
                | Command::FocusRight
                | Command::FocusUp
                | Command::FocusDown
                | Command::NextPane
                | Command::PreviousPane
                | Command::TabNumber(_)
                | Command::WorkspacePicker
                | Command::WorktreeNotes
                | Command::LastPane
                | Command::Back
                | Command::Forward
                | Command::PreviousWorkspace
                | Command::NextWorkspace
                | Command::WorkspaceNumber(_)
                | Command::PreviousAgent
                | Command::NextAgent
                | Command::AgentNumber(_) => 1,
                Command::NewWindow
                | Command::ToggleSidebar
                | Command::ToggleStatusBar
                | Command::FocusNextRegion
                | Command::FocusPreviousRegion
                | Command::ToggleZenMode
                | Command::DevicesOverview
                | Command::NewCoreTerminal
                | Command::MirrorCorePane
                | Command::IncreaseFontSize
                | Command::DecreaseFontSize
                | Command::ResetFontSize
                | Command::Settings
                | Command::Keybinds
                | Command::Sessions
                | Command::Themes
                | Command::Palette
                | Command::ResumePalette
                | Command::Detach
                | Command::Reconnect
                | Command::Quit
                | Command::Logs
                | Command::Messages
                | Command::CheckForUpdates
                | Command::About
                | Command::InstallBrowserSkill
                | Command::ToggleFullScreen
                | Command::CycleWindows
                | Command::CloseWindow
                | Command::ReloadConfig => 2,
                Command::OpenNotificationTarget => 1,
            };
            groups[group].1.push((
                keys.into_iter()
                    .map(|key| (Cow::Borrowed(key), None))
                    .collect(),
                Cow::Owned(crate::i18n::command(info)),
                Some(info),
            ));
        }
        if let Some(snapshot) = self
            .live
            .snapshot
            .as_ref()
            .filter(|_| self.live.status.is_connected())
        {
            let commands: Vec<_> = crate::keymap::available_commands(&snapshot.commands).collect();
            let bindings = self.keymap().custom_bindings(commands.iter().copied());
            groups[3]
                .1
                .extend(commands.iter().zip(bindings).enumerate().map(
                    |(index, (command, bindings))| {
                        let description = command_name(command);
                        // A row's name keys its selectors and is all that tells
                        // rows apart, so one already used by a built-in action
                        // or an earlier command is numbered. Daemon command IDs
                        // are generated per boot and mean nothing to a reader.
                        let earlier = usize::from(
                            description == paste
                                || COMMANDS
                                    .iter()
                                    .any(|info| crate::i18n::command(info) == description),
                        ) + commands[..index]
                            .iter()
                            .filter(|other| command_name(other) == description)
                            .count();
                        let description = match earlier {
                            0 => description.to_owned(),
                            earlier => format!("{description} ({})", earlier + 1),
                        };
                        (
                            bindings
                                .into_iter()
                                .map(|binding| {
                                    (Cow::Owned(binding.label), reach_note(binding.reach))
                                })
                                .collect(),
                            Cow::Owned(description),
                            None,
                        )
                    },
                ));
        }
        let total = groups.iter().map(|(_, rows)| rows.len()).sum();
        let sections: Vec<(SharedString, Vec<Row>)> = groups
            .into_iter()
            .filter_map(|(section, rows)| {
                let rows: Vec<_> = rows
                    .into_iter()
                    .filter(|(keys, description, _)| {
                        shortcut_matches(query, "", description, &section)
                            || keys.iter().any(|(keys, _)| {
                                shortcut_matches(query, keys, description, &section)
                            })
                    })
                    .map(|(keys, description, info)| Row {
                        keys: keys
                            .into_iter()
                            .map(|(key, note)| (key.into_owned().into(), note.map(Into::into)))
                            .collect(),
                        description: description.into_owned().into(),
                        info,
                    })
                    .collect();
                (!rows.is_empty()).then_some((section.into(), rows))
            })
            .collect();
        let count = sections.iter().map(|(_, rows)| rows.len()).sum();
        Paint {
            sections,
            search: self
                .menu
                .keybinds_search
                .as_ref()
                .map(|search| search.read(cx).paint()),
            editor: self.shortcut_editor_paint(cx),
            count,
            total,
            offset: self.menu.keybinds_scroll.offset(),
        }
    }

    pub(in crate::menu) fn render_keybinds(&self, cx: &mut Context<Self>) -> Div {
        self.render_keybinds_content(&self.keybinds_paint(cx), true, cx)
    }

    pub(in crate::menu) fn render_keybinds_paint(
        &self,
        paint: &Paint,
        cx: &mut Context<Self>,
    ) -> Div {
        self.render_keybinds_content(paint, false, cx)
    }

    fn render_keybinds_content(&self, paint: &Paint, active: bool, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let accent = accent(theme);
        let count = paint.count;
        let total = paint.total;
        let editor = paint
            .editor
            .as_ref()
            .map(|editor| self.render_shortcut_editor(editor, active, cx));
        let search = if active {
            self.menu
                .keybinds_search
                .as_ref()
                .map(|search| search.clone().into_any_element())
        } else {
            paint
                .search
                .as_ref()
                .map(|search| search.render().into_any_element())
        };
        let mut body = div().children(editor);
        for (section, shortcuts) in &paint.sections {
            body = body.child(
                div()
                    .pt(px(12.))
                    .pb(px(6.))
                    .text_size(px(font.size * 0.85))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(accent)
                    .child(section.clone()),
            );
            for row in shortcuts {
                let keys = &row.keys;
                let description = &row.description;
                let info = row.info;
                body = body.child(
                    div()
                        .debug_selector(|| format!("shortcut-{description}"))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .py(px(7.))
                        .border_b_1()
                        .border_color(rgb(theme.active))
                        .child(
                            div()
                                .debug_selector(|| format!("keys-{description}"))
                                .w(relative(0.45))
                                .flex_none()
                                .flex()
                                .flex_wrap()
                                .gap(px(10.))
                                .when(keys.is_empty(), |column| {
                                    column.child(
                                        div()
                                            .text_color(rgb(theme.muted))
                                            .child(fl!("menu-settings-no-shortcut")),
                                    )
                                })
                                .children(keys.iter().map(|(keys, note)| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(3.))
                                        .child(div().flex().flex_wrap().gap(px(4.)).children(
                                            keycaps(keys).map(|key| {
                                                div()
                                                    .flex_none()
                                                    .px(px(6.))
                                                    .py(px(2.))
                                                    .rounded(px(crate::config::corners::SMALL))
                                                    .border_1()
                                                    .border_color(rgb(theme.active))
                                                    .bg(rgb(theme.background))
                                                    .text_size(px(font.size * 0.9))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .child(key)
                                            }),
                                        ))
                                        .when_some(note.clone(), |binding, note| {
                                            binding.child(
                                                div()
                                                    .text_size(px(font.size * 0.85))
                                                    .text_color(rgb(theme.muted))
                                                    .child(note),
                                            )
                                        })
                                })),
                        )
                        .child(
                            div()
                                .debug_selector(|| format!("description-{description}"))
                                .flex_1()
                                .min_w_0()
                                .child(description.clone()),
                        )
                        .when_some(info, |row, info| {
                            row.child(
                                div()
                                    .id(SharedString::from(format!("edit-shortcut-{}", info.name)))
                                    .debug_selector(move || format!("edit-shortcut-{}", info.name))
                                    .flex_none()
                                    .px(px(6.))
                                    .py(px(4.))
                                    .text_color(accent)
                                    .child(fl!("menu-settings-edit"))
                                    .when(active, |button| {
                                        button.cursor_pointer().on_click(cx.listener(
                                            move |this, _, window, cx| {
                                                this.edit_shortcut(info, window, cx)
                                            },
                                        ))
                                    }),
                            )
                        }),
                );
            }
        }
        if count == 0 {
            body = body.child(
                div()
                    .debug_selector(|| "keybinds-empty".into())
                    .py(px(20.))
                    .text_color(rgb(theme.muted))
                    .child(fl!("menu-settings-no-match")),
            );
        }
        body = body.child(
            div()
                .py(px(14.))
                .text_color(rgb(theme.subtext()))
                .child(fl!("menu-settings-note")),
        );
        let body = div()
            .id("keybinds-body")
            .debug_selector(|| "keybinds-body".into())
            .flex_1()
            .min_h_0()
            .when(active, |body| {
                body.overflow_y_scroll()
                    .track_scroll(&self.menu.keybinds_scroll)
            })
            .when(!active, |body| body.overflow_hidden())
            .px(px(16.))
            .py(px(8.))
            .child(
                div()
                    .relative()
                    .top(if active { px(0.) } else { paint.offset.y })
                    .child(body),
            );
        div()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .child(
                div()
                    .debug_selector(|| "keybinds-header".into())
                    .flex()
                    .items_center()
                    .flex_none()
                    .gap(px(12.))
                    .p(px(16.))
                    .border_b_1()
                    .border_color(rgb(theme.active))
                    .child(
                        div()
                            .w(px(3.))
                            .h(px(font.size * 2.5))
                            .rounded_full()
                            .bg(accent),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(font.size * 1.35))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(fl!("menu-settings-title")),
                            )
                            .child(
                                div()
                                    .text_color(rgb(theme.muted))
                                    .child(fl!("menu-settings-subtitle")),
                            ),
                    )
                    .child(
                        div()
                            .id("menu-close")
                            .debug_selector(|| "keybinds-close".into())
                            .flex_none()
                            .px(px(8.))
                            .py(px(4.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .text_color(rgb(theme.muted))
                            .child(fl!("menu-settings-close"))
                            .when(active, |button| {
                                button
                                    .cursor_pointer()
                                    .hover(|style| {
                                        style
                                            .bg(rgb(theme.active))
                                            .text_color(rgb(theme.foreground))
                                    })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.dismiss_menu(window, cx)
                                    }))
                            }),
                    ),
            )
            .child(
                div()
                    .debug_selector(|| "keybinds-search-area".into())
                    .flex_none()
                    .px(px(16.))
                    .py(px(8.))
                    .when_some(search, |area, search| area.child(search))
                    .child(
                        div()
                            .debug_selector(|| "keybinds-count".into())
                            .pt(px(4.))
                            .text_color(rgb(theme.muted))
                            .child(fl!("menu-settings-count", count = count, total = total)),
                    ),
            )
            .child(body)
            .child(
                div()
                    .debug_selector(|| "keybinds-footer".into())
                    .flex_none()
                    .px(px(16.))
                    .py(px(10.))
                    .border_t_1()
                    .border_color(rgb(theme.active))
                    .text_color(rgb(theme.muted))
                    .child(fl!("menu-settings-footer")),
            )
    }
}
