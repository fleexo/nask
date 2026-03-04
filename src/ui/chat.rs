use crate::ui::app_ui_state::{AppUIState, ChatMessage};
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
    let items: Vec<(bool, String)> = state
        .chat_state
        .chat_messages
        .iter()
        .map(|m| (m.is_response, m.message.clone()))
        .collect();

    for (i, (is_response, msg)) in items.into_iter().enumerate() {
        let chat_message_box = ChatMessageBox::new(is_response, msg);

        let message_area = Rect {
            x: area.x,
            y: area.y + (i as u16 * chat_message_box.height),
            width: area.width,
            height: chat_message_box.height,
        };

        let rect = chat_message_box.area_rect(message_area);
        chat_message_box.render(rect, buf, state);
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
