// ABOUTME: Verifies real Core preference writes and restart-only native Settings behavior.
// ABOUTME: Captures private fixture windows without connecting to or starting Core.
use super::*;
use crate::config::preferences::Preference;

pub(crate) async fn verify(source: WindowHandle<SheprdWindow>, cx: &mut AsyncApp) -> Result<()> {
    let theme = if std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_LIGHT").is_some() {
        "Catppuccin Latte" // i18n: data
    } else {
        "Catppuccin Mocha" // i18n: data
    };
    let config = source.update(cx, |view, _, _| {
        view.config.theme = theme.into();
        view.config.clone()
    })?;
    cx.background_spawn(async move { config.save_theme(theme) })
        .await?;
    let weak = source.update(cx, |_, _, cx| cx.entity().downgrade())?;
    cx.update(|cx| {
        open_with(weak, cx, |view, _, cx| {
            view.section = Section::General;
            view.initialize_controls(cx);
        });
    });
    let settings = cx
        .update(|cx| cx.default_global::<SettingsWindowHandle>().window)
        .context("Core opt-in Settings window did not open")?;
    settings.update(cx, |view, _, _| view.section = Section::General)?;
    for saved in [false, true, false] {
        settings.update(cx, |view, _, cx| {
            view.save_preference(Preference::HerdrCoreEnabled(saved), cx)
        })?;
        settle_preferences(settings, cx).await?;
        let persisted = cx
            .background_spawn(async { Config::load_startup() })
            .await?;
        ensure!(
            persisted.herdr_core_enabled == saved,
            "Core preference was not persisted"
        );
        settings.update(cx, |view, _, cx| -> Result<()> {
            ensure!(
                view.config.herdr_core_enabled == saved,
                "Settings did not reload its saved Core preference"
            );
            ensure!(
                !crate::core_feature::enabled(cx),
                "saving Core preference changed the running app"
            );
            ensure!(
                crate::core_feature::restart_required(saved, cx) == saved,
                "Core restart hint disagrees with startup"
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
                    "core-{theme}-{width:.0}-{}.png",
                    if saved { "pending" } else { "off" }
                ));
                cx.background_spawn(async move {
                    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
                    let mut png = Vec::new();
                    image.write_with_encoder(PngEncoder::new_with_quality(
                        &mut png,
                        CompressionType::Fast,
                        FilterType::Sub,
                    ))?;
                    std::fs::write(path, png)?;
                    anyhow::Ok(())
                })
                .await?;
            }
        }
    }
    source.update(cx, |view, window, cx| -> Result<()> {
        ensure!(
            !view.core_enabled(),
            "source window changed runtime after saving"
        );
        view.config.herdr_2_terminals = true;
        view.live.missing_installation = true;
        view.poll_core_terminals(window, cx);
        ensure!(
            matches!(
                view.core_terminals.hub.state(),
                crate::core_terminals::State::Idle
            ),
            "disabled Core attempted a join or fallback"
        );
        Ok(())
    })??;
    ask::verify(settings, source, cx).await?;
    settings.update(cx, |_, window, _| window.remove_window())?;
    // i18n: data
    eprintln!("CORE OPT-IN native PASS: persisted edits, restart boundary and disabled fallback");
    Ok(())
}
