use crate::gui::content::{ContentConstructor, Renderable};
use anyhow::Result;
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer,
};
use wgpu::{
    CommandEncoder, Device, MultisampleState, Queue, RenderPassDescriptor, SurfaceConfiguration,
    TextureView,
};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use tracing::{debug, info, error};

pub struct PromptRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    #[allow(dead_code)]
    cache: Cache,
    viewport: glyphon::Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    buffer: Buffer,
    text: String,
    user_input: String,
    color: [f32; 4],
    background_color: Option<[f32; 4]>,
    #[allow(dead_code)]
    alignment: glyphon::cosmic_text::Align,
    is_prepared: bool,
}

impl PromptRenderer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
        text: String,
        font_size: f32,
        color: [f32; 4],
        background_color: Option<[f32; 4]>,
        _max_width: Option<u32>,
        alignment: glyphon::cosmic_text::Align,
    ) -> Result<Self> {
        info!(
            "PromptRenderer: Creating new renderer for text: '{}', font_size: {}",
            text, font_size
        );

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = glyphon::Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, config.format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        // Create buffer with the specified font size
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(font_size, font_size * 1.2));

        // Set buffer size
        let buffer_width = (config.width as f32).max(200.0) - 80.0;
        info!(
            "PromptRenderer: Setting buffer size to width: {}, height: {}",
            buffer_width, config.height
        );

        buffer.set_size(&mut font_system, Some(buffer_width), Some(config.height as f32));

        // Set initial text
        info!("PromptRenderer: Setting text content: '{}'", text);
        buffer.set_text(
            &mut font_system,
            &text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        // Apply alignment to all lines
        for line in buffer.lines.iter_mut() {
            line.set_align(Some(alignment));
        }

        info!(
            "PromptRenderer: Renderer created successfully with {} lines",
            buffer.lines.len()
        );

        Ok(Self {
            font_system,
            swash_cache,
            cache,
            viewport,
            atlas,
            text_renderer,
            buffer,
            text,
            user_input: String::new(),
            color,
            background_color,
            alignment,
            is_prepared: false,
        })
    }
}

impl Renderable for PromptRenderer {
    fn resize(&mut self, _device: &Device, queue: &Queue, config: &SurfaceConfiguration) {
        self.viewport.update(
            queue,
            Resolution {
                width: config.width,
                height: config.height,
            },
        );
        self.buffer.set_size(
            &mut self.font_system,
            Some((config.width as f32).max(200.0) - 80.0),
            Some(config.height as f32),
        );
    }

    fn handle_input(&mut self, event: &KeyEvent) -> bool {
        if event.state == ElementState::Pressed {
            match &event.logical_key {
                Key::Character(c) => {
                    if !c.chars().any(|x| x.is_control()) {
                        self.user_input.push_str(c);
                        self.is_prepared = false; // Mark that we need to re-prepare
                    }
                }
                Key::Named(NamedKey::Space) => {
                    self.user_input.push(' ');
                    self.is_prepared = false;
                }
                Key::Named(NamedKey::Backspace) => {
                    self.user_input.pop();
                    self.is_prepared = false;
                }
                Key::Named(NamedKey::Enter) => {
                    // Check if user has typed the correct text
                    if self.user_input.trim() == self.text.trim() {
                        debug!("PromptRenderer: User input matches prompt text, closing window");
                        return true;
                    }
                    // Otherwise add newline for continuation
                    self.user_input.push('\n');
                    self.is_prepared = false;
                }
                _ => {}
            }
        }
        false
    }

