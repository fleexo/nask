use crate::ui::app_ui_state::AdditionalContextState;
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

impl NvimBuffers {
    fn render_collapsed(area: Rect, count: usize, buf: &mut Buffer) {
        let line = Line::from(vec![
            Span::styled(
                "▴",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                "[",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                count.to_string(),
                Style::default()
                    .fg(ACCENT_COLOR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "]",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
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

    fn render_expanded(
        area: Rect,
        additional_context_state: &AdditionalContextState,
        buf: &mut Buffer,
    ) {
        if additional_context_state.entries.is_empty() {
            Paragraph::new("No entries...")
                .alignment(Alignment::Center)
                .style(Style::default().bg(Color::Rgb(22, 22, 22)))
                .render(area, buf);
            return;
        }

        let mut spans: Vec<Span> = Vec::new();

        for entry in additional_context_state.entries.iter() {
            let is_checked = entry.checked;
            let is_selected = entry.selected;

            let text_style = if is_selected {
                Style::default().fg(ACCENT_COLOR).bold()
            } else {
                Style::default()
            };

            spans.push(Span::raw("["));

            if is_checked {
                spans.push(Span::styled("x", Style::default().fg(ACCENT_COLOR).bold()));
            } else {
                spans.push(Span::raw(" "));
            }

            spans.push(Span::raw("] "));
            spans.push(Span::styled(entry.entry.as_str(), text_style));
            spans.push(Span::raw("  "));
        }

        let line = Line::from(spans);

        Paragraph::new(line)
            .style(Style::default().bg(Color::Rgb(22, 22, 22)))
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}

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

    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppUIState) {
        let buffer_state = &state.additional_context_state;

        let count = buffer_state.entries.len();
        if buffer_state.collapsed {
            NvimBuffers::render_collapsed(area, count, buf);
        } else {
            NvimBuffers::render_expanded(area, buffer_state, buf);
        }
    }
}
