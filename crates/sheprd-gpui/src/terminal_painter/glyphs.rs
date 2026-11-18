// ABOUTME: Reuses bounded font-dependent glyph shapes across terminal paints.
// ABOUTME: Replaces cold entries while protecting the current and previous paint's working set.
//! Shaped cell symbols, reused across frames.
//!
//! Only the font, its size and bold/italic change a symbol's glyphs; color is
//! applied when painting. Keying by color would shape one symbol once per
//! color, so truecolor output would exhaust the bounded cache and reshape
//! every frame. ASCII, most of any grid, is found by index without hashing.

use crate::terminal::{BOLD, ITALIC};
use gpui::{Font, FontStyle, ShapedLine};
use std::{cell::Cell, collections::HashMap, rc::Rc};

pub(super) const CACHE_LIMIT: usize = 4096;
const STYLES: usize = 4;

pub(super) struct GlyphCache {
    /// A shaped line is kilobytes, so the tables below hold indexes into this.
    lines: Vec<Cached>,
    /// `ascii[style * 128 + byte]`.
    ascii: Vec<Option<u32>>,
    other: [HashMap<Rc<str>, u32>; STYLES],
    pass: u64,
    cursor: usize,
    remaining: usize,
}

enum Key {
    Ascii(usize),
    Other { style: usize, symbol: Rc<str> },
}

struct Cached {
    key: Key,
    line: ShapedLine,
    used: Cell<u64>,
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            ascii: vec![None; STYLES * 128],
            other: Default::default(),
            pass: 0,
            cursor: 0,
            remaining: 0,
        }
    }
}

/// The modifier bits that change shaping, as an index.
pub(super) fn style(modifier: u16) -> usize {
    usize::from(modifier & BOLD != 0) | usize::from(modifier & ITALIC != 0) << 1
}

pub(super) fn style_modifier(style: usize) -> u16 {
    (if style & 1 != 0 { BOLD } else { 0 }) | if style & 2 != 0 { ITALIC } else { 0 }
}

/// The face a cached glyph `style` shapes in: bold cells draw a step heavier
/// than the configured weight rather than at a fixed bold.
pub(super) fn styled_font(font: &Font, style: usize) -> Font {
    let mut font = font.clone();
    let modifier = style_modifier(style);
    if modifier & BOLD != 0 {
        font.weight = crate::config::bold_font_weight(font.weight);
    }
    if modifier & ITALIC != 0 {
        font.style = FontStyle::Italic;
    }
    font
}

fn ascii_index(style: usize, symbol: &str) -> Option<usize> {
    match symbol.as_bytes() {
        [byte] if byte.is_ascii() => Some(style * 128 + usize::from(*byte)),
        _ => None,
    }
}

impl GlyphCache {
    pub(super) fn begin_paint(&mut self) {
        self.pass = self.pass.wrapping_add(1);
        // Search each existing slot at most once, even when every miss is uncached.
        self.remaining = self.lines.len();
    }

    pub(super) fn get(&self, style: usize, symbol: &str) -> Option<&ShapedLine> {
        let id = match ascii_index(style, symbol) {
            Some(index) => self.ascii[index]?,
            None => *self.other[style].get(symbol)?,
        };
        let cached = self.lines.get(usize::try_from(id).ok()?)?;
        cached.used.set(self.pass);
        Some(&cached.line)
    }

    /// Reuses a cold slot, or leaves this glyph uncached when recent paints use every slot.
    pub(super) fn room(&mut self) -> Option<usize> {
        if self.lines.len() < CACHE_LIMIT {
            return Some(self.lines.len());
        }
        while self.remaining > 0 {
            let slot = self.cursor;
            self.cursor = (self.cursor + 1) % self.lines.len();
            self.remaining -= 1;
            if self.pass.wrapping_sub(self.lines[slot].used.get()) > 1 {
                return Some(slot);
            }
        }
        None
    }

    /// `slot` comes from `room`; its old lookup must disappear before replacement.
    pub(super) fn insert(
        &mut self,
        slot: usize,
        style: usize,
        symbol: &str,
        line: ShapedLine,
    ) -> &ShapedLine {
        if let Some(cached) = self.lines.get(slot) {
            match &cached.key {
                Key::Ascii(index) => self.ascii[*index] = None,
                Key::Other { style, symbol } => {
                    self.other[*style].remove(symbol.as_ref());
                }
            }
        }
        // CACHE_LIMIT fits in u32; `room` bounds the slot below it.
        let id = u32::try_from(slot).unwrap_or(u32::MAX);
        let key = match ascii_index(style, symbol) {
            Some(index) => {
                self.ascii[index] = Some(id);
                Key::Ascii(index)
            }
            None => {
                let symbol: Rc<str> = symbol.into();
                self.other[style].insert(symbol.clone(), id);
                Key::Other { style, symbol }
            }
        };
        let cached = Cached {
            key,
            line,
            used: Cell::new(self.pass),
        };
        if slot == self.lines.len() {
            self.lines.push(cached);
        } else {
            self.lines[slot] = cached;
        }
        &self.lines[slot].line
    }

    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }

    #[cfg(any(test, feature = "integration-test"))]
    pub(super) fn len(&self) -> usize {
        self.lines.len()
    }

    /// Every cached symbol with its style index.
    #[cfg(any(test, feature = "integration-test"))]
    pub(super) fn iter(&self) -> impl Iterator<Item = (usize, String, &ShapedLine)> {
        let ascii = self.ascii.iter().enumerate().filter_map(|(index, id)| {
            let byte = u8::try_from(index % 128).ok()?;
            let symbol = char::from(byte).to_string();
            Some((
                index / 128,
                symbol,
                &self.lines.get(usize::try_from((*id)?).ok()?)?.line,
            ))
        });
        let other = self.other.iter().enumerate().flat_map(move |(style, ids)| {
            ids.iter().filter_map(move |(symbol, id)| {
                Some((
                    style,
                    symbol.to_string(),
                    &self.lines.get(usize::try_from(*id).ok()?)?.line,
                ))
            })
        });
        ascii.chain(other)
    }
}
