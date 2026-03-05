// src/ui/chat_message_box.rs
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ui::app_ui_state::{AppUIState, ChatMessage};
use crate::ui::common::{ACCENT_COLOR, SEMI_ACCENT_COLOR};
use crate::ui::renderable_trait::Renderable;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap};

fn fmt_ts(ts: SystemTime) -> String {
    let secs = ts.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let secs_in_day = secs % 86_400;

    let hours = secs_in_day / 3600;
    let minutes = (secs_in_day % 3600) / 60;
    let seconds = secs_in_day % 60;

    format!("[{:02}:{:02}:{:02}]", hours, minutes, seconds)
}

const TOP_BOT_MARGIN: u16 = 1;

// bubble padding (applied for both answer + user bubble)
const PAD_LR: u16 = 1;
const PAD_TB: u16 = 1;

pub struct ChatMessageBox {
    chat_msg_idx: usize,
}

impl ChatMessageBox {
    pub fn new(chat_message_idx: usize) -> Self {
        Self {
            chat_msg_idx: chat_message_idx,
        }
    }

    /// Dynamic height for message when rendered into `bubble_width` (half-column width).
    pub fn calc_height(chat_msg: &ChatMessage, bubble_width: u16) -> u16 {
        let padding_h = PAD_LR * 2;
        let padding_v = PAD_TB * 2;

        // user bubble has borders
        let border_h = if chat_msg.is_response { 0 } else { 2 };
        let border_v = if chat_msg.is_response { 0 } else { 2 };

        let inner_w = bubble_width.saturating_sub(padding_h + border_h).max(1);

        let text_lines = wrapped_lines(chat_msg.message.as_str(), inner_w).max(1);

        // 1 row (timestamp/spinner) + (padding + borders + wrapped text lines)
        TOP_BOT_MARGIN + padding_v + border_v + text_lines
    }

    fn render_answer_box(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &AppUIState,
        chat_msg: &ChatMessage,
    ) {
        // top row: spinner / check
        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

        let indicator = if chat_msg.is_complete {
            "✓"
        } else if let Some(frame) = state.chat_state.spinner_frame {
            spinner_frames[frame % spinner_frames.len()]
        } else {
            // should not happen if you only animate while spinner_frame = Some(...)
            spinner_frames[0]
        };

        Paragraph::new(indicator)
            .style(Style::default().fg(Color::DarkGray))
            .render(
                Rect {
                    x: area.x,
                    y: area.y,
                    width: area.width,
                    height: 1,
                },
                buf,
            );

        // bubble area below the top row
        let bubble_area = Rect {
            x: area.x,
            y: area.y + TOP_BOT_MARGIN,
            width: area.width,
            height: area.height.saturating_sub(TOP_BOT_MARGIN),
        };

        let block = Block::default()
            .style(Style::default().bg(ACCENT_COLOR))
            .padding(Padding {
                left: PAD_LR,
                right: PAD_LR,
                top: PAD_TB,
                bottom: PAD_TB,
            });

        let inner = block.inner(bubble_area);
        block.render(bubble_area, buf);

        // dark text for answer bubble
        Paragraph::new(chat_msg.message.as_str())
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Black))
            .render(inner, buf);
    }

    fn render_user_box(&self, area: Rect, buf: &mut Buffer, chat_msg: &ChatMessage) {
        // top row: timestamp
        let ts = fmt_ts(chat_msg.timestamp);
        Paragraph::new(Line::from(ts))
            .style(Style::default().fg(Color::DarkGray))
            .render(
                Rect {
                    x: area.x,
                    y: area.y,
                    width: area.width,
                    height: 1,
                },
                buf,
            );

        // bubble area below the top row
        let bubble_area = Rect {
            x: area.x,
            y: area.y + TOP_BOT_MARGIN,
            width: area.width,
            height: area.height.saturating_sub(TOP_BOT_MARGIN),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(SEMI_ACCENT_COLOR))
            .padding(Padding {
                left: PAD_LR,
                right: PAD_LR,
                top: PAD_TB,
                bottom: PAD_TB,
            });

        let inner = block.inner(bubble_area);
        block.render(bubble_area, buf);

        Paragraph::new(chat_msg.message.as_str())
            .wrap(Wrap { trim: false })
            .render(inner, buf);
    }
}

impl Renderable for ChatMessageBox {
    fn area_rect(&self, area: Rect) -> Rect {
        // Caller supplies computed dynamic height.
        area
    }

    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppUIState) {
        let chat_msg: &ChatMessage = &state.chat_state.chat_messages[self.chat_msg_idx];

        // left/right split
        let half = area.width.saturating_div(2);
        let x = if chat_msg.is_response {
            area.x
        } else {
            area.x.saturating_add(half)
        };

        let area = Rect {
            x,
            y: area.y,
            width: half,
            height: area.height,
        };

        if chat_msg.is_response {
            self.render_answer_box(area, buf, state, chat_msg);
        } else {
            self.render_user_box(area, buf, chat_msg);
        }
    }
}

/// Word-wrap line counter (no crates).
/// - Wraps at whitespace
/// - Preserves explicit '\n'
/// - Breaks long words if needed
fn wrapped_lines(text: &str, width: u16) -> u16 {
    let w = width.max(1) as usize;
    let mut lines: u16 = 0;

    for raw_line in text.split('\n') {
        if raw_line.is_empty() {
            lines += 1;
            continue;
        }

        let mut cur = 0usize;
        let mut had_any_word = false;

        for word in raw_line.split_whitespace() {
            had_any_word = true;
            let mut wl = word.chars().count();

            if cur == 0 {
                if wl <= w {
                    cur = wl;
                } else {
                    let full = (wl + w - 1) / w;
                    lines += full as u16;
                    wl %= w;
                    cur = wl;
                }
                continue;
            }

            if cur + 1 + wl <= w {
                cur += 1 + wl;
            } else {
                lines += 1;
                cur = 0;

                if wl <= w {
                    cur = wl;
                } else {
                    let full = (wl + w - 1) / w;
                    lines += full as u16;
                    wl %= w;
                    cur = wl;
                }
            }
        }

        lines += if had_any_word { 1 } else { 1 };
    }

    lines.max(1)
}
