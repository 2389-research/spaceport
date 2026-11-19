// ABOUTME: Captures real native popup entry, outgoing paint, and release frames.
// ABOUTME: Uses QA slow motion and bounded PNG receipts without changing the clock or OS policy.
use super::*;
use crate::{menu::Page, motion};
use anyhow::ensure;
use base64::Engine as _;
use image::{
    RgbaImage,
    codecs::png::{CompressionType, FilterType, PngEncoder},
};

struct SlowMotion(bool);

struct Frame {
    pixels: RgbaImage,
    departure: Option<(Page, f32)>,
}

impl Drop for SlowMotion {
    fn drop(&mut self) {
        motion::set_slow(self.0);
    }
}

fn capture(handle: WindowHandle<SheprdWindow>, cx: &mut AsyncApp, blur: bool) -> Result<Frame> {
    AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<_> {
        let view = root
            .downcast::<SheprdWindow>()
            .map_err(|_| anyhow!("unexpected root"))?;
        if blur {
            window.blur(cx);
        }
        window.refresh();
        window.draw(cx).clear(cx);
        Ok(Frame {
            departure: view.read(cx).native_departure_frame(),
            pixels: window.render_to_image()?,
        })
    })?
}

async fn receipt(name: &str, phase: &str, pixels: RgbaImage, cx: &AsyncApp) -> Result<()> {
    ensure!(
        u64::from(pixels.width()) * u64::from(pixels.height()) <= 2_000_000,
        "native departure image exceeds pixel limit"
    );
    let png = cx
        .background_spawn(async move {
            let mut png = Vec::new();
            pixels.write_with_encoder(PngEncoder::new_with_quality(
                &mut png,
                CompressionType::Fast,
                FilterType::Sub,
            ))?;
            ensure!(png.len() <= 131_072, "native departure PNG exceeds 128 KiB");
            anyhow::Ok(png)
        })
        .await?;
    eprintln!(
        "DEPARTURE_PNG {name} {phase} {}",
        base64::engine::general_purpose::STANDARD.encode(png)
    );
    Ok(())
}

fn compare(
    base: &RgbaImage,
    active: &RgbaImage,
    middle: Option<&RgbaImage>,
    settled: &RgbaImage,
) -> Result<()> {
    ensure!(
        base.dimensions() == active.dimensions() && base.dimensions() == settled.dimensions(),
        "native departure dimensions changed"
    );
    let different = |a: &RgbaImage, b: &RgbaImage| {
        (base.height() / 4..base.height() * 9 / 10).any(|y| {
            (16..base.width().saturating_sub(16)).any(|x| a.get_pixel(x, y) != b.get_pixel(x, y))
        })
    };
    // Raster differences prove scene changes. Inspect the emitted images for body content;
    // dialog backdrop changes alone can satisfy these comparisons.
    ensure!(different(active, base), "active popup scene is missing");
    ensure!(
        !different(settled, base),
        "settled popup region differs from the clean closed baseline"
    );
    if let Some(middle) = middle {
        ensure!(
            base.dimensions() == middle.dimensions(),
            "native outgoing dimensions changed"
        );
        ensure!(
            different(middle, base) && different(middle, active),
            "outgoing popup has no distinct native paint"
        );
    }
    Ok(())
}

