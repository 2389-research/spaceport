// ABOUTME: Exercises the Tesseract word and line contract used for native button selection.
// ABOUTME: Covers ambiguous labels, split lines, malformed input and misaligned or invalid boxes.
use super::*;

// Tesseract's level 1 page and level 2/3/4 layout rows precede its level 5 words.
const PAGE: &str = "1\t1\t0\t0\t0\t0\t0\t0\t640\t600\t-1\t\n";
const LINE: &str = "2\t1\t1\t0\t0\t0\t40\t80\t100\t20\t-1\t\n3\t1\t1\t1\t0\t0\t40\t80\t100\t20\t-1\t\n4\t1\t1\t1\t1\t0\t40\t80\t100\t20\t-1\t\n";
const START: &str = "5\t1\t1\t1\t1\t1\t40\t80\t35\t14\t94.829132\tStart\n";
const HERDR: &str = "5\t1\t1\t1\t1\t2\t80\t81\t40\t14\t91.063477\tHerdr\n";

fn tsv() -> String {
    format!("{HEADER}\n{PAGE}{LINE}{START}{HERDR}")
}

#[test]
fn selects_inside_the_first_word_of_the_exact_button_label() {
    assert_eq!(locate(&tsv(), (640, 600)).unwrap(), (57, 87));
}

#[test]
fn refuses_split_labels_extra_text_and_multiple_matching_lines() {
    let original = tsv();
    let another = format!("{LINE}{START}{HERDR}")
        .lines()
        .map(|row| row.replacen("\t1\t1\t", "\t1\t2\t", 1) + "\n")
        .collect::<String>();
    for invalid in [
        format!("{original}{another}"),
        original.replace("Start", "Restart"),
        original.replace(
            HERDR,
            &HERDR.replace("5\t1\t1\t1\t1\t2", "5\t1\t1\t1\t2\t1"),
        ),
        format!("{original}5\t1\t1\t1\t1\t3\t123\t81\t16\t14\t98.0\tnow\n"),
    ] {
        assert!(locate(&invalid, (640, 600)).is_err());
    }
}

#[test]
fn refuses_invalid_dimensions_rows_and_word_geometry() {
    let original = tsv();
    for invalid in [
        original.replace("word_num", "word"),
        original.replace(PAGE, ""),
        original.replace("640\t600", "900\t600"),
        original.replace("35\t14", "0\t14"),
        original.replace("35\t14", "999\t14"),
        original.replace("80\t81", "10\t81"),
        original.replace("80\t81", "80\t160"),
        original.replace("94.829132", "NaN"),
        format!("{original}broken\n"),
    ] {
        assert!(locate(&invalid, (640, 600)).is_err());
    }
}
