mod back_logic;
mod ui;

use std::io::Stdout;

use color_eyre::{Result, eyre::Ok};

use crossterm::event;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use ratatui::{DefaultTerminal, Frame, layout::Rect};
use ui::nask_center::NaskCenter;

use crate::back_logic::message_loop::Command;
use crate::ui::app_ui_state::{
    AdditionalContextState, AppUIState, CheckBoxEntry, MetaInfoState, NaskInputBoxState,
};
use crate::ui::event_system::{DedicatedEventProcessor, EventProcessor, EventSignal};
use crate::ui::meta_info::create_meta_info;
use crate::ui::nvim_buffers::create_nvim_buffers;

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

fn pump_message_loop(cmd: Command) {}

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
    let event_processor = DedicatedEventProcessor;
    let mut state = AppUIState::new(pump_message_loop);
    get_additional_contexts(&mut state.additional_context_state);
    get_meta_info(&mut state.meta_info_state);
    loop {
        terminal.draw(|f| render(f, &mut state))?;
        update_cursor_visibility(&mut terminal, &mut state.input_box_state);

        if EventSignal::Quit == (event::read()?).process(&mut state, &event_processor) {
            break Ok(());
        }
    }
}
