use crate::gui::content::{ContentConstructor, Renderable};
use anyhow::Result;
use tracing::{debug, info};
use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};
use winit::event::KeyEvent;

slint::include_modules!();

pub struct PromptContent {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub background_color: Option<[f32; 4]>,
    pub max_width: Option<u32>,
    pub alignment: String,
}

impl ContentConstructor for PromptContent {
    fn create_renderer(
        &self,
        _device: &Device,
        _queue: &Queue,
        _config: &SurfaceConfiguration,
    ) -> Result<Box<dyn Renderable>> {
        let renderer = PromptRenderer::new(
            self.text.clone(),
            self.font_size,
            self.color,
            self.background_color,
            self.alignment.clone(),
        )?;
        Ok(Box::new(renderer))
    }
}

pub struct PromptRenderer {
    window: slint::Weak<PromptWindow>,
    text: String,
    user_input: String,
    last_cursor_toggle: std::time::Instant,
    show_cursor: bool,
}

impl PromptRenderer {
    pub fn new(
        text: String,
        font_size: f32,
        color: [f32; 4],
        background_color: Option<[f32; 4]>,
        alignment: String,
    ) -> Result<Self> {
        info!(
            "PromptRenderer: Creating new Slint-based renderer for text: '{}', font_size: {}",
            text, font_size
        );

        let window = PromptWindow::new().unwrap();

        // Set properties
        window.set_prompt_text(text.clone().into());
        window.set_font_size(font_size);
        window.set_text_color(slint::Color::from_rgb_u8(
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
        ));

        let bg_color = background_color.unwrap_or([0.0, 0.0, 0.0, 1.0]);
        window.set_background_color(slint::Color::from_rgb_u8(
            (bg_color[0] * 255.0) as u8,
            (bg_color[1] * 255.0) as u8,
            (bg_color[2] * 255.0) as u8,
        ));

        window.set_alignment(alignment.into());

        // Run the Slint event loop to show the window
        // This will display the window and keep it responsive
        info!("PromptRenderer: Running Slint event loop to display window");
        slint::run_event_loop()
            .map_err(|e| anyhow::anyhow!("Failed to run Slint event loop: {}", e))?;

        info!("PromptRenderer: Slint window completed");

        Ok(Self {
            window: window.as_weak(),
            text,
            user_input: String::new(),
            last_cursor_toggle: std::time::Instant::now(),
            show_cursor: true,
        })
    }
}

impl Renderable for PromptRenderer {
    fn resize(&mut self, _device: &Device, _queue: &Queue, _config: &SurfaceConfiguration) {
        debug!("PromptRenderer: Resize called (handled by Slint)");
    }

    fn handle_input(&mut self, event: &KeyEvent) -> bool {
        use winit::event::ElementState;
        use winit::keyboard::{Key, NamedKey};

        if event.state == ElementState::Pressed {
            // Reset cursor blink on any input
            self.last_cursor_toggle = std::time::Instant::now();
            self.show_cursor = true;

            match &event.logical_key {
                Key::Character(c) => {
                    if !c.chars().any(|x| x.is_control()) {
                        self.user_input.push_str(c);
                        if let Some(window) = self.window.upgrade() {
                            window.set_user_input(self.user_input.clone().into());
                        }
                    }
                }
                Key::Named(NamedKey::Space) => {
                    self.user_input.push(' ');
                    if let Some(window) = self.window.upgrade() {
                        window.set_user_input(self.user_input.clone().into());
                    }
                }
                Key::Named(NamedKey::Backspace) => {
                    self.user_input.pop();
                    if let Some(window) = self.window.upgrade() {
                        window.set_user_input(self.user_input.clone().into());
                    }
                }
                Key::Named(NamedKey::Enter) => {
                    // Check if user has typed the correct text
                    if self.user_input.trim() == self.text.trim() {
                        debug!("PromptRenderer: User input matches prompt text, closing window");
                        return true;
                    }
                    // Otherwise add newline for continuation
                    self.user_input.push('\n');
                    if let Some(window) = self.window.upgrade() {
                        window.set_user_input(self.user_input.clone().into());
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn update(&mut self, _device: &Device, _queue: &Queue) -> Option<std::time::Instant> {
        // Update cursor blink state (blink every 500ms)
        let elapsed = self.last_cursor_toggle.elapsed().as_millis();
        if elapsed > 500 {
            self.show_cursor = !self.show_cursor;
            self.last_cursor_toggle = std::time::Instant::now();

            if let Some(window) = self.window.upgrade() {
                window.set_show_cursor(self.show_cursor);
            }
        }

        None
    }

    fn render(
        &self,
        _encoder: &mut CommandEncoder,
        _view: &TextureView,
        _queue: &Queue,
        _opacity: f32,
    ) {
        // Slint handles rendering internally
        debug!("PromptRenderer: Render called (handled by Slint internally)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires GUI display to create Slint windows"]
    fn test_prompt_creation() {
        let result = PromptRenderer::new(
            "Test prompt".to_string(),
            32.0,
            [1.0, 1.0, 1.0, 1.0],
            Some([0.0, 0.0, 0.0, 1.0]),
            "left".to_string(),
        );

        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Requires GUI display to create Slint windows"]
    fn test_user_input_handling() {
        let renderer = PromptRenderer::new(
            "Test prompt".to_string(),
            32.0,
            [1.0, 1.0, 1.0, 1.0],
            None,
            "left".to_string(),
        )
        .unwrap();

        // Simulate text input would happen in handle_input
        // This test just verifies the renderer can be created
        assert_eq!(renderer.user_input, "");
    }
}
