mod back_logic;
mod ui;

use std::io::Stdout;
use std::sync::{Arc, Mutex, mpsc};

use color_eyre::{Result, eyre::Ok};

use crossterm::event;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame, layout::Rect};
use ui::nask_center::NaskCenter;

use crate::back_logic::message_loop::{Command, MessageLoop};
use crate::ui::app_ui_state::{
    AdditionalContextState, AppUIState, ChatMessage, CheckBoxEntry, MetaInfoState,
    NaskInputBoxState, UiEvent, UiSink,
};
use crate::ui::common::ACCENT_COLOR;
use crate::ui::event_system::{DedicatedEventProcessor, EventProcessor, EventSignal};
use crate::ui::meta_info::create_meta_info;
use crate::ui::nask_center::INPUT_HEIGHT;
use crate::ui::nask_center_input::create_input_box;
use crate::ui::nvim_buffers::create_nvim_buffers;
use crate::ui::renderable_trait::Renderable;

fn get_meta_info(meta_info_state: &mut MetaInfoState) {
    meta_info_state.model_name = "qwen2.5-coder:7b".to_string();
    meta_info_state.endpoint = "ollama://localhost:11434".to_string();
}

fn get_additional_contexts(context_state: &mut AdditionalContextState) {
    context_state.entries = vec![
        CheckBoxEntry {
            checked: false,
            selected: false,
            entry: String::from("test.rs"),
        },
        CheckBoxEntry {
            checked: true,
            selected: false,
            entry: String::from("test1.rs"),
        },
        CheckBoxEntry {
            checked: false,
            selected: true,
            entry: String::from("test2.rs"),
        },
    ];
}

pub struct NaskChat {
    chat_dialog: Box<dyn Renderable>,
    input_box: Box<dyn Renderable>,
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

    fn estimate_wrapped_lines_asciiish(s: &str, width: u16) -> u16 {
        if width == 0 {
            return 0;
        }
        let w = width as usize;

        let mut lines: u16 = 0;
        for raw_line in s.split('\n') {
            // Keep empty lines
            if raw_line.is_empty() {
                lines = lines.saturating_add(1);
                continue;
            }

            let mut col: usize = 0;

            for word in raw_line.split_whitespace() {
                // Count "columns" as number of chars (not bytes)
                let word_len = word.chars().count();
                let extra = if col == 0 { word_len } else { 1 + word_len };

                if col + extra <= w {
                    col += extra;
                } else {
                    // new visual line
                    lines = lines.saturating_add(1);
                    col = word_len;

                    // If a single word is longer than width, hard-break it
                    if col > w {
                        // number of lines needed for this word
                        let full = col / w;
                        let rem = col % w;
                        lines = lines.saturating_add(full as u16);
                        col = rem;
                        if col == 0 {
                            // exactly ended on boundary; next word starts fresh
                            col = 0;
                        }
                    }
                }
            }

            // final line for this raw_line
            lines = lines.saturating_add(1);
        }

        lines.max(1)
    }

    fn message_height(msg: &ChatMessage, area_width: u16) -> u16 {
        let has_border = !msg.is_response;
        let inner_w = area_width
            .saturating_sub(if has_border { 2 } else { 0 })
            .max(1);

        let mut inner_lines = ChatDialog::estimate_wrapped_lines_asciiish(&msg.message, inner_w);

        if !msg.is_complete {
            inner_lines = inner_lines.saturating_add(1); // for "…"
        }

        inner_lines.saturating_add(if has_border { 2 } else { 0 })
    }
}
use ratatui::buffer::Buffer;
use ratatui::prelude::Widget;
const CHAT_DIALOG_PAD: u16 = 1;
impl Renderable for ChatDialog {
    fn area_rect(&self, area: Rect) -> Rect {
        let pad = CHAT_DIALOG_PAD;

        let x = area.x.saturating_add(pad);
        let y = area.y.saturating_add(self.top_padding).saturating_add(pad);

        let width = area.width.saturating_sub(pad * 2); // left+right padding
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
        let chat_state = &state.chat_state;

        // render newest at bottom
        let mut y_bottom = area.y + area.height;

        for msg in chat_state.chat_messages.iter().rev() {
            if y_bottom <= area.y {
                break;
            }

            let h = ChatDialog::message_height(msg, area.width);
            if h == 0 {
                continue;
            }

            let top = y_bottom.saturating_sub(h).max(area.y);
            let used_h = y_bottom.saturating_sub(top);

            let msg_area = Rect {
                x: area.x,
                y: top,
                width: area.width,
                height: used_h,
            };

            let (block, base_style) = if msg.is_response {
                (None, Style::default().add_modifier(Modifier::BOLD))
            } else {
                let b = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(ACCENT_COLOR));
                (Some(b), Style::default())
            };

            let mut text = Text::from(Line::from(Span::raw(msg.message.as_str())));
            if !msg.is_complete {
                text.lines.push(Line::from(Span::styled(
                    "…",
                    Style::default().add_modifier(Modifier::DIM),
                )));
            }

            let mut p = Paragraph::new(text)
                .style(base_style)
                .wrap(Wrap { trim: false });

            if let Some(b) = block {
                p = p.block(b);
            }

            p.render(msg_area, buf);

            y_bottom = top;
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
            input_box: create_input_box(input_h),
        }
    }
}

