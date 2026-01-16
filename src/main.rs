mod back_logic;
mod ui;
use std::collections::HashMap;
use std::io::Stdout;
use std::sync::OnceLock;

use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use ratatui::{DefaultTerminal, Frame, layout::Rect};
use ui::nask_center::NaskCenter;

use crate::back_logic::message_loop::Command;
use crate::ui::app_ui_state::{
    AdditionalContextState, AppUIState, CheckBoxEntry, InputMode, MetaInfoState, NaskInputBoxState,
};
use crate::ui::meta_info::create_meta_info;
use crate::ui::nask_center_input::clamp_input_scroll;
use crate::ui::nvim_buffers::create_nvim_buffers;
use tui_input::backend::crossterm::EventHandler;

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
#[derive(PartialEq, Eq)]
enum EventSignal {
    Continue,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum KeyOperationEvent {
    ToggleBuffers,
    SelectRightBuffer,
    SelectLeftBuffer,
    CheckSelectedBuffer,
    InputSubmitted,
    InputChangeToInsertMode,
    InputChangeToNormalMode,
    Quit,
    ForwardToInput,
    Noop,
}

fn get_key_operation_event(
    key: KeyEvent,
    app_state: &AppUIState,
    guard_map: &HashMap<KeyOperationEvent, fn(&AppUIState) -> bool>,
) -> KeyOperationEvent {
    let ev = match (key.code, key.modifiers, app_state.input_box_state.mode) {
        (KeyCode::Char('i'), _, InputMode::Normal) => KeyOperationEvent::InputChangeToInsertMode,
        (KeyCode::Char('q'), _, InputMode::Normal) => KeyOperationEvent::Quit,
        (KeyCode::Esc, _, InputMode::Insert) => KeyOperationEvent::InputChangeToNormalMode,

        (KeyCode::Enter, _, InputMode::Insert) => KeyOperationEvent::InputSubmitted,

        (KeyCode::Char('x'), mods, _) if mods.contains(KeyModifiers::ALT) => {
            KeyOperationEvent::ToggleBuffers
        }
        (KeyCode::Up, mods, _) if mods.contains(KeyModifiers::ALT) => {
            KeyOperationEvent::CheckSelectedBuffer
        }
        (KeyCode::Left, mods, _) if mods.contains(KeyModifiers::ALT) => {
            KeyOperationEvent::SelectLeftBuffer
        }
        (KeyCode::Right, mods, _) if mods.contains(KeyModifiers::ALT) => {
            KeyOperationEvent::SelectRightBuffer
        }

        _ => KeyOperationEvent::ForwardToInput,
        // will be checked by the guard below and either
        // get nooped or kept
    };

    match guard_map.get(&ev) {
        None => ev,
        Some(guard) if guard(app_state) => ev,
        _ => KeyOperationEvent::Noop,
    }
}

struct DedicatedEventProcessor;

impl DedicatedEventProcessor {
    fn get_key_operation_guard_map() -> &'static HashMap<KeyOperationEvent, fn(&AppUIState) -> bool>
    {
        static GUARD_MAP: OnceLock<HashMap<KeyOperationEvent, fn(&AppUIState) -> bool>> =
            OnceLock::new();

        GUARD_MAP.get_or_init(|| {
            fn not_collapsed_guard(state: &AppUIState) -> bool {
                !state.additional_context_state.collapsed
            }

            let mut map: HashMap<KeyOperationEvent, fn(&AppUIState) -> bool> = HashMap::new();

            map.insert(KeyOperationEvent::SelectLeftBuffer, not_collapsed_guard);

            map.insert(KeyOperationEvent::SelectRightBuffer, not_collapsed_guard);

            map.insert(KeyOperationEvent::CheckSelectedBuffer, not_collapsed_guard);

            map.insert(KeyOperationEvent::ForwardToInput, |state| {
                state.input_box_state.mode == InputMode::Insert
            });

            map
        })
    }

    fn process_key_event(&self, key: KeyEvent, state: &mut AppUIState) -> EventSignal {
        let key_operation_event = get_key_operation_event(
            key,
            state,
            DedicatedEventProcessor::get_key_operation_guard_map(),
        );
        match key_operation_event {
            KeyOperationEvent::ToggleBuffers => {
                state.additional_context_state.collapsed =
                    !state.additional_context_state.collapsed;
            }
            KeyOperationEvent::CheckSelectedBuffer => {
                if let Some(item) = state
                    .additional_context_state
                    .entries
                    .iter_mut()
                    .find(|v| v.selected)
                {
                    item.checked = !item.checked;
                }
            }
            KeyOperationEvent::SelectLeftBuffer => {
                if let Some((idx, item)) = state
                    .additional_context_state
                    .entries
                    .iter_mut()
                    .enumerate()
                    .find(|(_idx, v)| v.selected)
                {
                    item.selected = false;
                    let mut new_select_idx = idx;
                    if new_select_idx == 0 {
                        new_select_idx = state.additional_context_state.entries.len() - 1;
                    } else {
                        new_select_idx -= 1;
                    }
                    let new_select_item = state
                        .additional_context_state
                        .entries
                        .get_mut(new_select_idx);
                    if let Some(new_item) = new_select_item {
                        new_item.selected = true;
                    }
                }
            }
            KeyOperationEvent::SelectRightBuffer => {
                if let Some((idx, item)) = state
                    .additional_context_state
                    .entries
                    .iter_mut()
                    .enumerate()
                    .find(|(_idx, v)| v.selected)
                {
                    item.selected = false;
                    let mut new_select_idx = idx;
                    if new_select_idx == state.additional_context_state.entries.len() - 1 {
                        new_select_idx = 0;
                    } else {
                        new_select_idx += 1;
                    }
                    let new_select_item = state
                        .additional_context_state
                        .entries
                        .get_mut(new_select_idx);
                    if let Some(new_item) = new_select_item {
                        new_item.selected = true;
                    }
                }
            }
            KeyOperationEvent::InputChangeToInsertMode => {
                state.input_box_state.mode = InputMode::Insert
            }
            KeyOperationEvent::InputChangeToNormalMode => {
                state.input_box_state.mode = InputMode::Normal
            }
            KeyOperationEvent::Quit => {
                return EventSignal::Quit;
            }
            KeyOperationEvent::InputSubmitted => {
                pump_message_loop(Command::ChatMessage(String::from(
                    state.input_box_state.input.value(),
                )));
                state.input_box_state.input.reset();
                clamp_input_scroll(&mut state.input_box_state);
            }
            KeyOperationEvent::ForwardToInput => {
                EventHandler::handle_event(&mut state.input_box_state.input, &Event::Key(key));
                clamp_input_scroll(&mut state.input_box_state);
            }

            KeyOperationEvent::Noop => {}
        }
        EventSignal::Continue
    }
}

trait EventProcessor {
    fn process(self, state: &mut AppUIState, processor: &DedicatedEventProcessor) -> EventSignal;
}

impl EventProcessor for Event {
    fn process(self, state: &mut AppUIState, processor: &DedicatedEventProcessor) -> EventSignal {
        match self {
            Event::Key(key_event) => processor.process_key_event(key_event, state),
            _ => EventSignal::Continue, // Handle other cases if necessary
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
