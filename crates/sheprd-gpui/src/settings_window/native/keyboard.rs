// ABOUTME: Qualifies Settings keyboard behavior through exact-window AppKit events.
// ABOUTME: Keeps traversal, activation, modal focus, and invalid edits in the private fixture.
use super::*;
use crate::sidebar::native_tests::Key;
use crate::{config::ChatLayoutMode, settings_window::inputs::NumberField};

const TAB_LIMIT: usize = 160;

async fn foreground(settings: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    let title = settings.update(cx, |_, window, _| {
        let title = window.window_title();
        window.set_window_title("Spaceport keyboard check"); // i18n: data
        title
    })?;
    let result = wait_for_foreground(settings, cx).await;
    settings
        .update(cx, |_, window, _| window.set_window_title(&title))
        .context("restoring the Settings fixture title")?;
    result
}

async fn wait_for_foreground(
    settings: WindowHandle<SettingsWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    // Foreground permission is an external resource. The operator selects this
    // one window; no activation policy or desktop-wide input is changed here.
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let target = AnyWindowHandle::from(settings)
            .update(cx, |_, window, _| Target::acquire(window))??;
        let key = target.is_key();
        drop(target);
        if key {
            return Ok(());
        }
        if Instant::now() >= deadline {
            let target = AnyWindowHandle::from(settings)
                .update(cx, |_, window, _| Target::acquire(window))??;
            let facts = target.activation_facts()?;
            bail!("native Settings foreground resource timed out: {facts}");
        }
        cx.background_executor()
            .timer(Duration::from_millis(10))
            .await;
    }
}

fn draw(settings: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    AnyWindowHandle::from(settings).update(cx, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })?;
    Ok(())
}

fn press(settings: WindowHandle<SettingsWindow>, key: Key, cx: &mut AsyncApp) -> Result<()> {
    let target =
        AnyWindowHandle::from(settings).update(cx, |_, window, _| Target::acquire(window))??;
    ensure!(
        target.is_key(),
        "native keyboard target is not the Settings key window"
    );
    target.press(key)?;
    draw(settings, cx)
}

fn context(window: &Window, key: &str) -> Option<String> {
    window
        .context_stack()
        .iter()
        .rev()
        .find_map(|context| context.get(key).map(ToString::to_string))
}

fn anchor(settings: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    settings.update(cx, |view, window, cx| window.focus(&view.focus, cx))?;
    draw(settings, cx)
}

fn tab_to(
    settings: WindowHandle<SettingsWindow>,
    key: &str,
    id: &str,
    cx: &mut AsyncApp,
) -> Result<FocusHandle> {
    anchor(settings, cx)?;
    for _ in 0..TAB_LIMIT {
        press(settings, Key::Tab, cx)?;
        let focus = AnyWindowHandle::from(settings).update(cx, |_, window, cx| {
            (context(window, key).as_deref() == Some(id))
                .then(|| window.focused(cx))
                .flatten()
        })?;
        if let Some(focus) = focus {
            return Ok(focus);
        }
    }
    bail!("native Tab never reached {key}={id}")
}

async fn settle(settings: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let ready = settings.update(cx, |view, _, _| -> Result<_> {
            ensure!(
                view.error.is_none(),
                "native keyboard save failed: {:?}",
                view.error
            );
            Ok(!view.busy() && !view.persistence.loading())
        })??;
        if ready {
            return draw(settings, cx);
        }
        ensure!(
            Instant::now() < deadline,
            "native keyboard Settings save did not settle"
        );
        cx.background_executor()
            .timer(Duration::from_millis(10))
            .await;
    }
}

fn section(
    settings: WindowHandle<SettingsWindow>,
    section: Section,
    cx: &mut AsyncApp,
) -> Result<()> {
    tab_to(settings, "settings-section", &section.selector(), cx)?;
    press(settings, Key::Enter, cx)?;
    ensure!(
        settings.update(cx, |view, _, _| view.section == section)?,
        "native Enter did not select {section:?}"
    );
    Ok(())
}

fn row_bounds(id: &str, cx: &App) -> Result<Bounds<Pixels>> {
    let id: SharedString = id.to_owned().into();
    cx.try_global::<search::FocusBounds>()
        .and_then(|bounds| bounds.0.get(&id))
        .or_else(|| {
            cx.try_global::<search::RowBounds>()
                .and_then(|bounds| bounds.0.get(&id))
        })
        .copied()
        .with_context(|| format!("native Settings row was not painted: {id}"))
}

