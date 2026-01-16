use crate::back_logic::message_loop::Command;
use tui_input::Input;

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

pub struct AppUIState {
    pub input_box_state: NaskInputBoxState,
    pub meta_info_state: MetaInfoState,
    pub additional_context_state: AdditionalContextState,
    pub pump_message_loop: Box<dyn FnMut(Command)>,
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
            pump_message_loop: Box::new(pump),
        }
    }
}
