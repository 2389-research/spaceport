// ABOUTME: Select the development identity or an explicit macOS preview build.
// ABOUTME: Preview signing does not enable the production release or credential policy.

pub fn select<'a>(version: Option<&'a str>, target_os: &str) -> Option<(&'static str, &'a str)> {
    let Some(version) = version else {
        return Some((app_id(include_str!("development-app-id"))?, "dev"));
    };
    let (date, counter) = version.split_once('.')?;
    let positive_digits = |part: &str| {
        !part.starts_with('0') && !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())
    };
    if target_os != "macos"
        || date.len() != 8
        || !positive_digits(date)
        || !positive_digits(counter)
    {
        return None;
    }
    Some((app_id(include_str!("macos-preview-app-id"))?, version))
}

/// Identity data files allow a conventional final newline, but no empty segments
/// or characters that could become a second build-script directive.
pub fn app_id(value: &str) -> Option<&str> {
    let value = value.trim();
    (value.contains('.')
        && value.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        }))
    .then_some(value)
}
