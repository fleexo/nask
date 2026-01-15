mod ui;
use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::Rect};
use ui::nask_center::NaskCenter;

use crate::ui::app_ui_state::{
    AdditionalContextState, AppUIState, CheckBoxEntry, InputMode, MetaInfoState,
};
use crate::ui::meta_info::create_meta_info;
use crate::ui::nask_center_input::clamp_input_scroll;
use crate::ui::nvim_buffers::create_nvim_buffers;
use tui_input::backend::crossterm::EventHandler;

pub fn nvim_buffers_rect(area: Rect, w: u16, h: u16) -> Rect {
    Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(h),
        width: w.min(area.width),
        height: h.min(area.height),
    }
}

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

fn render(frame: &mut Frame, state: &mut AppUIState) {
    let frame_area = frame.area();
    let nask_center = NaskCenter::new(frame.area());
    let renderables = nask_center.get_renderables();
    {
        let render_buffer = frame.buffer_mut();

        for r in renderables.iter() {
            let rect = r.area_rect(nask_center.center_rect);
            r.render(rect, render_buffer, state);
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

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut state = AppUIState::default();
    get_additional_contexts(&mut state.additional_context_state);
    get_meta_info(&mut state.meta_info_state);
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
        }

        let mut input_state = &mut state.input_box_state;
        let addition_context_state = &mut state.additional_context_state;
        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                (KeyCode::Char('x'), mods) if mods.contains(KeyModifiers::ALT) => {
                    addition_context_state.collapsed = !addition_context_state.collapsed;
                }
                (KeyCode::Up, mods) if mods.contains(KeyModifiers::ALT) => {
                    if addition_context_state.collapsed == false {
                        if let Some(item) = addition_context_state
                            .entries
                            .iter_mut()
                            .find(|v| v.selected)
                        {
                            item.checked = !item.checked;
                        }
                    }
                }
                (KeyCode::Left, mods) if mods.contains(KeyModifiers::ALT) => {
                    if addition_context_state.collapsed == false {
                        if let Some((idx, item)) = addition_context_state
                            .entries
                            .iter_mut()
                            .enumerate()
                            .find(|(_idx, v)| v.selected)
                        {
                            item.selected = false;
                            let mut new_select_idx = idx;
                            if new_select_idx == 0 {
                                new_select_idx = addition_context_state.entries.len() - 1;
                            } else {
                                new_select_idx -= 1;
                            }
                            let new_select_item =
                                addition_context_state.entries.get_mut(new_select_idx);
                            if let Some(new_item) = new_select_item {
                                new_item.selected = true;
                            }
                        }
                    }
                }
                (KeyCode::Right, mods) if mods.contains(KeyModifiers::ALT) => {
                    if addition_context_state.collapsed == false {
                        if let Some((idx, item)) = addition_context_state
                            .entries
                            .iter_mut()
                            .enumerate()
                            .find(|(_idx, v)| v.selected)
                        {
                            item.selected = false;
                            let mut new_select_idx = idx;
                            if new_select_idx == addition_context_state.entries.len() - 1 {
                                new_select_idx = 0;
                            } else {
                                new_select_idx += 1;
                            }
                            let new_select_item =
                                addition_context_state.entries.get_mut(new_select_idx);
                            if let Some(new_item) = new_select_item {
                                new_item.selected = true;
                            }
                        }
                    }
                }
                _ => {}
            }
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
