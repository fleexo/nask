use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{JoinHandle, Thread},
};

pub enum Command {
    ChatMessage(String),
}

pub struct ThreadSafeDeque<Command> {
    deque: Mutex<VecDeque<Command>>,
}

impl<Command> ThreadSafeDeque<Command> {
    pub fn new() -> Self {
        Self {
            deque: Mutex::new(VecDeque::new()),
        }
    }

    pub fn push(&self, cmd: Command) {
        let mut q = self.deque.lock().expect("ThreadSafeDeque mutex poisoned");
        q.push_back(cmd);
    }

    pub fn pop(&self) -> Option<Command> {
        let mut q = self.deque.lock().expect("ThreadSafeDeque mutex poisoned");
        q.pop_front()
    }
}

pub struct MessageLoop {
    loop_running: Arc<AtomicBool>,
    message_deque: Arc<ThreadSafeDeque<Command>>, // I dont want to implement a queue atm
    loop_thread: Option<JoinHandle<()>>,
}

impl Default for MessageLoop {
    fn default() -> Self {
        Self {
            loop_running: Arc::new(AtomicBool::new(false)),
            message_deque: Arc::new(ThreadSafeDeque::new()),
            loop_thread: None,
        }
    }
}

impl MessageLoop {
    pub fn pump_message_loop(&mut self, cmd: Command) {
        if !self.loop_running.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        self.message_deque.push(cmd);
    }

    pub fn run(&mut self) {
        if self.loop_thread.is_some() {
            return;
        }

        self.loop_running.store(true, Ordering::Relaxed);

        let running = Arc::clone(&self.loop_running);
        let thread_msg_deque = Arc::clone(&self.message_deque);
        self.loop_thread = Some(std::thread::spawn(move || {
            let mut current_command = None;
            while running.load(Ordering::Relaxed) && current_command.is_some() {
                current_command = thread_msg_deque.pop();
                std::thread::yield_now(); // placeholder so it doesn't hard-spin
            }
        }));
    }

    pub fn stop(&mut self) {
        self.loop_running.store(false, Ordering::Relaxed);
        if let Some(h) = self.loop_thread.take() {
            let _ = h.join();
        }
    }
}
