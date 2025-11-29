use crate::gui::window_manager::WindowHandle;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum MessageType {
    User,
    System,
    Assistant,
    Error,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub msg_type: MessageType,
    pub content: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub handle: WindowHandle,
    pub created: Instant,
    pub timeout: Option<Duration>,
}

pub struct LoopState {
    pub iteration_count: u64,
    pub last_execution: Instant,
    pub conversation_history: VecDeque<Message>,
    pub retry_count: usize,
    pub active_windows: HashMap<WindowHandle, WindowInfo>,
}

impl Default for LoopState {
    fn default() -> Self {
        Self::new()
    }
}

impl LoopState {
    pub fn new() -> Self {
        Self {
            iteration_count: 0,
            last_execution: Instant::now(),
            conversation_history: VecDeque::new(),
            retry_count: 0,
            active_windows: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, msg_type: MessageType, content: String) {
        self.conversation_history.push_back(Message {
            msg_type,
            content,
            timestamp: Instant::now(),
        });
        // Keep history size manageable (e.g., last 50 messages)
        if self.conversation_history.len() > 50 {
            self.conversation_history.pop_front();
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.add_message(MessageType::Error, error);
        self.retry_count += 1;
    }

    pub fn reset_retry(&mut self) {
        self.retry_count = 0;
    }

    pub fn register_window(&mut self, handle: WindowHandle, timeout: Option<Duration>) {
        self.active_windows.insert(
            handle,
            WindowInfo {
                handle,
                created: Instant::now(),
                timeout,
            },
        );
    }
}
