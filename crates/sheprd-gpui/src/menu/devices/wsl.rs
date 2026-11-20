// ABOUTME: Owns WSL setup and exact saved-distribution removal effects.
// ABOUTME: Keeps disposable discovery observers separate from accepted background writes.
//! WSL distributions as devices: the dialog that lists this machine's
//! distributions and saves one once Herdr answers inside it, and removal, which
//! only forgets the distribution here. Nothing here installs Herdr, edits a
//! distribution, or boots one the user did not pick.
use super::Page;
use crate::i18n::fl;
use crate::{SheprdWindow, endpoint::WSL_PREFIX, search_input::SearchInput};
use gpui::{prelude::*, *};
use sheprd_client::HostProbe;

mod paint;
mod render;
pub(super) use paint::{Paint, RemovalPaint};

pub(in crate::menu) struct WslSetup {
    distros: Distros,
    /// The distribution the user picked; the first unsaved one by default.
    selected: Option<String>,
    session: Entity<SearchInput>,
    step: Step,
    task: Option<Task<()>>,
    /// Why the last check failed, kept with this form when another Add Device
    /// source is shown.
    error: Option<String>,
    scroll: ScrollHandle,
    list_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
}

enum Distros {
    Loading,
    Ready(Vec<String>),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Step {
    Form,
    /// Probing the distribution, then saving it when Herdr answers.
    Checking(String),
}

/// What became of one add: saved, or why not, in the dialog's words.
enum Outcome {
    Saved,
    Refused(String),
}

impl SheprdWindow {
    /// Set the WSL form's error, which stays with that form.
    fn set_wsl_error(&mut self, error: Option<String>) {
        if let Some(setup) = &mut self.menu.wsl_setup {
            setup.error = error;
        }
    }

    /// Show the dialog and list the distributions in the background.
    pub(super) fn open_add_wsl(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let session = cx.new(SearchInput::new);
        session.update(cx, |input, cx| {
            input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
            input.set_placeholder("default", cx);
        });
        window.focus(&session.read(cx).focus.clone(), cx);
        let background = cx
            .background_executor()
            .spawn(async { sheprd_client::list_distros() });
        let task = cx.spawn(async move |this, cx| {
            let result = background.await;
            let _ = this.update(cx, |this, cx| {
                let saved = this.saved_distros();
                let Some(setup) = &mut this.menu.wsl_setup else {
                    return;
                };
                setup.distros = match result {
                    Ok(distros) => {
                        setup.selected = distros
                            .iter()
                            .find(|distro| !saved.contains(distro))
                            .cloned();
                        Distros::Ready(distros)
                    }
                    Err(error) => Distros::Failed(error.to_string()),
                };
                setup.task = None;
                cx.notify();
            });
        });
        self.menu.wsl_setup = Some(WslSetup {
            distros: Distros::Loading,
            selected: None,
            session,
            step: Step::Form,
            task: Some(task),
            error: None,
            scroll: ScrollHandle::new(),
            list_bounds: Default::default(),
        });
        self.set_wsl_error(None);
        self.menu.page = Some(Page::AddWsl);
    }

    /// The distributions already saved as devices.
    fn saved_distros(&self) -> Vec<String> {
        self.endpoints
            .iter()
            .filter_map(|endpoint| endpoint.id.strip_prefix(WSL_PREFIX))
            .map(str::to_owned)
            .collect()
    }

    pub(in crate::menu) fn add_wsl_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let saved = self.saved_distros();
        let Some(setup) = &mut self.menu.wsl_setup else {
            return;
        };
        if setup.session.read(cx).is_composing() {
            return;
        }
        match event.keystroke.key.as_str() {
            "enter" => self.submit_add_wsl(window, cx),
            "escape" => self.dismiss_menu(window, cx),
            key @ ("up" | "down") => {
                let Distros::Ready(distros) = &setup.distros else {
                    return;
                };
                // Only the distributions that can still be added, as a click allows.
                let distros: Vec<&String> = distros
                    .iter()
                    .filter(|distro| !saved.contains(distro))
                    .collect();
                if distros.is_empty() {
                    return;
                }
                let current = setup
                    .selected
                    .as_ref()
                    .and_then(|selected| distros.iter().position(|distro| *distro == selected));
                let next = match (current, key) {
                    (None, "up") => distros.len() - 1,
                    (None, _) => 0,
                    (Some(index), "up") => (index + distros.len() - 1) % distros.len(),
                    (Some(index), _) => (index + 1) % distros.len(),
                };
                setup.selected = Some(distros[next].to_owned());
                cx.notify();
            }
            _ => return,
        }
        cx.stop_propagation();
        window.prevent_default();
    }

