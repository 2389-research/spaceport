//! The Container tab of Settings > Cloud devices. Its fields save to
//! `[container]` in the local config. Engine reads and saved-row removal run
//! through `cloud::worker`, never on the UI thread.
// i18n: migrated

use super::{
    super::{Section, SettingsWindow},
    Hint, follow_config, input,
};
use crate::direction::Logical;
use crate::{
    cloud::{
        CloudProvider,
        worker::{self, Worker},
    },
    config::{Config, ContainerFields},
    container::{
        Settings,
        setup::{self, Device, Overview, State},
    },
    i18n::{self, Localize, fl},
    search_input::SearchInput,
};
use gpui::{prelude::*, *};

/// The editable `[container]` keys, in display order, with their labels'
/// message IDs and hints.
const FIELDS: [(&str, Hint); 1] = [(
    "settings-cloud-container-engine",
    Hint::Message("settings-cloud-container-engine-placeholder"),
)];

enum Update {
    Overview(crate::Result<Overview>),
    Done(crate::Result<()>),
}

pub(in crate::settings_window) struct ContainerCard {
    fields: [Entity<SearchInput>; 1],
    /// The config values the fields last loaded; see `follow_config`.
    loaded: ContainerFields,
    /// `None` until the first read finishes.
    overview: Option<Overview>,
    message: Option<String>,
    job: Option<Worker>,
    /// Tests build the card without asking the user's real engine.
    #[cfg(test)]
    reads_engine: bool,
}

impl ContainerCard {
    fn new(
        config: &Config,
        theme: &crate::config::Theme,
        cx: &mut Context<SettingsWindow>,
    ) -> Self {
        let loaded = ContainerFields::from_config(&config.container);
        let texts = texts(&loaded);
        let fields = std::array::from_fn(|index| {
            let hint = FIELDS[index].1;
            input(move || hint.text(), texts[index], config, theme, cx)
        });
        Self {
            fields,
            loaded,
            overview: None,
            message: None,
            job: None,
            #[cfg(test)]
            reads_engine: false,
        }
    }

    fn values(&self, cx: &App) -> ContainerFields {
        let text = |index: usize| self.fields[index].read(cx).text().trim().to_owned();
        ContainerFields { engine: text(0) }
    }

    fn devices(&self) -> &[Device] {
        self.overview
            .as_ref()
            .map_or(&[], |overview| &overview.devices)
    }
}

/// The editable values in `FIELDS` order.
fn texts(values: &ContainerFields) -> [&str; 1] {
    [&values.engine]
}

/// What the row says about a container: its engine and its state.
fn state_text(engine: &str, state: Option<State>) -> String {
    let engine = engine.to_owned();
    match state {
        Some(State::Running) => fl!("settings-cloud-container-running", engine = engine),
        Some(State::Stopped) => fl!("settings-cloud-container-stopped", engine = engine),
        Some(State::Missing) => fl!("settings-cloud-container-missing", engine = engine),
        None => fl!("settings-cloud-container-unreachable", engine = engine),
    }
}

impl SettingsWindow {
    pub(super) fn open_container_card(&mut self, cx: &mut Context<Self>) {
        if self.container_card.is_none() {
            self.container_card = Some(ContainerCard::new(&self.config, &self.theme, cx));
        }
        self.refresh_container(cx);
    }

    fn container_settings(&self) -> crate::container::Result<Settings> {
        Settings::resolve(&self.config.container)
    }

    fn container_job(
        &mut self,
        cx: &mut Context<Self>,
        work: impl FnOnce(&dyn Fn() -> bool, &dyn Fn(Update)) + Send + 'static,
    ) {
        let Some(card) = &mut self.container_card else {
            return;
        };
        card.job = None;
        match worker::spawn(
            "herdr-container-settings",
            cx,
            work,
            |this: &mut Self, update, cx| this.apply_container(update, cx),
        ) {
            Ok(job) => {
                if let Some(card) = &mut self.container_card {
                    card.job = Some(job);
                }
            }
            Err(error) => {
                tracing::error!(category = "container_worker", error_kind = ?error.kind(), "Could not start the container settings worker");
                if let Some(card) = &mut self.container_card {
                    // i18n: data
                    card.message = Some(
                        crate::cloud::Error::Worker(crate::cloud::Worker::Settings).localize(),
                    );
                }
            }
        }
    }

