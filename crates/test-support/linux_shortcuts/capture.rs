// ABOUTME: Decodes bounded terminal byte receipts from the actual pane screen.
// ABOUTME: Requires matching nonce, case, completion marker, and exact byte count.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Capture {
    // keep-sorted start
    Complete(Vec<u8>),
    Malformed,
    Pending,
    // keep-sorted end
}

pub(crate) fn marker(nonce: &str, id: &str, suffix: &str) -> String {
    format!("PANE_{nonce}_{id}_{suffix}")
}

pub(crate) fn read(screen: &str, nonce: &str, id: &str, count: usize) -> Capture {
    if !(1..=64).contains(&count) {
        return Capture::Malformed;
    }
    let Some((_, tail)) = screen.split_once(&marker(nonce, id, "READY")) else {
        return Capture::Pending;
    };
    let Some((body, _)) = tail.split_once(&marker(nonce, id, "DONE")) else {
        return Capture::Pending;
    };
    let mut bytes = Vec::with_capacity(count);
    for word in body.split_whitespace() {
        if word.len() != 2 || bytes.len() == count {
            return Capture::Malformed;
        }
        let Ok(byte) = u8::from_str_radix(word, 16) else {
            return Capture::Malformed;
        };
        bytes.push(byte);
    }
    if bytes.len() == count {
        Capture::Complete(bytes)
    } else {
        Capture::Malformed
    }
}

#[cfg(test)]
#[path = "capture/tests.rs"]
mod tests;
