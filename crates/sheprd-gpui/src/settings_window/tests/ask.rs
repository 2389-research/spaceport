// ABOUTME: Exercises the Ask toggle and agent picker through real Settings controls.
// ABOUTME: Keeps runtime enablement unchanged while saved choices reload after clicks.
use super::*;
use crate::{
    config::preferences::Preference,
    sidebar::layout_tests::{fixture_window, full_draw},
};

#[gpui::test]
fn ask_toggle_and_agent_picker_save_independent_choices(cx: &mut TestAppContext) {
    let source = cx.add_window(fixture_window);
    let disk = Arc::new(Mutex::new(crate::config::chat::ChatConfig::default()));
    let written = disk.clone();
    let loaded = disk.clone();
    let io = controls::preferences::PreferenceIo::with_load(
        move |edit| {
            let mut chat = written.lock().unwrap();
            match edit {
                Preference::AskEnabled(value) => chat.ask_enabled = value,
                Preference::AskAgent(value) => chat.ask_agent = value,
                _ => panic!("unexpected Ask edit"),
            }
        },
        move || {
            let mut result = fixture();
            result.config.chat = loaded.lock().unwrap().clone();
            Ok(result)
        },
    );
    let weak = cx.update(|cx| source.update(cx, |_, _, cx| cx.weak_entity()).unwrap());
    let settings = cx.add_window(move |_, cx| {
        let mut view = SettingsWindow::new(weak, cx);
        view.controls.preference_io = Some(io);
        view.initialize_controls(cx);
        view.section = Section::General;
        view
    });
    let click = |selector: &'static str, cx: &mut TestAppContext| {
        let cx = &mut VisualTestContext::from_window(settings.into(), cx);
        cx.simulate_resize(size(px(960.), px(2200.)));
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let control = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(control.center(), Default::default());
        cx.run_until_parked();
    };
    click("general-ask-enabled-toggle", cx);
    assert!(disk.lock().unwrap().ask_enabled);
    click("general-ask-agent-control-1", cx);
    assert_eq!(
        disk.lock().unwrap().ask_agent,
        Some(crate::config::chat::AskAgent::Pi)
    );
    click("general-ask-enabled-toggle", cx);
    let chat = disk.lock().unwrap();
    assert!(!chat.ask_enabled);
    assert_eq!(chat.ask_agent, Some(crate::config::chat::AskAgent::Pi));
    drop(chat);
    source
        .update(cx, |source, _, cx| {
            assert!(!source.activity_panel_listed(crate::sidebar::ActivityPanel::Ask));
            assert!(crate::ask::status(cx).is_none());
        })
        .unwrap();
}