    fn update(&mut self, device: &Device, queue: &Queue) -> Option<std::time::Instant> {
        debug!("PromptRenderer::update called");

        let width = self.viewport.resolution().width as i32;
        let height = self.viewport.resolution().height as i32;

        // Skip rendering if viewport is invalid
        if width <= 0 || height <= 0 {
            debug!(
                "PromptRenderer: Viewport invalid ({}x{}), skipping update",
                width, height
            );
            return None;
        }

        // Update buffer text with prompt + user input
        let display_text = if self.user_input.is_empty() {
            format!("{}\n\nStart typing...", self.text)
        } else {
            format!("{}\n\n> {}", self.text, self.user_input)
        };

        self.buffer.set_text(
            &mut self.font_system,
            &display_text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        // Create bounds for text layout
        let padding = 40i32;
        let bounds = TextBounds {
            left: padding,
            top: padding,
            right: (width - padding).max(padding + 1),
            bottom: (height - padding).max(padding + 1),
        };

        debug!(
            "PromptRenderer: Preparing text render with bounds: left={}, top={}, right={}, bottom={}",
            bounds.left, bounds.top, bounds.right, bounds.bottom
        );

        // Prepare for rendering
        match self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            [TextArea {
                buffer: &self.buffer,
                left: 40.0,
                top: 60.0,
                scale: 1.0,
                bounds,
                default_color: Color::rgba(
                    (self.color[0] * 255.0).clamp(0.0, 255.0) as u8,
                    (self.color[1] * 255.0).clamp(0.0, 255.0) as u8,
                    (self.color[2] * 255.0).clamp(0.0, 255.0) as u8,
                    (self.color[3] * 255.0).clamp(0.0, 255.0) as u8,
                ),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        ) {
            Ok(_) => {
                debug!("PromptRenderer::update: prepare successful");
                self.is_prepared = true;
            }
            Err(e) => {
                error!("PromptRenderer::update: prepare failed: {:?}", e);
                self.is_prepared = false;
            }
        }

        None
    }

    fn render(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        _queue: &Queue,
        opacity: f32,
    ) {
        debug!("PromptRenderer::render called with opacity: {}", opacity);

        // Clear pass
        let load_op = if let Some(bg) = self.background_color {
            wgpu::LoadOp::Clear(wgpu::Color {
                r: bg[0] as f64,
                g: bg[1] as f64,
                b: bg[2] as f64,
                a: bg[3] as f64 * opacity as f64,
            })
        } else {
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.8 * opacity as f64,
            })
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Prompt Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        debug!("PromptRenderer: About to render text with color: {:?}", self.color);

        if self.is_prepared {
            if let Err(e) = self.text_renderer.render(&self.atlas, &self.viewport, &mut pass) {
                error!("PromptRenderer: Failed to render text: {}", e);
            } else {
                debug!("PromptRenderer: Text rendered successfully");
            }
        } else {
            debug!("PromptRenderer: Skipping render - text not prepared");
        }
    }
}

pub struct PromptContent {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub background_color: Option<[f32; 4]>,
    pub max_width: Option<u32>,
    pub alignment: glyphon::cosmic_text::Align,
}

impl ContentConstructor for PromptContent {
    fn create_renderer(
        &self,
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
    ) -> Result<Box<dyn Renderable>> {
        let renderer = PromptRenderer::new(
            device,
            queue,
            config,
            self.text.clone(),
            self.font_size,
            self.color,
            self.background_color,
            self.max_width,
            self.alignment,
        )?;
        Ok(Box::new(renderer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_text_setting() {
        let mut font_system = FontSystem::new();
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(32.0, 32.0 * 1.2));

        buffer.set_size(&mut font_system, Some(720.0), Some(600.0));

        let text = "Type this text";
        buffer.set_text(
            &mut font_system,
            text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        // Verify buffer has lines
        assert!(
            !buffer.lines.is_empty(),
            "Buffer should have lines after setting text"
        );
        assert_eq!(buffer.lines.len(), 1, "Buffer should have 1 line for single-line text");
    }

    #[test]
    fn test_multiline_buffer() {
        let mut font_system = FontSystem::new();
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(32.0, 32.0 * 1.2));

        buffer.set_size(&mut font_system, Some(720.0), Some(600.0));

        let text = "Line 1\n\nLine 3";
        buffer.set_text(
            &mut font_system,
            text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        assert!(!buffer.lines.is_empty(), "Buffer should have lines");
        assert_eq!(buffer.lines.len(), 3, "Buffer should have 3 lines");
    }
}
