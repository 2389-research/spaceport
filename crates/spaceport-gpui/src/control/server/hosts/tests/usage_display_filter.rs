// ABOUTME: Keeps the host usage CLI aligned with display filtering after polling removal.
// ABOUTME: Verifies observed zero survives while hidden reports remain stored unchanged.
use super::*;
use crate::usage::{
    Host, Reading, Usage,
    model::{Account, Kind, Report, Window},
    registry,
};

#[gpui::test]
fn hidden_reports_stay_out_of_host_usage(cx: &mut TestAppContext) {
    let (view, cx) = window(cx);
    view.update(cx, |view, _| {
        view.config.usage.hide_providers = vec!["claude".into()];
        view.usage = Usage::showing(
            Host::Local,
            ["claude", "codex"]
                .map(|id| {
                    let provider = registry::find(id).unwrap();
                    Reading {
                        provider,
                        report: Some(Report::new(
                            provider,
                            Account::default(),
                            vec![Window::new(Kind::Session, 0., None, None)],
                        )),
                        error: None,
                    }
                })
                .to_vec(),
        );
        let Response::Host {
            reply: Reply::Usage { usage, note, .. },
        } = usage(view, SystemTime::now())
        else {
            panic!("expected usage reply")
        };
        assert_eq!(
            usage
                .iter()
                .map(|reading| reading.provider.as_str())
                .collect::<Vec<_>>(),
            ["Codex"]
        );
        assert!(note.is_none());
        assert_eq!(view.usage.current().unwrap().readings.len(), 2);
    });
}

#[gpui::test]
fn unavailable_usage_does_not_guess_runtime_capabilities(cx: &mut TestAppContext) {
    let (view, cx) = window(cx);
    cx.update(|_, cx| {
        let Response::Host {
            reply: Reply::Usage {
                note: Some(note), ..
            },
        } = usage(view.read(cx), SystemTime::now())
        else {
            panic!("expected unavailable usage")
        };
        assert_eq!(
            note,
            "Plan usage is unavailable on Local: no authoritative usage feed is connected"
        );
        assert_eq!(
            crate::i18n::english("usage-core-unavailable-detail", &[]),
            "No authoritative usage feed is connected."
        );
    });
}