fn visible_row(settings: WindowHandle<SettingsWindow>, id: &str, cx: &mut AsyncApp) -> Result<()> {
    settings.update(cx, |view, window, cx| -> Result<()> {
        let row = row_bounds(id, cx)?;
        let body = view.body_scroll.bounds();
        ensure!(
            row.top() >= body.top() - px(1.) && row.bottom() <= body.bottom() + px(1.),
            "native focused row {id} stayed clipped: {row:?} in {body:?}"
        );
        ensure!(
            window.last_input_was_keyboard() && window.focused(cx).is_some(),
            "native row reveal lost keyboard focus"
        );
        Ok(())
    })??;
    Ok(())
}

fn ring_pixels(
    settings: WindowHandle<SettingsWindow>,
    id: &str,
    cx: &mut AsyncApp,
) -> Result<usize> {
    AnyWindowHandle::from(settings).update(cx, |root, window, cx| -> Result<_> {
        let view = root
            .downcast::<SettingsWindow>()
            .map_err(|_| anyhow::anyhow!("unexpected Settings root"))?;
        let row = row_bounds(id, cx)?;
        let color = view.read(cx).theme.ink(view.read(cx).theme.primary());
        let image = window.render_to_image()?;
        let scale = image.width() as f32 / f32::from(window.viewport_size().width);
        let left = f32::from((row.right() - px(92.)).max(row.left())) * scale;
        let right = f32::from(row.right()) * scale;
        let top = f32::from(row.top()) * scale;
        let bottom = f32::from(row.bottom()) * scale;
        let rgb = [(color >> 16) as u8, (color >> 8) as u8, color as u8];
        let mut matching = 0;
        for y in (top.max(0.) as u32)..(bottom as u32).min(image.height()) {
            for x in (left.max(0.) as u32)..(right as u32).min(image.width()) {
                let pixel = image.get_pixel(x, y).0;
                matching += usize::from(
                    pixel[3] > 240
                        && (0..3).all(|channel| pixel[channel].abs_diff(rgb[channel]) <= 3),
                );
            }
        }
        Ok(matching)
    })?
}

fn traversal_and_ring(settings: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    anchor(settings, cx)?;
    press(settings, Key::Tab, cx)?;
    let search_focus = settings.update(cx, |view, window, cx| -> Result<_> {
        let focus = view.nav_search.read(cx).focus.clone();
        ensure!(
            focus.is_focused(window),
            "native Tab missed navigation search"
        );
        Ok(focus)
    })??;
    press(settings, Key::Tab, cx)?;
    let general = AnyWindowHandle::from(settings).update(cx, |_, window, cx| -> Result<_> {
        ensure!(
            context(window, "settings-section").as_deref()
                == Some(Section::General.selector().as_str()),
            "native second Tab missed General"
        );
        window
            .focused(cx)
            .context("General navigation has no focus")
    })??;
    press(settings, Key::ShiftTab, cx)?;
    ensure!(
        AnyWindowHandle::from(settings)
            .update(cx, |_, window, _| search_focus.is_focused(window))?,
        "native Shift-Tab did not return to search"
    );
    press(settings, Key::Tab, cx)?;
    ensure!(
        AnyWindowHandle::from(settings).update(cx, |_, window, _| general.is_focused(window))?,
        "native Tab did not restore the same category focus"
    );
    press(settings, Key::Space, cx)?;
    ensure!(
        settings.update(cx, |view, _, _| view.section == Section::General)?,
        "native Space did not activate General"
    );
    let control = tab_to(settings, "settings-row", "general-smooth-scroll", cx)?;
    visible_row(settings, "general-smooth-scroll", cx)?;
    let focused = ring_pixels(settings, "general-smooth-scroll", cx)?;
    press(settings, Key::ShiftTab, cx)?;
    ensure!(
        AnyWindowHandle::from(settings).update(cx, |_, window, _| !control.is_focused(window))?,
        "native Shift-Tab did not leave the switch"
    );
    let unfocused = ring_pixels(settings, "general-smooth-scroll", cx)?;
    ensure!(
        focused > unfocused,
        "native keyboard focus ring was not painted: {focused} <= {unfocused}"
    );
    press(settings, Key::Tab, cx)?;
    ensure!(
        AnyWindowHandle::from(settings).update(cx, |_, window, _| control.is_focused(window))?,
        "native Tab lost the switch's focus handle"
    );
    Ok(())
}

