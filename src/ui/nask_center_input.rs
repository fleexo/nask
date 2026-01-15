use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Widget},
};

use crate::ui::{
    app_ui_state::{AppUIState, Focus, InputMode, NaskInputBoxState},
    common::{ACCENT_COLOR, SEMI_ACCENT_COLOR},
    nask_center_banner::banner_height,
    renderable_trait::Renderable,
};

pub struct NaskInputBox {
    line_height: u16,
}

pub fn clamp_input_scroll(state: &mut NaskInputBoxState) {
    let w = state.last_input_inner_width; // visible columns
    let len = state.input.value().len() as u16;

    if w == 0 || len == 0 {
        state.input_scroll = 0;
        return;
    }

    let cursor = state.input.cursor() as u16;

    let max_scroll = len.saturating_sub(w);
    state.input_scroll = state.input_scroll.min(max_scroll);

    let min_visible = state.input_scroll;
    let max_visible = state.input_scroll + w.saturating_sub(1);

    if cursor < min_visible {
        state.input_scroll = cursor.min(max_scroll);
    } else if cursor > max_visible {
        state.input_scroll = cursor.saturating_sub(w.saturating_sub(1)).min(max_scroll);
    }
}

impl NaskInputBox {
    fn new(line_height: u16) -> Self {
        Self { line_height }
    }

    pub fn input_block(input_state: &NaskInputBoxState) -> Block<'static> {
        let focused = input_state.focus == Focus::Input;
        let input_mode = input_state.mode;
        let border_style = if focused && input_mode == InputMode::Insert {
            Style::default()
                .fg(ACCENT_COLOR)
                .add_modifier(Modifier::BOLD)
        } else if focused && input_mode == InputMode::Normal {
            Style::default().fg(SEMI_ACCENT_COLOR)
        } else {
            Style::default()
        };

        let title = if input_mode == InputMode::Insert {
            "[Ask]"
        } else {
            ""
        };

        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style)
            .padding(Padding {
                left: 1,
                right: 1,
                top: 1,
                bottom: 0,
            })
    }

    fn cursor_pos(&self, inner: Rect, app_state: &NaskInputBoxState) -> (u16, u16) {
        let cursor = app_state.input.cursor() as u16;
        (
            inner.x + cursor.saturating_sub(app_state.input_scroll),
            inner.y,
        )
    }
}

pub fn create_input_box(line_height: u16) -> Box<dyn Renderable> {
    Box::new(NaskInputBox::new(line_height))
}

impl Renderable for NaskInputBox {
    fn area_rect(&self, area: Rect) -> Rect {
        Rect {
            x: area.x,
            y: area.y + banner_height(),
            width: area.width,
            height: self.line_height,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppUIState) {
        let input_box_state = &mut state.input_box_state;

        let block = Self::input_block(input_box_state);
        let inner = block.inner(area);

        input_box_state.last_input_inner_width = inner.width;
        clamp_input_scroll(input_box_state);

        let inner_w = inner.width as usize;
        let value = input_box_state.input.value();
        let visible = if value.is_empty() {
            Line::from(Span::styled(
                "Feel free to ask a question (:",
                Style::default().add_modifier(Modifier::DIM),
            ))
        } else if inner_w == 0 {
            Line::from("")
        } else {
            let start = input_box_state.input_scroll as usize;
            let start = start.min(value.len().saturating_sub(1)); // safety
            let end = (start + inner_w).min(value.len());
            Line::from(value[start..end].to_string())
        };

        if input_box_state.mode != InputMode::Insert {
            input_box_state.cursor_pos = None;
        } else {
            input_box_state.cursor_pos = Some(self.cursor_pos(inner, input_box_state));
        }

        Paragraph::new(visible).block(block).render(area, buf);
    }
}
