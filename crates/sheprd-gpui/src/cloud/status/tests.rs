use super::*;
use sheprd_i18n::{Locale, Pseudo};

fn steps() -> Vec<Step> {
    vec![
        Step::Creating,
        Step::Starting,
        Step::Building("pending".into()),
        Step::CheckingHerdr,
        Step::Image(Progress {
            step: Some((2, 4)),
            status: "RUN make".into(),
        }),
        Step::InstallingHerdr,
    ]
}

#[test]
fn image_progress_without_a_step_keeps_its_status() {
    let quiet = Progress {
        step: None,
        status: "building".into(),
    };
    assert_eq!(build(&quiet), "building");
}

#[test]
fn every_step_is_a_message_under_en_xa() {
    i18n::show(&Pseudo::Accented.locale());
    let mut shown: Vec<_> = steps().iter().map(step).collect();
    if let Some(&provider) = CloudProvider::ALL.first() {
        shown.push(contacting(provider));
    }
    i18n::show(&Locale::FALLBACK);
    for line in shown {
        assert!(sheprd_i18n::pseudo::is_marked(&line), "{line}");
    }
}
