// ABOUTME: Checks Ask settings persistence and the disabled native startup boundary.
// ABOUTME: Captures actual Settings windows with private fixture state and no agent calls.
use super::*;
use crate::config::{chat::AskAgent, preferences::Preference};

pub(super) async fn verify(
    settings: WindowHandle<SettingsWindow>,
    source: WindowHandle<SheprdWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    for saved in [false, true, false] {
        settings.update(cx, |view, _, cx| {
            view.save_preference(Preference::AskEnabled(saved), cx)
        })?;
        settle_preferences(settings, cx).await?;
        settings.update(cx, |view, _, cx| {
            view.save_preference(Preference::AskAgent(Some(AskAgent::Pi)), cx)
        })?;
        settle_preferences(settings, cx).await?;
        let persisted = cx
            .background_spawn(async { Config::load_startup() })
            .await?;
        ensure!(
            persisted.chat.ask_enabled == saved,
            "Ask opt-in did not persist"
        );
        ensure!(
            persisted.chat.ask_agent == Some(AskAgent::Pi),
            "Ask agent did not persist"
        );
        settings.update(cx, |view, _, cx| -> Result<()> {
            ensure!(
                view.config.chat.ask_enabled == saved,
                "Ask switch did not reload"
            );
            ensure!(
                view.config.chat.ask_agent == Some(AskAgent::Pi),
                "Ask agent did not reload"
            );
            ensure!(
                crate::ask::Feature::restart_required(saved, cx) == saved,
                "Ask restart hint disagrees with startup"
            );
            ensure!(
                crate::ask::status(cx).is_none(),
                "disabled Ask created an active controller"
            );
            Ok(())
        })??;
        source.update(cx, |view, _, _| -> Result<()> {
            ensure!(
                !view.activity_panel_listed(crate::sidebar::ActivityPanel::Ask),
                "saved Ask opt-in changed the running window"
            );
            Ok(())
        })??;
        if let Some(root) = std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_CAPTURE") {
            let root = std::path::PathBuf::from(root);
            let theme = if std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_LIGHT").is_some() {
                "light"
            } else {
                "dark"
            };
            for width in [960., 680.] {
                settings.update(cx, |_, window, _| window.resize(size(px(width), px(780.))))?;
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                let image = AnyWindowHandle::from(settings).update(cx, |_, window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                    window.render_to_image()
                })??;
                let path = root.join(format!(
                    "ask-{theme}-{width:.0}-{}.png",
                    if saved { "pending" } else { "off" }
                ));
                cx.background_spawn(async move { image.save(path) }).await?;
            }
        }
    }
    settings.update(cx, |view, _, cx| {
        view.save_preference(Preference::AskAgent(None), cx)
    })?;
    settle_preferences(settings, cx).await?;
    let persisted = cx
        .background_spawn(async { Config::load_startup() })
        .await?;
    ensure!(
        !persisted.chat.ask_enabled && persisted.chat.ask_agent.is_none(),
        "first-use choice did not persist independently"
    );
    eprintln!(
        // i18n: data
        "ASK SETTINGS native PASS: persisted toggle and agent; restart boundary; disabled controller"
    );
    Ok(())
}
