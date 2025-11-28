use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::WindowId;
use std::sync::mpsc::channel;
use super::event_loop::GuiCommand;

use super::window::Window;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowHandle(pub Uuid);

use std::time::{Duration, Instant};
use winit::event::{ElementState, KeyEvent, MouseButton};

#[derive(Debug, Clone)]
pub enum WindowMessage {
    CloseRequested(WindowHandle),
    Resized(WindowHandle, u32, u32),
    KeyboardInput(WindowHandle, KeyEvent),
    MouseInput(WindowHandle, MouseButton, ElementState),
    CursorMoved(WindowHandle, f64, f64),
}

#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub title: Option<String>,
    pub position: Option<(i32, i32)>,
    pub size: Option<(u32, u32)>,
    pub always_on_top: bool,
    pub decorations: bool,
    pub opacity: f32,
    pub click_through: bool,
    pub resizable: bool,
    pub visible: bool,
    pub timeout: Option<Duration>,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: None,
            position: None,
            size: None,
            always_on_top: false,
            decorations: true,
            opacity: 1.0,
            click_through: false,
            resizable: true,
            visible: true,
            timeout: None,
        }
    }
}

#[derive(Clone)]
pub struct GuiController {
    proxy: EventLoopProxy<GuiCommand>,
}

impl GuiController {
    pub fn new(proxy: EventLoopProxy<GuiCommand>) -> Self {
        Self { proxy }
    }

    pub fn create_window(&self, options: WindowOptions) -> Result<WindowHandle> {
        let (tx, rx) = channel();
        self.proxy.send_event(GuiCommand::CreateWindow(options, tx))
            .map_err(|_| anyhow::anyhow!("Event loop closed"))?;
        rx.recv().map_err(|_| anyhow::anyhow!("Failed to receive response"))?
    }

    pub fn close_window(&self, handle: WindowHandle) -> Result<()> {
        self.proxy.send_event(GuiCommand::CloseWindow(handle))
            .map_err(|_| anyhow::anyhow!("Event loop closed"))?;
        Ok(())
    }
}

pub struct WindowManager {
    windows: HashMap<WindowHandle, Window>,
    winit_to_handle: HashMap<WindowId, WindowHandle>,
    deadlines: Vec<(Instant, WindowHandle)>,
    messages: Vec<WindowMessage>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            winit_to_handle: HashMap::new(),
            deadlines: Vec::new(),
            messages: Vec::new(),
        }
    }

    pub fn create_window(&mut self, options: WindowOptions, event_loop: &ActiveEventLoop) -> Result<WindowHandle> {
        let id = Uuid::new_v4();
        let handle = WindowHandle(id);
        
        if let Some(timeout) = options.timeout {
            self.deadlines.push((Instant::now() + timeout, handle));
        }

        let window = Window::new(options, event_loop)?;
        let window_id = window.winit_window.id();
        
        self.windows.insert(handle, window);
        self.winit_to_handle.insert(window_id, handle);
        
        Ok(handle)
    }

    pub fn close_window(&mut self, handle: WindowHandle) {
        if let Some(window) = self.windows.remove(&handle) {
            self.winit_to_handle.remove(&window.winit_window.id());
        }
        // Remove from deadlines
        self.deadlines.retain(|(_, h)| *h != handle);
    }
    
    pub fn check_timeouts(&mut self) -> Option<Instant> {
        let now = Instant::now();
        let mut expired = Vec::new();
        
        // Find expired windows
        self.deadlines.retain(|(deadline, handle)| {
            if *deadline <= now {
                expired.push(*handle);
                false
            } else {
                true
            }
        });
        
        // Close expired windows
        for handle in expired {
            self.close_window(handle);
        }
        
        // Return next deadline
        self.deadlines.iter().map(|(d, _)| *d).min()
    }
    
    pub fn get_window(&self, handle: WindowHandle) -> Option<&Window> {
        self.windows.get(&handle)
    }
    
    pub fn get_window_mut(&mut self, handle: WindowHandle) -> Option<&mut Window> {
        self.windows.get_mut(&handle)
    }

    pub fn get_handle_from_winit(&self, window_id: WindowId) -> Option<WindowHandle> {
        self.winit_to_handle.get(&window_id).copied()
    }

    pub fn push_message(&mut self, message: WindowMessage) {
        self.messages.push(message);
    }

    pub fn poll_messages(&mut self) -> Vec<WindowMessage> {
        std::mem::take(&mut self.messages)
    }

    #[cfg(test)]
    pub fn add_test_deadline(&mut self, handle: WindowHandle, timeout: Duration) {
        self.deadlines.push((std::time::Instant::now() + timeout, handle));
    }
    
    #[cfg(test)]
    pub fn get_window_count(&self) -> usize {
        self.windows.len()
    }
}
