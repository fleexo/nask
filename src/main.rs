mod ui;
use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{DefaultTerminal, Frame, layout::Rect};
use ui::nask_center::NaskCenter;

use crate::ui::app_ui_state::{AppUIState, InputMode};
use crate::ui::nask_center_input::clamp_input_scroll;
use tui_input::backend::crossterm::EventHandler;
pub fn meta_rect(area: Rect, w: u16, h: u16) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(w.min(area.width)),
        y: area.y + 1,
        width: w.min(area.width),
        height: h.min(area.height),
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

fn render(frame: &mut Frame, state: &mut AppUIState) {
    let nask_center = NaskCenter::new(frame.area());
    let renderables = nask_center.get_renderables();
    let render_buffer = frame.buffer_mut();

    // this is the center region of the app in the initial screen
    for nask_center_renderable in renderables.iter() {
        let rect = nask_center_renderable.area_rect(nask_center.center_rect);
        nask_center_renderable.render(rect, render_buffer, state);
    }

    if state.input_box_state.cursor_pos != None {
        frame.set_cursor_position(state.input_box_state.cursor_pos.unwrap());
    }
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
    let mut state = AppUIState::default();

    loop {
        terminal.draw(|f| render(f, &mut state))?;

        let desired_cursor = state.input_box_state.cursor_pos;
        if desired_cursor != state.input_box_state.last_cursor_pos {
            state.input_box_state.last_cursor_pos = desired_cursor;

            match desired_cursor {
                Some(_) => {
                    terminal.show_cursor()?;
                }
                None => {
                    terminal.hide_cursor()?;
                }
            }
        } else {
            // No change: do nothing (avoid syscalls)
        }
        let mut input_state = &mut state.input_box_state;
        if let Event::Key(key) = event::read()? {
            match input_state.mode {
                InputMode::Insert => {
                    if key.code == KeyCode::Esc {
                        input_state.mode = InputMode::Normal;
                    } else {
                        EventHandler::handle_event(&mut input_state.input, &Event::Key(key));
                        clamp_input_scroll(&mut input_state); //stick to end
                    }
                }
                InputMode::Normal => {
                    if key.code == KeyCode::Char('i') {
                        input_state.mode = InputMode::Insert;
                    } else if key.code == KeyCode::Char('q') {
                        break Ok(());
                    }
                }
            }
        }
    }
}
