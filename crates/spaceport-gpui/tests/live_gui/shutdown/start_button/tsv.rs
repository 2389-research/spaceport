// ABOUTME: Locates one complete Start Herdr text line in OCR from the private native window.
// ABOUTME: Checks word order and pixel bounds before returning a point inside the painted label.
use anyhow::{Context, Result, ensure};

const HEADER: &str =
    "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext";

struct Word<'a> {
    text: &'a str,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

fn parse_lines(tsv: &str, size: (u32, u32)) -> Result<Vec<Vec<Word<'_>>>> {
    let mut rows = tsv.lines();
    ensure!(
        rows.next() == Some(HEADER),
        "unexpected Tesseract TSV header"
    );
    let mut hierarchy = [0u32; 5];
    let mut lines: Vec<Vec<Word>> = Vec::new();
    for row in rows {
        let fields: [&str; 12] = row
            .split('\t')
            .collect::<Vec<_>>()
            .try_into()
            .ok()
            .context("expected twelve Tesseract TSV fields")?;
        let mut values = [0u32; 10];
        for (value, field) in values.iter_mut().zip(&fields) {
            *value = field
                .parse()
                .context("invalid Tesseract index or geometry")?;
        }
        let [
            level,
            page,
            block,
            paragraph,
            line,
            number,
            left,
            top,
            width,
            height,
        ] = values;
        ensure!(
            (1..=5).contains(&level) && page == 1,
            "unexpected OCR level or page"
        );
        // Each new layout row increments its own number and resets its descendants.
        let depth = level as usize - 1;
        ensure!(
            hierarchy[..depth].iter().all(|id| *id > 0),
            "OCR row without its parent"
        );
        hierarchy[depth] += 1;
        hierarchy[depth + 1..].fill(0);
        ensure!(
            hierarchy == [page, block, paragraph, line, number],
            "unordered OCR hierarchy"
        );
        let right = left.checked_add(width).context("OCR box overflow")?;
        let bottom = top.checked_add(height).context("OCR box overflow")?;
        ensure!(
            width > 0 && height > 0 && right <= size.0 && bottom <= size.1,
            "OCR box outside window"
        );
        let confidence: f32 = fields[10].parse()?;
        let text = fields[11];
        if level == 1 {
            ensure!(
                (left, top) == (0, 0) && (width, height) == size,
                "OCR page dimensions differ from capture"
            );
        }
        if level < 5 {
            ensure!(
                confidence == -1.0 && text.is_empty(),
                "invalid OCR layout row"
            );
            if level == 4 {
                lines.push(Vec::new());
            }
            continue;
        }
        ensure!(
            (0.0..=100.0).contains(&confidence),
            "invalid OCR word confidence"
        );
        let words = lines.last_mut().context("word without OCR line")?;
        words.push(Word {
            text,
            left,
            top,
            right,
            bottom,
        });
    }

    Ok(lines)
}

pub(super) fn locate(tsv: &str, size: (u32, u32)) -> Result<(u32, u32)> {
    let lines = parse_lines(tsv, size)?;
    let mut click = None;
    for words in &lines {
        for pair in words.windows(2) {
            let first = &pair[0];
            let second = &pair[1];
            if !first.text.eq_ignore_ascii_case("Start")
                || !second.text.eq_ignore_ascii_case("Herdr")
            {
                continue;
            }
            ensure!(
                words.len() == 2 && click.is_none(),
                "ambiguous Start Herdr label"
            );
            let gap = second
                .left
                .checked_sub(first.right)
                .context("overlapping or reversed label words")?;
            let height = (first.bottom - first.top).max(second.bottom - second.top);
            ensure!(
                gap <= height && first.top.max(second.top) < first.bottom.min(second.bottom),
                "separated label words"
            );
            // The first word is inside the button even when the glyph gap is wide.
            click = Some((
                first.left + (first.right - first.left) / 2,
                first.top + (first.bottom - first.top) / 2,
            ));
        }
    }
    click.context("no complete Start Herdr label in the native window")
}

#[cfg(test)]
#[path = "tests/tsv.rs"]
mod tests;
