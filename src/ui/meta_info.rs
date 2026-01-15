use crate::ui::app_ui_state::AppUIState;
use crate::ui::renderable_trait::Renderable;
use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
};

pub struct MetaInfo;

pub fn create_meta_info() -> &'static dyn Renderable {
    static META: MetaInfo = MetaInfo;
    &META
}

const META_INFO_WIDTH: u16 = 90;
const META_INFO_HEIGHT: u16 = 5;
const META_INFO_TOP_PAD: u16 = 0;
const META_INFO_RIGHT_PAD: u16 = 5;

impl Renderable for MetaInfo {
    fn area_rect(&self, area: Rect) -> Rect {
        Rect {
            x: area.x
                + area
                    .width
                    .saturating_sub(META_INFO_WIDTH)
                    .saturating_sub(META_INFO_RIGHT_PAD),
            y: area.y + META_INFO_TOP_PAD,
            width: META_INFO_WIDTH.min(area.width),
            height: META_INFO_HEIGHT.min(area.height),
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppUIState) {
        let meta_info_state = &state.meta_info_state;

        let line = Line::from(vec![
            Span::styled(
                format!("model: {}", meta_info_state.model_name),
                Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
            ),
            Span::raw("  ·  "),
            Span::styled(
                meta_info_state.endpoint.as_str(),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
        ]);

        Paragraph::new(line)
            .alignment(Alignment::Right)
            .render(area, buf);
    }
}