    fn submit_add_wsl(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let saved = self.saved_distros();
        let Some(setup) = &mut self.menu.wsl_setup else {
            return;
        };
        if setup.step != Step::Form {
            return;
        }
        let Some(distro) = setup.selected.clone() else {
            self.set_wsl_error(Some(fl!("devices-wsl-pick")));
            cx.notify();
            return;
        };
        if saved.contains(&distro) {
            self.set_wsl_error(Some(fl!(
                "devices-wsl-already-added",
                distro = distro.as_str()
            )));
            cx.notify();
            return;
        }
        let session = match setup.session.read(cx).text().trim() {
            "" => "default".to_owned(),
            session => session.to_owned(),
        };
        setup.step = Step::Checking(distro.clone());
        setup.error = None;
        let background = cx
            .background_executor()
            .spawn(async move { add(&distro, &session) });
        setup.task = Some(cx.spawn_in(window, async move |this, cx| {
            let outcome = background.await;
            let _ = this.update_in(cx, |this, window, cx| {
                let Some(setup) = &mut this.menu.wsl_setup else {
                    return;
                };
                setup.task = None;
                match outcome {
                    // The catalog poll shows the device; nothing is left to
                    // say. If another source is shown, only this form goes.
                    Outcome::Saved if this.menu.page != Some(Page::AddWsl) => {
                        this.menu.wsl_setup = None;
                        cx.notify();
                    }
                    Outcome::Saved => this.dismiss_menu(window, cx),
                    Outcome::Refused(message) => {
                        setup.step = Step::Form;
                        setup.error = Some(message);
                        cx.notify();
                    }
                }
            });
        }));
        cx.notify();
    }

    /// Ask before forgetting a saved distribution.
    pub(crate) fn open_remove_wsl(
        &mut self,
        id: &str,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(distro) = id.strip_prefix(WSL_PREFIX) else {
            return;
        };
        if self.menu.removing_devices.contains(id) || !self.open_menu(window, cx) {
            return;
        }
        self.menu.anchor = anchor;
        self.menu.wsl_remove = Some(distro.to_owned());
        self.menu.page = Some(Page::RemoveWsl);
    }

    pub(in crate::menu) fn remove_wsl_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event.keystroke.key.as_str() {
            "enter" if event.is_held => {}
            "enter" => self.confirm_remove_wsl(window, cx),
            "escape" => self.dismiss_menu(window, cx),
            _ => return,
        }
        cx.stop_propagation();
        window.prevent_default();
    }

    /// Close at once and forget the distribution in the background; its header
    /// pulses until the catalog drops it, like an SSH device's removal.
    fn confirm_remove_wsl(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(distro) = self.menu.wsl_remove.clone() else {
            return;
        };
        let id = format!("{WSL_PREFIX}{distro}");
        let background = cx.background_executor().spawn({
            let distro = distro.clone();
            async move { sheprd_client::remove_wsl_host(false, &distro) }
        });
        self.menu.removing_devices.insert(id.clone());
        self.dismiss_menu(window, cx);
        cx.spawn(async move |this, cx| {
            let result = background.await;
            // From the app: the window may have closed by now.
            if result.is_ok() {
                cx.update(|cx| crate::notification_mutes::Mutes::forget_host(cx, &id));
            }
            let _ = this.update(cx, |this, cx| {
                if let Err(error) = result {
                    // The log's context.
                    // i18n: data
                    crate::storage_warning::warn_storage_failure("Remove WSL device", &error);
                    this.menu.removing_devices.remove(&id);
                    this.notices.show(
                        crate::notice::Cause::Command,
                        fl!(
                            "devices-wsl-remove-failed",
                            distro = distro.as_str(),
                            error = error.to_string(),
                        ),
                    );
                    cx.notify();
                }
            });
        })
        .detach();
    }
}

/// Check Herdr inside `distro` and save it when a compatible copy answers. A
/// stopped server is fine: the bridge starts it on the first connection.
/// Blocking: runs on the background executor.
fn add(distro: &str, session: &str) -> Outcome {
    match sheprd_client::probe_distro(distro, session) {
        Ok(HostProbe::Running | HostProbe::Stopped) => {
            match sheprd_client::add_wsl_host(false, distro, session) {
                Ok(()) => Outcome::Saved,
                Err(error) => {
                    // The log's context.
                    // i18n: data
                    crate::storage_warning::warn_storage_failure("Save WSL device", &error);
                    Outcome::Refused(fl!(
                        "devices-wsl-save-failed",
                        distro = distro,
                        error = error.to_string(),
                    ))
                }
            }
        }
        Ok(HostProbe::Missing) => Outcome::Refused(fl!("devices-wsl-missing", distro = distro)),
        Ok(HostProbe::Outdated) => Outcome::Refused(fl!("devices-wsl-outdated", distro = distro)),
        Ok(HostProbe::SshFailed(_)) => {
            Outcome::Refused(fl!("devices-wsl-no-answer", distro = distro))
        }
        Err(error) => Outcome::Refused(fl!(
            "devices-wsl-check-failed",
            distro = distro,
            error = error.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(in crate::menu) use tests::departure_fixture;
#[cfg(test)]
pub(in crate::menu) use tests::long_fixture;
