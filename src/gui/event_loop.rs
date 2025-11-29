use super::content::ContentConstructor;
use super::window_manager::{WindowHandle, WindowManager, WindowMessage, WindowOptions};
use anyhow::Result;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::WindowId;

pub enum GuiCommand {
    CreateWindow(WindowOptions, Sender<Result<WindowHandle>>),
    CloseWindow(WindowHandle),
    SetContent(WindowHandle, Box<dyn ContentConstructor>),
}

pub struct App {
    window_manager: Arc<Mutex<WindowManager>>,
}

impl App {
    pub fn new(window_manager: Arc<Mutex<WindowManager>>) -> Self {
        Self { window_manager }
    }
}

impl ApplicationHandler<GuiCommand> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Resumed
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: GuiCommand) {
        match event {
            GuiCommand::CreateWindow(options, reply_sender) => {
                let mut wm = self.window_manager.lock().unwrap();
                let result = wm.create_window(options, event_loop);
                let _ = reply_sender.send(result);
            }
            GuiCommand::CloseWindow(handle) => {
                let mut wm = self.window_manager.lock().unwrap();
                wm.close_window(handle);
            }
            GuiCommand::SetContent(handle, content) => {
                let mut wm = self.window_manager.lock().unwrap();
                if let Some(window) = wm.get_window_mut(handle)
                    && let Err(e) = window.set_content(content)
                {
                    eprintln!("Failed to set content: {}", e);
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut wm = self.window_manager.lock().unwrap();
        wm.request_redraws();
        if let Some(next_deadline) = wm.check_timeouts() {
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_deadline));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                let mut wm = self.window_manager.lock().unwrap();
                if let Some(handle) = wm.get_handle_from_winit(window_id) {
                    wm.push_message(WindowMessage::CloseRequested(handle));
                }
            }
            WindowEvent::Resized(physical_size) => {
                let mut wm = self.window_manager.lock().unwrap();
                if let Some(handle) = wm.get_handle_from_winit(window_id) {
                    if let Some(window) = wm.get_window_mut(handle) {
                        window.resize(physical_size);
                    }
                    wm.push_message(WindowMessage::Resized(
                        handle,
                        physical_size.width,
                        physical_size.height,
                    ));
                }
            }
            WindowEvent::RedrawRequested => {
                let mut wm = self.window_manager.lock().unwrap();
                if let Some(window) = wm
                    .get_handle_from_winit(window_id)
                    .and_then(|h| wm.get_window_mut(h))
                {
                    match window.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            window.resize(window.winit_window.inner_size());
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let mut wm = self.window_manager.lock().unwrap();
                if let Some(handle) = wm.get_handle_from_winit(window_id) {
                    let should_close = if let Some(window) = wm.get_window_mut(handle) {
                        window.handle_input(&event)
                    } else {
                        false
                    };

                    if should_close {
                        wm.close_window(handle);
                    }

                    wm.push_message(WindowMessage::KeyboardInput(handle, event));
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let mut wm = self.window_manager.lock().unwrap();
                if let Some(handle) = wm.get_handle_from_winit(window_id) {
                    wm.push_message(WindowMessage::CursorMoved(handle, position.x, position.y));
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mut wm = self.window_manager.lock().unwrap();
                if let Some(handle) = wm.get_handle_from_winit(window_id) {
                    wm.push_message(WindowMessage::MouseInput(handle, button, state));
                }
            }
            _ => (),
        }
    }
}

pub fn create_event_loop() -> Result<(EventLoop<GuiCommand>, EventLoopProxy<GuiCommand>)> {
    let event_loop = EventLoop::<GuiCommand>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    Ok((event_loop, proxy))
}

pub fn run_event_loop(
    event_loop: EventLoop<GuiCommand>,
    window_manager: Arc<Mutex<WindowManager>>,
) -> Result<()> {
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(window_manager);
    event_loop.run_app(&mut app)?;
    Ok(())
}