    fn refresh_container(&mut self, cx: &mut Context<Self>) {
        #[cfg(test)]
        if self
            .container_card
            .as_ref()
            .is_some_and(|card| !card.reads_engine)
        {
            return;
        }
        let settings = match self.container_settings() {
            Ok(settings) => settings,
            Err(error) => {
                if let Some(card) = &mut self.container_card {
                    card.message = Some(error.to_string());
                }
                return;
            }
        };
        self.container_job(cx, move |cancelled, send| {
            send(Update::Overview(
                setup::overview(&settings, cancelled).map_err(Into::into),
            ));
        });
    }

    fn apply_container(&mut self, update: Update, cx: &mut Context<Self>) {
        let Some(card) = &mut self.container_card else {
            return;
        };
        match update {
            Update::Overview(Ok(overview)) => card.overview = Some(overview),
            Update::Overview(Err(error)) => card.message = Some(error.to_string()),
            Update::Done(result) => {
                card.message = result.err().map(|error| error.to_string());
                self.refresh_container(cx);
            }
        }
    }

    fn save_container(&mut self, cx: &mut Context<Self>) {
        let Some(card) = &mut self.container_card else {
            return;
        };
        let values = card.values(cx);
        let existing = self.config.container.clone();
        card.message = None;
        // Reloading the saved config refreshes the engine line.
        self.save_native(move || Config::save_container(&values, &existing), cx);
    }

    /// Forget the device; the container keeps running on the engine.
    fn forget_container_device(&mut self, id: String, cx: &mut Context<Self>) {
        self.container_job(cx, move |_, send| {
            send(Update::Done(
                crate::cloud::remove(CloudProvider::Container, &id).map_err(Into::into),
            ));
        });
    }

    pub(super) fn container_config_changed(&mut self, cx: &mut Context<Self>) {
        if let Some(card) = &mut self.container_card {
            let loaded = ContainerFields::from_config(&self.config.container);
            follow_config(&card.fields, texts(&card.loaded), texts(&loaded), cx);
            card.loaded = loaded;
        }
        if self.container_card.is_some() && self.section == Section::CloudDevices {
            self.refresh_container(cx);
        }
    }

    pub(super) fn render_container_card(&self, cx: &mut Context<Self>) -> Div {
        let Some(card) = &self.container_card else {
            return div();
        };
        let ready = !self.busy();
        let status = match card.overview.as_ref().map(|overview| &overview.engine) {
            None => fl!("settings-cloud-container-checking"),
            Some(Ok((engine, version))) => fl!(
                "settings-cloud-container-engine-running",
                engine = engine.name(),
                version = version.clone()
            ),
            Some(Err(error)) => error.clone(),
        };
        let mut engine = self
            .cloud_card(crate::cloud::name(CloudProvider::Container).to_owned())
            .debug_selector(|| "cloud-container-card".into())
            .child(self.control_note(fl!("settings-cloud-container-note")))
            .child(div().min_w_0().child(status));
        for (index, (label, _)) in FIELDS.iter().enumerate() {
            engine = engine.child(
                self.cloud_field(i18n::message(label, &[]), card.fields[index].clone().into()),
            );
        }
        engine = engine.child(
            div().row().justify_end().child(
                self.cloud_button("cloud-container-save", fl!("settings-cloud-save"), ready)
                    .debug_selector(|| "cloud-container-save".into())
                    .when(ready, |button| {
                        button.on_click(cx.listener(|this, _, _, cx| this.save_container(cx)))
                    }),
            ),
        );
        if let Some(message) = &card.message {
            engine = engine.child(self.control_note(message.clone()));
        }
        let mut devices = self
            .cloud_card(fl!("settings-cloud-container-devices"))
            .debug_selector(|| "cloud-container-devices".into())
            .child(self.control_note(fl!("settings-cloud-container-devices-note")));
        if card.devices().is_empty() {
            devices = devices.child(self.control_note(fl!("settings-cloud-none-yet")));
        }
        for (index, device) in card.devices().iter().enumerate() {
            devices = devices.child(self.cloud_device_row_with(
                index,
                &device.saved,
                state_text(&device.saved.account, device.state),
                ready,
                Self::forget_container_device,
                [],
                cx,
            ));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .min_w_0()
            .child(engine)
            .child(devices)
    }
}

#[cfg(test)]
mod tests;
