// ABOUTME: Captures usage display data without opaque report or input owners.
// ABOUTME: Shares borrowed reports for live drawing and concrete snapshots for departure.
use super::{
    model::{Provider, Report},
    text,
    ui::Ui,
};
use crate::{
    SheprdWindow,
    i18n::{Localize, fl},
};
use gpui::{AnyElement, Pixels, Point};
use sheprd_usage::details::{ClaudeAccount, OpenaiDaily};
use std::time::SystemTime;

pub(super) struct Metadata {
    pub(super) provider: Provider,
    pub(super) host: String,
    pub(super) updated: String,
    pub(super) errors: Vec<String>,
    pub(super) tabs: Vec<Provider>,
    pub(super) now: SystemTime,
    pub(super) offset: Point<Pixels>,
}

pub(super) struct ReportPaint {
    standard: Report,
    daily: Option<OpenaiDaily>,
    accounts: Option<Vec<ClaudeAccount>>,
}

impl From<&Report> for ReportPaint {
    fn from(report: &Report) -> Self {
        Self {
            standard: Report {
                provider: report.provider,
                account: report.account.clone(),
                windows: report.windows.clone(),
                balances: report.balances.clone(),
                sections: report.sections.clone(),
                // Arbitrary Any values can own resources; only copy supported data below.
                detail: None,
            },
            daily: report.detail::<OpenaiDaily>().map(|daily| OpenaiDaily {
                days: daily.days.clone(),
            }),
            accounts: report.detail::<Vec<ClaudeAccount>>().cloned(),
        }
    }
}

pub(crate) struct Paint {
    pub(super) metadata: Metadata,
    pub(super) report: Option<ReportPaint>,
}

#[derive(Clone, Copy)]
pub(super) enum ReportView<'a> {
    Current(&'a Report),
    Frozen(&'a ReportPaint),
}

impl ReportView<'_> {
    pub(super) fn standard(&self) -> &Report {
        match self {
            Self::Current(report) => report,
            Self::Frozen(paint) => &paint.standard,
        }
    }

    pub(super) fn render(&self, ui: &Ui) -> AnyElement {
        match self {
            Self::Current(report) => super::details::render(report, ui),
            Self::Frozen(paint) => super::details::render_known(
                &paint.standard,
                paint.daily.as_ref(),
                paint.accounts.as_deref(),
                ui,
            ),
        }
    }
}

impl SheprdWindow {
    pub(crate) fn usage_paint(&self, provider: Provider) -> Paint {
        let (metadata, report) = self.usage_metadata(provider);
        Paint {
            metadata,
            report: report.map(ReportPaint::from),
        }
    }

    pub(super) fn usage_metadata(&self, provider: Provider) -> (Metadata, Option<&Report>) {
        let now = SystemTime::now();
        let entry = self.usage.current();
        let readings = entry
            .map(|entry| entry.tabs(&self.config.usage))
            .unwrap_or_default();
        let reading = readings
            .iter()
            .copied()
            .find(|reading| reading.provider == provider);
        let report = reading.and_then(|reading| reading.report.as_ref());
        let metadata = Metadata {
            provider,
            host: self
                .endpoints
                .get(self.selected_endpoint)
                .map(|endpoint| endpoint.label.clone())
                .unwrap_or_default(),
            updated: entry
                .and_then(|entry| entry.updated)
                .map(|at| text::updated(now.duration_since(at).unwrap_or_default()))
                .unwrap_or_else(|| fl!("usage-not-updated-yet")),
            errors: reading
                .and_then(|reading| reading.error.as_deref())
                .into_iter()
                .chain(entry.and_then(|entry| entry.error.as_deref()))
                .map(Localize::localize)
                .collect(),
            tabs: readings
                .into_iter()
                .map(|reading| reading.provider)
                .collect(),
            now,
            offset: self.menu.usage_scroll.offset(),
        };
        (metadata, report)
    }
}

#[cfg(test)]
mod tests;
