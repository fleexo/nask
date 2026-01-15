use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

const ACCENT_COLOR: Color = Color::Rgb(100, 160, 220);
const SEMI_ACCENT_COLOR: Color = Color::Rgb(90, 120, 150);
const ASCII_ART_NASK_BANNER: &str = include_str!("../assets/nask.txt");

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Focus {
    Input,
    Other,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct AppState {
    pub input: Input,
    pub focus: Focus,
    pub mode: Mode,
    pub input_scroll: u16,
    pub last_input_inner_width: u16,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            input: Input::default(),
            focus: Focus::Input,
            mode: Mode::Insert,
            input_scroll: 0,
            last_input_inner_width: 0,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct AppLayout {
    pub meta_hints: Rect,
    pub title: Rect,
    pub message_input_box: Rect,
    pub buffers: Rect,
}

pub fn meta_rect(area: Rect, w: u16, h: u16) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(w.min(area.width)),
        y: area.y + 1,
        width: w.min(area.width),
        height: h.min(area.height),
    }
}

pub fn nask_center_rect(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);

    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) * 2 / 5,
        width: w,
        height: h,
    }
}

pub fn nvim_buffers_rect(area: Rect, w: u16, h: u16) -> Rect {
    Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(h),
        width: w.min(area.width),
        height: h.min(area.height),
    }
}

/* ---------- Input Box ---------- */

pub struct NaskInputBox<'a> {
    pub input: &'a Input,
    pub focused: bool,
    pub title: &'a str,
    pub placeholder: &'a str,
    pub scroll: u16,
    pub insert_mode: Mode,
}

fn input_block<'a>(title: &'a str, focused: bool, insert_mode: Mode) -> Block<'a> {
    let border_style = if focused && insert_mode == Mode::Insert {
        Style::default()
            .fg(ACCENT_COLOR)
            .add_modifier(Modifier::BOLD)
    } else if focused && insert_mode == Mode::Normal {
        Style::default().fg(SEMI_ACCENT_COLOR)
    } else {
        Style::default()
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

impl<'a> NaskInputBox<'a> {
    pub fn inner_rect(&self, area: Rect) -> Rect {
        input_block(self.title, self.focused, self.insert_mode).inner(area)
    }

    pub fn cursor_pos(&self, area: Rect) -> (u16, u16) {
        let inner = self.inner_rect(area);
        let cursor = self.input.cursor() as u16;
        (inner.x + cursor.saturating_sub(self.scroll), inner.y)
    }
}

impl<'a> Widget for NaskInputBox<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = input_block(self.title, self.focused, self.insert_mode);
        let inner = block.inner(area);
        let inner_w = inner.width as usize;

        let value = self.input.value();
        let visible = if value.is_empty() {
            Line::from(Span::styled(
                self.placeholder.to_string(),
                Style::default().add_modifier(Modifier::DIM),
            ))
        } else if inner_w == 0 {
            Line::from("")
        } else {
            let start = self.scroll as usize;
            let end = (start + inner_w).min(value.len());
            Line::from(value[start..end].to_string())
        };

        Paragraph::new(visible).block(block).render(area, buf);
    }
}

/* ---------- Layout ---------- */

fn banner_height() -> u16 {
    ASCII_ART_NASK_BANNER.lines().count() as u16
}

pub fn compute(area: Rect) -> AppLayout {
    let input_h = 5;
    let title_h = banner_height();
    let center_h = title_h + input_h;
    let center = nask_center_rect(area, 70, center_h);
    AppLayout {
        meta_hints: meta_rect(area, 10, 10),
        title: Rect {
            x: center.x,
            y: center.y,
            width: center.width,
            height: title_h,
        },
        message_input_box: Rect {
            x: center.x,
            y: center.y + title_h,
            width: center.width,
            height: input_h,
        },
        buffers: nvim_buffers_rect(area, area.width, 2),
    }
}

/* ---------- Widgets ---------- */

fn nask_title_widget() -> Paragraph<'static> {
    Paragraph::new(ASCII_ART_NASK_BANNER)
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: false })
        .style(Style::default().add_modifier(Modifier::BOLD))
}

/* ---------- Render ---------- */

fn clamp_input_scroll(state: &mut AppState) {
    let w = state.last_input_inner_width;
    let s = state.input.value();
    let len = s.len() as u16;

    // Nothing to show
    if w == 0 || len == 0 {
        state.input_scroll = 0;
        return;
    }

    let cursor = state.input.cursor() as u16; // 0..=len (cursor can be at end)

    let max_scroll = len.saturating_sub(1);
    state.input_scroll = state.input_scroll.min(max_scroll);

    let min_visible = state.input_scroll;
    let max_visible = state.input_scroll.saturating_add(w.saturating_sub(1));

    if cursor < min_visible || cursor > max_visible {
        state.input_scroll = cursor.saturating_sub(w.saturating_sub(1)).min(max_scroll);
    }
}

fn render(frame: &mut Frame, state: &mut AppState) {
    let layout = compute(frame.area());

    frame.render_widget(nask_title_widget(), layout.title);
    frame.render_widget(Paragraph::new("test"), layout.meta_hints);

    let tmp = NaskInputBox {
        input: &state.input,
        focused: true,
        title: "[Ask]",
        placeholder: "",
        scroll: state.input_scroll,
        insert_mode: state.mode,
    };
    state.last_input_inner_width = tmp.inner_rect(layout.message_input_box).width;
    clamp_input_scroll(state);

    let input_box = NaskInputBox {
        input: &state.input,
        focused: state.focus == Focus::Input,
        title: if state.mode == Mode::Insert {
            "[Ask]"
        } else {
            ""
        },
        placeholder: "Ask a question…",
        scroll: state.input_scroll,
        insert_mode: state.mode,
    };

    let cursor = if state.focus == Focus::Input && state.mode == Mode::Insert {
        Some(input_box.cursor_pos(layout.message_input_box))
    } else {
        None
    };

    frame.render_widget(input_box, layout.message_input_box);

    if let Some((x, y)) = cursor {
        frame.set_cursor_position((x, y));
    }

    frame.render_widget(Paragraph::new("test4"), layout.buffers);
}

/* ---------- Main ---------- */

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut state = AppState::default();

    loop {
        terminal.draw(|f| render(f, &mut state))?;

        if let Event::Key(key) = event::read()? {
            match state.mode {
                Mode::Insert => {
                    if key.code == KeyCode::Esc {
                        state.mode = Mode::Normal;
                    } else {
                        EventHandler::handle_event(&mut state.input, &Event::Key(key));
                        clamp_input_scroll(&mut state); // stick to end
                    }
                }
                Mode::Normal => {
                    if key.code == KeyCode::Char('i') {
                        state.mode = Mode::Insert;
                    } else if key.code == KeyCode::Char('q') {
                        break Ok(());
                    }
                }
            }
        }
    }
}