async fn switch_and_segments(
    settings: WindowHandle<SettingsWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    let initial = settings.update(cx, |view, _, _| {
        (view.config.smooth_scroll, view.config.chat.layout)
    })?;
    tab_to(settings, "settings-row", "general-smooth-scroll", cx)?;
    press(settings, Key::Space, cx)?;
    settle(settings, cx).await?;
    ensure!(
        settings.update(cx, |view, _, _| view.config.smooth_scroll != initial.0)?,
        "native Space did not toggle smooth scrolling"
    );
    press(settings, Key::Enter, cx)?;
    settle(settings, cx).await?;
    ensure!(
        settings.update(cx, |view, _, _| view.config.smooth_scroll == initial.0)?,
        "native Enter did not restore smooth scrolling"
    );
    let old = ChatLayoutMode::ALL
        .iter()
        .position(|mode| *mode == initial.1)
        .context("current chat layout is absent from its catalog")?;
    ensure!(
        ChatLayoutMode::ALL.len() > 1,
        "chat layout fixture needs alternatives"
    );
    let next = (old + 1) % ChatLayoutMode::ALL.len();
    for (index, key) in [(next, Key::Enter), (old, Key::Space)] {
        tab_to(settings, "settings-row", "general-chat-layout", cx)?;
        for _ in 0..index {
            press(settings, Key::Tab, cx)?;
        }
        press(settings, key, cx)?;
        settle(settings, cx).await?;
        ensure!(
            settings.update(cx, |view, _, _| view.config.chat.layout
                == ChatLayoutMode::ALL[index])?,
            "native segment did not choose layout {index}"
        );
    }
    Ok(())
}

async fn dropdown(settings: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    let trigger = tab_to(settings, "settings-row", "general-language", cx)?;
    press(settings, Key::Enter, cx)?;
    let (selected, count) = settings.update(cx, |view, window, _| -> Result<_> {
        let open = view
            .dropdown
            .as_ref()
            .context("native Enter did not open language list")?;
        ensure!(
            open.focus().is_focused(window) && open.previous_focus() == Some(&trigger),
            "native dropdown did not own focus or remember its trigger"
        );
        Ok((open.selected, open.options.len()))
    })??;
    ensure!(count > 1, "language list has no alternatives");
    for (key, expected) in [(Key::Down, (selected + 1) % count), (Key::Up, selected)] {
        press(settings, key, cx)?;
        ensure!(
            settings.update(cx, |view, _, _| view
                .dropdown
                .as_ref()
                .is_some_and(|open| open.selected == expected))?,
            "native arrow did not move the dropdown selection"
        );
    }
    press(settings, Key::Escape, cx)?;
    ensure!(
        settings.update(cx, |view, window, _| view.dropdown.is_none()
            && trigger.is_focused(window))?,
        "native Escape did not restore the dropdown trigger"
    );
    press(settings, Key::Space, cx)?;
    ensure!(
        settings.update(cx, |view, _, _| view.dropdown.is_some())?,
        "native Space did not reopen the dropdown"
    );
    press(settings, Key::Enter, cx)?;
    settle(settings, cx).await?;
    ensure!(
        settings.update(cx, |view, window, _| view.dropdown.is_none()
            && trigger.is_focused(window))?,
        "native selection reopened the dropdown or lost trigger focus"
    );
    Ok(())
}

