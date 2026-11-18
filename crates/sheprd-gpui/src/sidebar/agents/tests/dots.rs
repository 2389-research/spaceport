// ABOUTME: Checks default status marks remain distinct without their colors.
// ABOUTME: Exercises both marker geometry and the actual painted status elements.
use super::*;
use crate::config::{FontConfig, Theme};
use crate::sidebar::agents::dot_radius;
use gpui::{IntoElement, ParentElement, Render};

#[test]
fn done_and_blocked_have_distinct_default_marker_geometry() {
    for diameter in [5., 8., 16.] {
        assert_ne!(
            dot_radius(AgentStatus::Done, diameter),
            dot_radius(AgentStatus::Blocked, diameter)
        );
    }
}

struct Mark {
    status: AgentStatus,
    size: f32,
}

impl Render for Mark {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        let font = FontConfig {
            size: self.size,
            ..crate::config::Config::default().sidebar
        };
        gpui::div().child(crate::sidebar::agents::status_indicator(
            self.status,
            &font,
            Indicators::new(None, StatusIcons::Dots, false, false, &Theme::default()),
        ))
    }
}

#[gpui::test]
fn rendered_done_and_blocked_marks_have_distinct_shapes(cx: &mut gpui::TestAppContext) {
    for size in [12., 24.] {
        for status in [AgentStatus::Done, AgentStatus::Blocked] {
            let (_, cx) = cx.add_window_view(|_, _| Mark { status, size });
            cx.update(|window, cx| {
                crate::sidebar::layout_tests::full_draw(window, cx).clear(cx);
                let quads: Vec<_> = window
                    .painted_quads()
                    .into_iter()
                    .filter(|quad| quad.border_widths.top.0 > 0.)
                    .collect();
                assert!(!quads.is_empty());
                let square = quads.iter().all(|quad| quad.corner_radii.top_left.0 == 0.);
                assert_eq!(square, status == AgentStatus::Blocked);
            });
        }
    }
}
