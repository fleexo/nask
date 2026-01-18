use std::collections::HashMap;

use crate::{back_logic::message_loop::Command, ui::app_ui_state::UiSink};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKind {
    ChatMessage,
    Quit,
}

impl Command {
    pub fn kind(&self) -> CommandKind {
        match self {
            Command::ChatMessage(_) => CommandKind::ChatMessage,
        }
    }
}

pub trait Dispatch {
    fn execute(&self, cmd: &Command, ui_sink: &UiSink);
}

pub struct Dispatcher {
    dispatches: HashMap<CommandKind, Box<dyn Dispatch>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        let mut dispatches: HashMap<CommandKind, Box<dyn Dispatch>> = HashMap::new();
        dispatches.insert(CommandKind::ChatMessage, Box::new(ChatMessageDispatch));
        Self { dispatches }
    }

    pub fn dispatch(&self, cmd: &Command, ui_sink: &UiSink) {
        if let Some(d) = self.dispatches.get(&cmd.kind()) {
            d.execute(cmd, ui_sink);
        }
    }
}

struct ChatMessageDispatch;
impl Dispatch for ChatMessageDispatch {
    fn execute(&self, cmd: &Command, ui_sink: &UiSink) {
        let Command::ChatMessage(msg) = cmd;
        ui_sink.chat_answer(String::from(msg), true);
    }
}
