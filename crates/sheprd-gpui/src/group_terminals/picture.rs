//! A group's terminal picture, painted as a cached view of its own.
//!
//! A group that shows another group's tab, or a tab through a parked
//! connection, paints a picture rather than the window's own cells. Painting
//! it in the canvas its element builds would re-enter the grid on every frame
//! of the window, a moving mark's included, while the window's own surface
//! replays its cached regions and paints nothing; see
//! `docs/notes-mirror-canvas-cache.md`. The picture is a view instead, as a
//! region is: while what it paints has not changed, GPUI replays its scene,
//! and a change takes a fresh view, which has no scene to replay.

use crate::{
    browser::GroupId,
    presentation::Picture,
    terminal::CellMetrics,
    terminal_painter::{Ground, ImageTarget, PlacedImages, TerminalPainter},
};
use gpui::{prelude::*, *};
use std::{cell::RefCell, rc::Rc, sync::Arc};

/// Everything besides the cells that changes how a picture paints.
#[derive(Clone, PartialEq)]
pub(crate) struct PictureLook {
    pub(crate) cell_height: f32,
    pub(crate) cell_width: f32,
    pub(crate) font: Font,
    pub(crate) ground: Ground,
}

/// One group's picture and the view painting it.
pub(crate) struct GroupPicture {
    group: GroupId,
    look: PictureLook,
    picture: Option<Picture>,
    view: Entity<PictureView>,
}

impl GroupPicture {
    pub(crate) fn new(
        group: GroupId,
        picture: Option<Picture>,
        look: PictureLook,
        view: Entity<PictureView>,
    ) -> Self {
        Self {
            group,
            look,
            picture,
            view,
        }
    }

    pub(crate) fn group(&self) -> GroupId {
        self.group
    }

    pub(crate) fn view(&self) -> &Entity<PictureView> {
        &self.view
    }

    /// Whether `picture` through `look` paints exactly what this view already
    /// painted. A picture placing images never does: a texture no paint looks
    /// up is released while a replayed scene would still sample it.
    pub(crate) fn paints_like(&self, picture: &Option<Picture>, look: &PictureLook) -> bool {
        let unchanged = match (&self.picture, picture) {
            (None, None) => true,
            (Some(old), Some(new)) => {
                old.frame.graphics.placements.is_empty()
                    && Arc::ptr_eq(&old.frame, &new.frame)
                    && Arc::ptr_eq(&old.images, &new.images)
            }
            _ => false,
        };
        unchanged && self.look == *look
    }

    /// Keeps this view for `picture`, which paints the same cells, without
    /// notifying it: a notified view is drawn again, and only the next frame
    /// would show it. The view holds what this frame painted, so the next
    /// frame compares against the newest picture.
    pub(crate) fn adopt(
        mut self,
        picture: Option<Picture>,
        look: PictureLook,
        cx: &mut App,
    ) -> Self {
        self.picture = picture.clone();
        self.look = look.clone();
        self.view.update(cx, |view, _| {
            view.look = look;
            view.picture = picture;
        });
        self
    }
}

/// One group's picture, painted over the box its element gives it.
pub(crate) struct PictureView {
    painter: Rc<RefCell<TerminalPainter>>,
    look: PictureLook,
    picture: Option<Picture>,
}

impl PictureView {
    pub(crate) fn new(
        painter: Rc<RefCell<TerminalPainter>>,
        picture: Option<Picture>,
        look: PictureLook,
    ) -> Self {
        Self {
            painter,
            look,
            picture,
        }
    }
}

impl Render for PictureView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let painter = self.painter.clone();
        let look = self.look.clone();
        let picture = self.picture.clone();
        canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let Some(Picture {
                    frame: surface,
                    images,
                }) = &picture
                else {
                    return;
                };
                let metrics = CellMetrics::new(look.cell_width, look.cell_height);
                let mut painter = painter.borrow_mut();
                // A frame wider than the group stays inside it.
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    painter.paint_frame(
                        &surface.frame,
                        bounds.origin,
                        Some(bounds.size),
                        look.ground,
                        look.cell_width,
                        &look.font,
                        &[],
                        &surface.panes,
                        None,
                        Some(PlacedImages {
                            placements: &surface.graphics.placements,
                            images,
                            target: ImageTarget::Main,
                        }),
                        window,
                        cx,
                    );
                    if let Some(popup) = &surface.popup {
                        let offset =
                            crate::terminal::popup_origin(&surface.frame, &popup.frame, metrics);
                        painter.paint_frame(
                            &popup.frame,
                            bounds.origin + offset,
                            None,
                            Ground::Opaque,
                            look.cell_width,
                            &look.font,
                            &[],
                            &[],
                            None,
                            Some(PlacedImages {
                                placements: &surface.graphics.placements,
                                images,
                                target: ImageTarget::Popup(&popup.terminal_id),
                            }),
                            window,
                            cx,
                        );
                    }
                });
            },
        )
        .size_full()
    }
}