fn render(frame: &mut Frame, state: &mut AppUIState) {
    let frame_area = frame.area();
    if state.chat_state.chat_messages.is_empty() {
        let nask_center = NaskCenter::new(frame.area());
        let renderables = nask_center.get_renderables();
        {
            let render_buffer = frame.buffer_mut();

            for r in renderables.iter() {
                let rect = r.area_rect(nask_center.center_rect);
                r.render(rect, render_buffer, state);
            }
        }
    } else {
        // render the chat
        let meta_info_height = 1; // TODO!
        let contexts_menu_height = 1; // TODO!
        let nask_chat = NaskChat::new(meta_info_height, contexts_menu_height);
        {
            let render_buffer = frame.buffer_mut();
            {
                let rect = nask_chat.chat_dialog.area_rect(frame_area);
                nask_chat.chat_dialog.render(rect, render_buffer, state);
            }
            {
                let rect = nask_chat.input_box.area_rect(frame_area);
                nask_chat.input_box.render(rect, render_buffer, state);
            }
        }
    }
    if state.input_box_state.cursor_pos.is_some() {
        frame.set_cursor_position(state.input_box_state.cursor_pos.unwrap());
    }

    {
        let render_buffer = frame.buffer_mut();
        let static_renderables = [create_meta_info(), create_nvim_buffers()];
        for r in static_renderables.iter() {
            let rect = r.area_rect(frame_area);
            r.render(rect, render_buffer, state);
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn update_cursor_visibility(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    input_box_state: &mut NaskInputBoxState,
) {
    let desired_cursor = input_box_state.cursor_pos;

    if desired_cursor != input_box_state.last_cursor_pos {
        input_box_state.last_cursor_pos = desired_cursor;

        if desired_cursor.is_some() {
            if let Err(e) = terminal.show_cursor() {
                eprintln!("Error showing cursor: {}", e);
            }
        } else if let Err(e) = terminal.hide_cursor() {
            eprintln!("Error hiding cursor: {}", e);
        }
    }
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let message_loop = Arc::new(Mutex::new(MessageLoop::default()));
    let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
    let ui_sink = UiSink { tx: ui_tx };
    {
        message_loop.lock().unwrap().run(ui_sink.clone());
    }

    let event_processor = DedicatedEventProcessor;
    let ml = Arc::clone(&message_loop);
    let mut state = AppUIState::new(move |cmd: Command| ml.lock().unwrap().pump_message_loop(cmd));

    get_additional_contexts(&mut state.additional_context_state);
    get_meta_info(&mut state.meta_info_state);

    let result = loop {
        while let std::result::Result::Ok(ev) = ui_rx.try_recv() {
            state.apply_ui_event(ev);
        }

        terminal.draw(|f| render(f, &mut state))?;
        update_cursor_visibility(&mut terminal, &mut state.input_box_state);

        if EventSignal::Quit == (event::read()?).process(&mut state, &event_processor) {
            break Ok(());
        }
    };
    {
        message_loop.lock().unwrap().stop();
    }
    result
}
