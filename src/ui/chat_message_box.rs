use crate::ui::app_ui_state::AppUIState;
use crate::ui::renderable_trait::Renderable;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use crate::ui::common::ACCENT_COLOR;
use crate::ui::common::SEMI_ACCENT_COLOR;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;


pub struct ChatMessageBox {
    pub is_answer: bool,
    pub message: String,
    pub height: u16,
}

impl ChatMessageBox {
    pub fn new(is_answer: bool, message: String) -> Self {
        Self {
            is_answer,
            message,
            height: 3,
        }
    }
}

impl Renderable for ChatMessageBox {
        fn area_rect(&self, area: Rect) -> Rect {
            let half = area.width.saturating_div(2);
        
            let x = if self.is_answer {
                area.x
            } else {
                area.x.saturating_add(half)
            };
        
            Rect {
                x,
                y: area.y,         
                width: half,
                height: self.height,
            }
        }

    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppUIState) {
        
let mut color = ACCENT_COLOR;
if self.is_answer == false {
    color = SEMI_ACCENT_COLOR;
}

let block = Block::default().style(Style::default().bg(color));

Paragraph::new(Text::from(self.message.clone()))
    .block(block)
    .render(area, buf);
        
    }
}
