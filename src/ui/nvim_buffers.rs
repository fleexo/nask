use crate::ui::app_ui_state::AppUIState;
use crate::ui::common::ACCENT_COLOR;
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

pub struct NvimBuffers;

pub fn create_nvim_buffers() -> &'static dyn Renderable {
    static NVIM: NvimBuffers = NvimBuffers;
    &NVIM
}

const NVIM_BUFFERS_HEIGHT: u16 = 1;

impl Renderable for NvimBuffers {
    fn area_rect(&self, area: Rect) -> Rect {
        Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(NVIM_BUFFERS_HEIGHT),
            width: area.width,
            height: NVIM_BUFFERS_HEIGHT,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, _state: &mut AppUIState) {
        let count = 3;

        let line = Line::from(vec![
            // left arrow
            Span::styled(
                "▴",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
            // opening bracket
            Span::styled(
                "[",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
            // number in accent color
            Span::styled(
                count.to_string(),
                Style::default()
                    .fg(ACCENT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            // closing bracket
            Span::styled(
                "]",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
            // right arrow
            Span::styled(
                "▴",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
        ]);

        Paragraph::new(line)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}
