//! Cloud machines as devices: `sheprd_remote::cloud`, plus what the window
//! drives here: existing device reads and idle checks.
// i18n: migrated

pub(crate) mod worker;

pub(crate) use sheprd_remote::cloud::*;

/// Stop idle containers. Each reads its table as the first frame does, with no
/// maintenance, and only when one of its devices is saved.
pub(crate) fn install_idle_checks(cx: &mut gpui::App) {
    #[cfg(feature = "container")]
    every(cx, crate::container::idle::CHECK, || {
        crate::container::idle::check_all(|| {
            crate::config::Config::load_startup()
                .ok()
                .map(|config| config.container)
        })
    });
    let _ = cx;
}

#[cfg(feature = "container")]
fn every(cx: &mut gpui::App, interval: std::time::Duration, check: impl Fn() + Send + 'static) {
    let timer = cx.background_executor().clone();
    cx.background_executor()
        .spawn(async move {
            loop {
                timer.timer(interval).await;
                check();
            }
        })
        .detach();
}