fn font_picker(settings: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    section(settings, Section::Fonts, cx)?;
    tab_to(settings, "settings-row", "fonts-all-row", cx)?;
    press(settings, Key::Enter, cx)?;
    ensure!(
        settings.update(cx, |view, window, cx| view.font_picker_shown()
            && view.native_font_search(cx).0.is_focused(window))?,
        "native font button did not open with search focus"
    );
    for step in 0..48 {
        let key = if step < 24 { Key::Tab } else { Key::ShiftTab };
        press(settings, key, cx)?;
        ensure!(
            AnyWindowHandle::from(settings).update(cx, |_, window, cx| {
                window.focused(cx).is_some()
                    && window
                        .context_stack()
                        .iter()
                        .any(|context| context.contains("SettingsFontPicker"))
            })?,
            "native Tab escaped the font picker"
        );
    }
    // Return to search through native traversal, then reach the preceding Close button.
    for _ in 0..TAB_LIMIT {
        if settings.update(cx, |view, window, cx| {
            view.native_font_search(cx).0.is_focused(window)
        })? {
            press(settings, Key::ShiftTab, cx)?;
            press(settings, Key::Space, cx)?;
            ensure!(
                settings.update(cx, |view, _, _| !view.font_picker_shown())?,
                "native Space did not activate the modal Close button"
            );
            return Ok(());
        }
        press(settings, Key::Tab, cx)?;
    }
    bail!("native font picker traversal never returned to search")
}

fn invalid_number(settings: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    section(settings, Section::Notifications, cx)?;
    tab_to(settings, "settings-row", "notif-delay", cx)?;
    visible_row(settings, "notif-delay", cx)?;
    let initial = settings.update(cx, |view, _, _| {
        view.config.notification_overrides.delay_seconds()
    })?;
    press(settings, Key::Enter, cx)?;
    let input = settings.update(cx, |view, window, cx| -> Result<_> {
        let edit = view
            .number_edit
            .as_ref()
            .context("native Enter did not open the number")?;
        ensure!(
            edit.field == NumberField::ToastDelay && edit.input.read(cx).focus.is_focused(window),
            "native number editor has the wrong field or focus"
        );
        Ok(edit.input.clone())
    })??;
    press(settings, Key::InvalidX, cx)?;
    ensure!(
        input.read_with(cx, |input, _| input.text() == "x"),
        "native invalid typing did not replace the selected value"
    );
    for key in [Key::Tab, Key::ShiftTab, Key::Enter] {
        press(settings, key, cx)?;
        ensure!(
            settings.update(cx, |view, window, cx| {
                view.number_edit
                    .as_ref()
                    .is_some_and(|edit| edit.invalid && edit.field == NumberField::ToastDelay)
                    && input.read(cx).focus.is_focused(window)
                    && input.read(cx).text() == "x"
                    && view.config.notification_overrides.delay_seconds() == initial
            })?,
            "native invalid number lost its text/focus or changed the saved delay"
        );
    }
    press(settings, Key::Escape, cx)?;
    ensure!(
        settings.update(cx, |view, _, _| view.number_edit.is_none()
            && view.config.notification_overrides.delay_seconds() == initial)?,
        "native Escape failed to cancel the invalid number"
    );
    Ok(())
}

pub(super) async fn verify(
    settings: WindowHandle<SettingsWindow>,
    source: WindowHandle<SheprdWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .context("native Settings fixture has no HOME")?;
    ensure!(
        std::env::var_os("XDG_CONFIG_HOME").map(std::path::PathBuf::from)
            == Some(home.join("config")),
        "native Settings keyboard checks require the private sandbox"
    );
    let path = cx
        .background_executor()
        .spawn(async { Config::local_path() })
        .await?;
    ensure!(
        path.starts_with(&home),
        "native Settings preferences escaped the sandbox"
    );
    let before = source.update(cx, |view, _, _| view.input_probe)?;
    foreground(settings, cx).await?;
    traversal_and_ring(settings, cx)?;
    switch_and_segments(settings, cx).await?;
    dropdown(settings, cx).await?;
    font_picker(settings, cx)?;
    invalid_number(settings, cx)?;
    section(settings, Section::General, cx)?;
    settle(settings, cx).await?;
    source.update(cx, |view, _, _| -> Result<()> {
        ensure!(
            view.input_probe.text == before.text
                && view.input_probe.keys == before.keys
                && view.input_probe.actions == before.actions,
            "native Settings keyboard events leaked into the source terminal"
        );
        Ok(())
    })??;
    eprintln!(
        // i18n: data
        "SIDEBAR native Settings keyboard PASS: AppKit Tab/Shift-Tab, painted focus ring, Enter/Space switches/segments/buttons, dropdown arrows/Escape/trigger restore, font modal trap, revealed invalid numeric retention; private preferences only; no global physical keyboard claim"
    );
    Ok(())
}
