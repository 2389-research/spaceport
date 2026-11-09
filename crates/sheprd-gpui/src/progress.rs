// ABOUTME: Describes file-transfer progress for the Windows taskbar.
// ABOUTME: Unknown totals stay busy until a real fraction is available.

/// How far along a bar is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Progress {
    /// No reliable fraction: a segment slides back and forth.
    Busy,
    /// A settled fraction.
    Fraction(f32),
}
