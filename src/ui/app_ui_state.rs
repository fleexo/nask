use crate::back_logic::message_loop::Command;
use tui_input::Input;

use std::{sync::mpsc, time::SystemTime};

#[derive(Clone)]
pub struct UiSink {
    pub tx: mpsc::Sender<UiEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

pub enum UiEvent {
    ChatMessage {
        role: ChatRole,
        text: String,
        more_follows: bool,
    },
}

impl UiSink {
    pub fn chat_answer(&self, text: String, more_follows: bool) {
        let _ = self.tx.send(UiEvent::ChatMessage {
            role: ChatRole::Assistant,
            text,
            more_follows,
        });
    }

    pub fn chat_user(&self, text: String) {
        let _ = self.tx.send(UiEvent::ChatMessage {
            role: ChatRole::User,
            text,
            more_follows: false,
        });
    }

    pub fn emit(&self, ev: UiEvent) {
        let _ = self.tx.send(ev);
    }
}

pub struct CheckBoxEntry {
    pub checked: bool,
    pub selected: bool,
    pub entry: String,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Focus {
    Input,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

pub struct AdditionalContextState {
    pub entries: Vec<CheckBoxEntry>,
    pub collapsed: bool,
}

pub struct NaskInputBoxState {
    pub input: Input,
    pub focus: Focus,
    pub mode: InputMode,
    pub input_scroll: u16,
    pub last_input_inner_width: u16,
    pub cursor_pos: Option<(u16, u16)>,
    pub last_cursor_pos: Option<(u16, u16)>,
}

pub struct MetaInfoState {
    pub model_name: String,
    pub endpoint: String,
}

pub struct ChatMessage {
    pub timestamp: SystemTime,
    pub is_response: bool,
    pub message: String,
    pub is_complete: bool,
}

impl ChatMessage {
    pub fn new(response: bool, message: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            is_response: response,
            message,
            is_complete: false,
        }
    }
}

pub struct ChatState {
    pub chat_messages: Vec<ChatMessage>,
}

pub struct AppUIState {
    pub input_box_state: NaskInputBoxState,
    pub meta_info_state: MetaInfoState,
    pub additional_context_state: AdditionalContextState,
    pub chat_state: ChatState,

    pub pump_message_loop: Box<dyn FnMut(Command)>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            chat_messages: Vec::new(),
        }
    }
}

impl Default for MetaInfoState {
    fn default() -> Self {
        Self {
            model_name: String::new(),
            endpoint: String::new(),
        }
    }
}

impl Default for AdditionalContextState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            collapsed: true,
        }
    }
}

impl Default for NaskInputBoxState {
    fn default() -> Self {
        Self {
            input: Input::default(),
            focus: Focus::Input,
            mode: InputMode::Insert,
            input_scroll: 0,
            last_input_inner_width: 0,
            cursor_pos: None,
            last_cursor_pos: None,
        }
    }
}

impl AppUIState {
    pub fn new(pump: impl FnMut(Command) + 'static) -> Self {
        Self {
            input_box_state: NaskInputBoxState::default(),
            meta_info_state: MetaInfoState::default(),
            additional_context_state: AdditionalContextState::default(),
            chat_state: ChatState::default(),
            pump_message_loop: Box::new(pump),
        }
    }

    pub fn apply_ui_event(&mut self, ev: UiEvent) {
        match ev {
            UiEvent::ChatMessage {
                role,
                text,
                more_follows,
            } => self.apply_chat_message(role, text, more_follows),
        }
    }

    fn apply_chat_message(&mut self, role: ChatRole, text: String, more_follows: bool) {
        if text.is_empty() && more_follows {
            return;
        }

        let is_response = matches!(role, ChatRole::Assistant);

        let should_append = match self.chat_state.chat_messages.last() {
            Some(m) => m.is_response == is_response && !m.is_complete,
            None => false,
        };

        if should_append {
            let last = self.chat_state.chat_messages.last_mut().unwrap();
            last.message.push_str(&text);
            last.is_complete = !more_follows;
        } else {
            let mut msg = ChatMessage::new(is_response, text);
            msg.is_complete = !more_follows; 
            self.chat_state.chat_messages.push(msg);
        }
    }

    pub fn dispatch(&mut self, cmd: Command) {
        (self.pump_message_loop)(cmd);
    }
}