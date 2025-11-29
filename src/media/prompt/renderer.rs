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

pub struct PromptRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    #[allow(dead_code)]
    cache: Cache,
    viewport: glyphon::Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    buffer: Buffer,
    #[allow(dead_code)]
    text: String,
    user_input: String,
    alignment: glyphon::cosmic_text::Align,
    color: [f32; 4],
    background_color: Option<[f32; 4]>,
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
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = glyphon::Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, config.format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        let mut buffer = Buffer::new(&mut font_system, Metrics::new(font_size, font_size * 1.2));

        buffer.set_size(
            &mut font_system,
            Some(config.width as f32),
            Some(config.height as f32),
        );
        buffer.set_text(
            &mut font_system,
            &text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        // Apply alignment
        for line in buffer.lines.iter_mut() {
            line.set_align(Some(alignment));
        }

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
            alignment,
            color,
            background_color,
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
            Some(config.width as f32),
            Some(config.height as f32),
        );
    }

    fn handle_input(&mut self, event: &KeyEvent) -> bool {
        if event.state == ElementState::Pressed {
            match &event.logical_key {
                Key::Character(c) => {
                    if !c.chars().any(|x| x.is_control()) {
                        self.user_input.push_str(c);
                    }
                }
                Key::Named(NamedKey::Space) => {
                    self.user_input.push(' ');
                }
                Key::Named(NamedKey::Backspace) => {
                    self.user_input.pop();
                }
                Key::Named(NamedKey::Enter) => {
                    self.user_input.push('\n');
                }
                _ => {}
            }

            let display_text = format!("{}\n{}", self.text, self.user_input);
            self.buffer.set_text(
                &mut self.font_system,
                &display_text,
                &Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );

            for line in self.buffer.lines.iter_mut() {
                line.set_align(Some(self.alignment));
            }

            if self.user_input.trim() == self.text.trim() {
                return true;
            }
        }
        false
    }

    fn update(&mut self, device: &Device, queue: &Queue) -> Option<std::time::Instant> {
        let _ = self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            [TextArea {
                buffer: &self.buffer,
                left: 0.0,
                top: 0.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: self.viewport.resolution().width as i32,
                    bottom: self.viewport.resolution().height as i32,
                },
                default_color: Color::rgba(
                    (self.color[0] * 255.0) as u8,
                    (self.color[1] * 255.0) as u8,
                    (self.color[2] * 255.0) as u8,
                    (self.color[3] * 255.0) as u8,
                ),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        );
        None
    }

    fn render(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        _queue: &Queue,
        opacity: f32,
    ) {
        // Clear pass if background color is set
        {
            let load_op = if let Some(bg) = self.background_color {
                wgpu::LoadOp::Clear(wgpu::Color {
                    r: bg[0] as f64,
                    g: bg[1] as f64,
                    b: bg[2] as f64,
                    a: bg[3] as f64 * opacity as f64,
                })
            } else {
                wgpu::LoadOp::Load
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

            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .unwrap();
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
