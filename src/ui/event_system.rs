use std::{collections::HashMap, sync::OnceLock};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    back_logic::message_loop::Command,
    ui::{
        app_ui_state::{AppUIState, InputMode},
        nask_center_input::clamp_input_scroll,
    },
};

#[derive(PartialEq, Eq)]
pub enum EventSignal {
    Continue,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyOperationEvent {
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

pub struct DedicatedEventProcessor;

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
                (state.pump_message_loop)(Command::ChatMessage(String::from(
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

pub trait EventProcessor {
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
