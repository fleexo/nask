use crate::ui::app_ui_state::AppUIState;
use crate::ui::chat_message_box::ChatMessageBox;

use crate::ui::nask_center::INPUT_HEIGHT;
use crate::ui::nask_center_input::create_input_box;
use crate::ui::renderable_trait::Renderable;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

pub struct NaskChat {
    pub chat_dialog: Box<dyn Renderable>,
    pub input_box: Box<dyn Renderable>,
}

struct ChatDialog {
    top_padding: u16, // meta info
    bot_padding: u16, // (input_box + menu with context's)
}

impl ChatDialog {
    pub fn new(top_pad: u16, bot_pad: u16) -> Self {
        Self {
            top_padding: top_pad,
            bot_padding: bot_pad,
        }
    }

    pub fn _scroll_up(state: &mut AppUIState, lines: usize) {
        state.chat_state.follow_tail = false;
        state.chat_state.scroll_start_idx = state.chat_state.scroll_start_idx.saturating_sub(lines);
    }

    pub fn _scroll_down(state: &mut AppUIState, lines: usize) {
        state.chat_state.follow_tail = false;
        state.chat_state.scroll_start_idx = state.chat_state.scroll_start_idx.saturating_add(lines);
    }

    pub fn _scroll_to_bottom(state: &mut AppUIState) {
        state.chat_state.follow_tail = true;
    }
}

const CHAT_DIALOG_PAD: u16 = 1;

impl Renderable for ChatDialog {
    fn area_rect(&self, area: Rect) -> Rect {
        let pad = CHAT_DIALOG_PAD;

        let x = area.x.saturating_add(pad);
        let y = area.y.saturating_add(self.top_padding).saturating_add(pad);

        let width = area.width.saturating_sub(pad * 2);
        let height = area
            .height
            .saturating_sub(self.top_padding)
            .saturating_sub(self.bot_padding)
            .saturating_sub(pad * 2);

        Rect {
            x,
            y,
            width,
            height,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppUIState) {
        let count = state.chat_state.chat_messages.len();
        if count == 0 {
            return;
        }

        let half = area.width.saturating_div(2);

        // Precompute heights (dynamic + wrapped)
        let mut heights: Vec<u16> = Vec::with_capacity(count);
        for msg in &state.chat_state.chat_messages {
            heights.push(ChatMessageBox::calc_height(msg, half));
        }

        // Compute "bottom anchored" start index (last messages that fit)
        let bottom_start_idx = {
            let mut used: u16 = 0;
            let mut idx = count; // exclusive
            while idx > 0 {
                let h = heights[idx - 1];
                if used.saturating_add(h) > area.height {
                    break;
                }
                used = used.saturating_add(h);
                idx -= 1;
            }
            idx
        };

        // If we follow the tail, always jump to bottom
        if state.chat_state.follow_tail {
            state.chat_state.scroll_start_idx = bottom_start_idx;
        }

        // Clamp scroll start
        if state.chat_state.scroll_start_idx > count {
            state.chat_state.scroll_start_idx = bottom_start_idx;
        }

        // Render visible messages starting at scroll_start_idx until we fill the viewport
        let mut y = area.y;
        for i in state.chat_state.scroll_start_idx..count {
            let h = heights[i];
            if y >= area.y.saturating_add(area.height) {
                break;
            }

            // Stop if the message would start below the viewport
            if y.saturating_add(h) <= area.y {
                y = y.saturating_add(h);
                continue;
            }

            let box_ = ChatMessageBox::new(i);

            let message_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: h,
            };

            let rect = box_.area_rect(message_area);
            box_.render(rect, buf, state);

            y = y.saturating_add(h);
            if y > area.y.saturating_add(area.height) {
                break;
            }
        }
    }
}

pub fn create_chat_dialog(top_padding: u16, bot_padding: u16) -> Box<dyn Renderable> {
    Box::new(ChatDialog::new(top_padding, bot_padding))
}

impl NaskChat {
    pub fn new(meta_h: u16, contexts_h: u16) -> Self {
        let input_h = INPUT_HEIGHT;
        let top = meta_h;
        let bottom = input_h + contexts_h;
        Self {
            chat_dialog: create_chat_dialog(top, bottom),
            input_box: create_input_box(input_h, contexts_h),
        }
    }
}
