#![allow(clippy::unimplemented, clippy::unwrap_used)]
//! Fixtures shared by this crate's tests.

use crate::ui::{Labels, MESSAGE_IDS, SWATCH_NONE, Source, Unit, VALUE_MARK};

/// The app's words, as the English catalog words them, so a test checks the
/// layout the app draws. An id with no word stands for itself.
pub(crate) fn labels() -> Labels {
    Labels {
        message,
        number_range,
        follow_herdr,
    }
}

fn message(id: &'static str) -> String {
    match id {
        id if id == Source::Herdr.label_id() => "Herdr".to_owned(),
        id if id == Source::Spaceport.label_id() => "Spaceport".to_owned(),
        id if id == Unit::Percent.label_id() => "%".to_owned(),
        id if id == Unit::Pixels.label_id() => "px".to_owned(),
        id if id == Unit::Seconds.label_id() => "s".to_owned(),
        id if id == SWATCH_NONE => "none".to_owned(),
        id => id.to_owned(),
    }
}

fn number_range(low: f32, high: f32) -> String {
    format!("{low} to {high}")
}

fn follow_herdr() -> String {
    format!("Use Herdr setting \u{b7} {VALUE_MARK}")
}

/// Every id the kit names is one the app is told to word, so no word can go
/// missing from a language.
#[test]
fn every_id_the_kit_names_is_listed_for_the_app() {
    for id in [
        SWATCH_NONE,
        Source::Herdr.label_id(),
        Source::Spaceport.label_id(),
        Unit::Percent.label_id(),
        Unit::Pixels.label_id(),
        Unit::Seconds.label_id(),
    ] {
        assert!(MESSAGE_IDS.contains(&id), "{id} is not in MESSAGE_IDS");
    }
}
