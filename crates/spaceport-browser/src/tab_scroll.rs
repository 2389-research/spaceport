//! A group's tab strip scrolls sideways once its tabs outgrow it, as an
//! editor's does: the wheel scrolls it, the chosen tab is brought into view,
//! and a thin thumb along its foot shows where the view sits and drags it.
use super::{GroupId, Pick};
use gpui::{Pixels, ScrollHandle, point, px};
use std::collections::HashMap;

/// The frames a strip spends bringing a newly chosen tab into view.
const MAX_TRIES: u8 = 4;

/// What a thumb drags: the strip it scrolls.
#[derive(Clone, Copy, Debug)]
pub struct ThumbDrag(pub GroupId);

use spaceport_widgets::thumb::Thumb;

#[derive(Default)]
struct Strip {
    handle: ScrollHandle,
    /// The tab last brought into view, and how wide it was then, so a strip
    /// the user scrolled away stays put until the choice changes, or the tab
    /// grows past what was brought into view.
    revealed: Option<(Pick, f32)>,
    /// Where on the thumb the pointer took hold of it.
    grab: f32,
    /// The frames spent bringing `revealed` into view; at `MAX_TRIES` it is
    /// done, so a strip the user scrolls away stays put.
    tries: u8,
}

impl Strip {
    /// Whether the strip's last layout shows the tab at `index` whole, or
    /// from its left edge when it is wider than the strip.
    fn shows(&self, index: usize, width: f32) -> bool {
        let Some(tab) = self.handle.bounds_for_item(index) else {
            return false;
        };
        let view = self.handle.bounds();
        if view.size.width <= Pixels::ZERO {
            return false;
        }
        let offset = self.handle.offset().x;
        let slack = px(0.5);
        tab.size.width >= px(width) - slack
            && tab.left() + offset >= view.left() - slack
            && (tab.right() + offset <= view.right() + slack || tab.size.width > view.size.width)
    }
}

#[derive(Default)]
pub struct TabScroll {
    strips: HashMap<GroupId, Strip>,
}

impl TabScroll {
    /// What tracks `group`'s strip.
    pub fn handle(&mut self, group: GroupId) -> ScrollHandle {
        self.strips.entry(group).or_default().handle.clone()
    }

    /// Brings the tab at `index`, `pick`, into view when it is newly chosen,
    /// when its `width` changes, or every frame while `growing`, so a tab
    /// opening at the far end is followed until it has its full width. A
    /// page's title comes in after its tab opened, often once it is whole,
    /// and its tab then widens past the strip's edge, taking its close
    /// button out of view. Whether the strip needs another frame to do it.
    pub fn reveal(
        &mut self,
        group: GroupId,
        pick: &Pick,
        index: usize,
        width: f32,
        growing: bool,
    ) -> bool {
        let strip = self.strips.entry(group).or_default();
        let same = strip
            .revealed
            .as_ref()
            .is_some_and(|(revealed, at)| revealed == pick && (at - width).abs() < 0.5);
        if !same {
            strip.revealed = Some((pick.clone(), width));
            strip.tries = 0;
        } else if !growing && strip.tries >= MAX_TRIES {
            return false;
        }
        // GPUI scrolls to an item by the strip's last layout, before this
        // frame's; a strip not yet laid out, or since resized, can miss, so
        // it tries again until the last layout shows the tab. The frame that
        // changes the choice or the width lays the tab out anew, so the last
        // layout cannot yet say it shows.
        if same && strip.shows(index, width) && !growing {
            strip.tries = MAX_TRIES;
            return false;
        }
        strip.handle.scroll_to_item(index);
        // Animation frames do not spend the bounded attempts needed once
        // the tab reaches its full width in the following layout.
        strip.tries = if growing { 0 } else { strip.tries + 1 };
        strip.tries < MAX_TRIES || growing
    }

    /// The thumb `group`'s strip draws, as the strip last laid out.
    pub fn thumb(&self, group: GroupId) -> Option<Thumb> {
        let handle = &self.strips.get(&group)?.handle;
        Thumb::of(
            f32::from(handle.bounds().size.width),
            f32::from(handle.max_offset().x),
            -f32::from(handle.offset().x),
        )
    }

    /// Takes hold of `group`'s thumb with the pointer at `x`.
    pub fn grab(&mut self, group: GroupId, x: Pixels) {
        let Some(thumb) = self.thumb(group) else {
            return;
        };
        let Some(strip) = self.strips.get_mut(&group) else {
            return;
        };
        strip.grab = f32::from(x - strip.handle.bounds().left()) - thumb.left;
    }

    /// Moves `group`'s thumb with the pointer at `x`; whether it scrolled.
    pub fn drag(&mut self, group: GroupId, x: Pixels) -> bool {
        let Some(thumb) = self.thumb(group) else {
            return false;
        };
        let Some(strip) = self.strips.get(&group) else {
            return false;
        };
        let bounds = strip.handle.bounds();
        let left = f32::from(x - bounds.left()) - strip.grab;
        let scrolled = thumb.scrolled_at(
            f32::from(bounds.size.width),
            f32::from(strip.handle.max_offset().x),
            left,
        );
        let offset = strip.handle.offset();
        if (f32::from(offset.x) + scrolled).abs() < 0.5 {
            return false;
        }
        strip.handle.set_offset(point(px(-scrolled), offset.y));
        true
    }

    /// Forgets the strips of groups that are gone.
    pub fn retain(&mut self, live: impl Fn(GroupId) -> bool) {
        self.strips.retain(|group, _| live(*group));
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use core::prelude::v1::test;

    mod reveal;
}