pub(super) async fn verify(handle: WindowHandle<SheprdWindow>, cx: &mut AsyncApp) -> Result<()> {
    crate::accessibility::policy::prime();
    let reduced = crate::accessibility::policy::reduce_motion();
    cx.update(|cx| cx.set_reduce_motion(reduced));
    let _slow = SlowMotion(motion::is_slow());
    motion::set_slow(true);
    handle.update(cx, |view, window, cx| {
        view.sidebar_visible = false;
        if let Some(snapshot) = &mut view.live.snapshot {
            let snapshot = Arc::make_mut(snapshot);
            // Sidebar collapse retains the compact rail. Keep its authored indicators still.
            for workspace in &mut snapshot.workspaces {
                workspace.agent_status = AgentStatus::Idle;
            }
            for tab in &mut snapshot.tabs {
                tab.agent_status = AgentStatus::Idle;
            }
            for agent in &mut snapshot.agents {
                agent.agent_status = AgentStatus::Idle;
            }
        }
        if let Some(endpoint) = view.endpoints.first_mut() {
            endpoint.live = view.live.clone();
        }
        if let Some(surface) = &mut view.live.surface {
            // This authored scene needs no terminal caret between captures.
            Arc::make_mut(surface).frame.cursor = None;
        }
        view.dismiss_menu(window, cx);
        window.resize(size(px(640.), px(480.)));
        cx.notify();
    })?;
    cx.background_executor()
        .timer(motion::duration(motion::LEAVE))
        .await;
    for (name, page, command) in [
        ("popover", Page::Menu, None),
        ("dialog", Page::About, Some(Command::About)),
        ("palette", Page::Palette, Some(Command::Palette)),
    ] {
        let base = capture(handle, cx, true)?.pixels;
        let search = handle.update(cx, |view, window, cx| -> Result<_> {
            match command {
                Some(command) => view.command(command, window, cx),
                None => {
                    ensure!(
                        view.open_menu(window, cx),
                        "native footer menu refused to open"
                    );
                    view.menu.anchor = point(px(16.), window.viewport_size().height - px(40.));
                }
            }
            ensure!(
                view.menu.page == Some(page),
                "native {name} opened the wrong page"
            );
            if page == Page::Palette {
                let query = "About";
                if let Some(palette) = &view.menu.palette {
                    palette
                        .search
                        .update(cx, |input, cx| input.set_text_at(query, query.len(), cx));
                }
                view.filter_palette(query, cx);
            }
            cx.notify();
            Ok(view
                .menu
                .palette
                .as_ref()
                .map(|palette| palette.search.downgrade()))
        })??;
        // Entry uses the real clock, with the existing QA slow-motion scale.
        let _ = capture(handle, cx, false)?;
        cx.background_executor()
            .timer(motion::duration(motion::ENTER))
            .await;
        let active = capture(handle, cx, false)?.pixels;
        AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<()> {
            let view = root
                .downcast::<SheprdWindow>()
                .map_err(|_| anyhow!("unexpected root"))?;
            window.dispatch_keystroke(crate::keymap::parse_keystroke("escape")?, cx);
            ensure!(
                view.read(cx).menu.page.is_none()
                    && view.read(cx).menu.close.is_none()
                    && view.read(cx).menu.palette.is_none(),
                "native {name} kept an active popup owner after dismissal"
            );
            Ok(())
        })??;
        let mut immediate = None;
        let middle = if reduced {
            immediate = Some(capture(handle, cx, true)?.pixels);
            handle.update(cx, |view, _, _| -> Result<()> {
                ensure!(
                    view.native_departure_released(),
                    "reduced-motion {name} kept outgoing paint after its first draw"
                );
                Ok(())
            })??;
            None
        } else {
            let deadline = Instant::now() + motion::duration(motion::LEAVE);
            loop {
                let frame = capture(handle, cx, false)?;
                if let Some((drawn_page, shown)) = frame.departure
                    && drawn_page == page
                    && (0.25..0.8).contains(&shown)
                {
                    eprintln!("DEPARTURE_FRAME {name} remaining={shown}");
                    break Some(frame.pixels);
                }
                ensure!(
                    Instant::now() < deadline,
                    "native {name} had no intermediate outgoing frame"
                );
                cx.background_executor()
                    .timer(Duration::from_millis(25))
                    .await;
            }
        };
        cx.background_executor()
            .timer(motion::duration(motion::LEAVE))
            .await;
        let settled = capture(handle, cx, true)?.pixels;
        handle.update(cx, |view, _, cx| -> Result<()> {
            ensure!(
                view.native_departure_released(),
                "native {name} kept outgoing paint after settling"
            );
            if let Some(search) = &search {
                ensure!(
                    search.upgrade().is_none(),
                    "native palette retained its search input"
                );
            }
            cx.notify();
            Ok(())
        })??;
        let (comparison, base, active, middle, settled, immediate) = cx
            .background_spawn(async move {
                let first_draw = immediate
                    .as_ref()
                    .map_or(Ok(()), |immediate| compare(&base, &active, None, immediate));
                (
                    first_draw.and_then(|()| compare(&base, &active, middle.as_ref(), &settled)),
                    base,
                    active,
                    middle,
                    settled,
                    immediate,
                )
            })
            .await;
        receipt(name, "active", active, cx).await?;
        if let Some(middle) = middle {
            receipt(name, "outgoing", middle, cx).await?;
        }
        receipt(name, "settled", settled, cx).await?;
        if comparison.is_err() {
            receipt(name, "baseline", base, cx).await?;
            if let Some(immediate) = immediate {
                receipt(name, "immediate", immediate, cx).await?;
            }
        }
        comparison?;
    }
    eprintln!(
        "DEPARTURES native PASS: mode={} owner reset, native frames, settled baseline and release",
        if reduced { "reduced" } else { "animated" }
    );
    Ok(())
}
