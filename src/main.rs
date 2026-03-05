mod back_logic;
mod ui;

use std::io::Stdout;
use std::sync::{Arc, Mutex, mpsc};

use color_eyre::{Result, eyre::Ok};
use crossterm::event;
use crossterm::event::Event;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use std::time::{Duration, Instant};

use ratatui::{DefaultTerminal, Frame, layout::Rect};
use ui::chat::NaskChat;
use ui::nask_center::NaskCenter;

use crate::back_logic::message_loop::{Command, MessageLoop};
use crate::ui::app_ui_state::{
    AdditionalContextState, AppUIState, CheckBoxEntry, MetaInfoState, NaskInputBoxState, UiEvent,
    UiSink,
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
    if state.chat_state.chat_messages.is_empty() {
        let nask_center = NaskCenter::new(frame.area());
        let renderables = nask_center.get_renderables();
        {
            let render_buffer = frame.buffer_mut();

            for r in renderables.iter() {
                let rect = r.area_rect(nask_center.center_rect);
                r.render(rect, render_buffer, state);
            }
        }
    } else {
        // render the chat
        let meta_info_height = 1; // TODO!
        let contexts_menu_height = 1; // TODO!
        let nask_chat = NaskChat::new(meta_info_height, contexts_menu_height);
        {
            let render_buffer = frame.buffer_mut();
            {
                let rect = nask_chat.chat_dialog.area_rect(frame_area);
                nask_chat.chat_dialog.render(rect, render_buffer, state);
            }
            {
                let rect = nask_chat.input_box.area_rect(frame_area);
                nask_chat.input_box.render(rect, render_buffer, state);
            }
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
    let message_loop = Arc::new(Mutex::new(MessageLoop::default()));
    let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
    let ui_sink = UiSink { tx: ui_tx };
    {
        message_loop.lock().unwrap().run(ui_sink.clone());
    }

    let event_processor = DedicatedEventProcessor;
    let ml = Arc::clone(&message_loop);
    let mut state = AppUIState::new(move |cmd: Command| ml.lock().unwrap().pump_message_loop(cmd));

    get_additional_contexts(&mut state.additional_context_state);
    get_meta_info(&mut state.meta_info_state);

    let tick_rate = Duration::from_millis(80);
    let mut last_tick = Instant::now();
    let mut redraw = true;

    let result = loop {
        // 1) Drain backend/UI events
        let mut got_ui_event = false;
        while let std::result::Result::Ok(ev) = ui_rx.try_recv() {
            got_ui_event = true;
            state.apply_ui_event(ev);
        }
        if got_ui_event {
            redraw = true;
        }

        // 2) Decide whether spinner should run (based on messages)
        let should_spin = state
            .chat_state
            .chat_messages
            .iter()
            .any(|m| m.is_response && !m.is_complete);

        match (should_spin, state.chat_state.spinner_frame.is_some()) {
            (true, false) => {
                state.chat_state.spinner_frame = Some(0);
                last_tick = Instant::now();
                redraw = true;
            }
            (false, true) => {
                state.chat_state.spinner_frame = None;
                redraw = true; // ensure ✓ renders once
            }
            _ => {}
        }

        let spinner_active = state.chat_state.spinner_frame.is_some();

        // 3) Tick spinner only when active
        if spinner_active && last_tick.elapsed() >= tick_rate {
            if let Some(frame) = &mut state.chat_state.spinner_frame {
                *frame = frame.wrapping_add(1);
            }
            last_tick = Instant::now();
            redraw = true;
        }

        // 4) Draw only when needed
        if redraw {
            terminal.draw(|f| render(f, &mut state))?;
            update_cursor_visibility(&mut terminal, &mut state.input_box_state);
            redraw = false;
        }

        // 5) Wait for input; use timeout only while spinner is active
        let timeout = if spinner_active {
            tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_millis(0))
        } else {
            Duration::from_secs(3600)
        };

        if event::poll(timeout)? {
            let ev = event::read()?;
            if EventSignal::Quit == ev.process(&mut state, &event_processor) {
                break Ok(());
            }
            redraw = true;
        }
    };

    {
        message_loop.lock().unwrap().stop();
    }
    result
}
