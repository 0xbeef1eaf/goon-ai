use anyhow::Result;
use winit::window::{Window as WinitWindow, WindowAttributes};
use winit::event_loop::ActiveEventLoop;
use super::window_manager::WindowOptions;
use super::renderer::Renderer;
use std::sync::Arc;

pub struct Window {
    pub winit_window: Arc<WinitWindow>,
    pub renderer: Option<Renderer>,
}

impl Window {
    pub fn new(options: WindowOptions, event_loop: &ActiveEventLoop) -> Result<Self> {
        let mut attributes = WindowAttributes::default()
            .with_decorations(options.decorations)
            .with_transparent(true)
            .with_resizable(options.resizable)
            .with_visible(options.visible);

        if let Some(title) = options.title {
            attributes = attributes.with_title(title);
        }

        if let Some(size) = options.size {
            attributes = attributes.with_inner_size(winit::dpi::LogicalSize::new(size.0, size.1));
        }
        
        if let Some(pos) = options.position {
             attributes = attributes.with_position(winit::dpi::LogicalPosition::new(pos.0, pos.1));
        }
        
        if options.always_on_top {
             attributes = attributes.with_window_level(winit::window::WindowLevel::AlwaysOnTop);
        }

        let winit_window = Arc::new(event_loop.create_window(attributes)?);
        
        Ok(Self {
            winit_window,
            renderer: None,
        })
    }

    pub async fn init_renderer(&mut self) -> Result<()> {
        let renderer = Renderer::new(self.winit_window.clone()).await?;
        self.renderer = Some(renderer);
        Ok(())
    }

    pub fn set_always_on_top(&self, always_on_top: bool) {
        let level = if always_on_top {
            winit::window::WindowLevel::AlwaysOnTop
        } else {
            winit::window::WindowLevel::Normal
        };
        self.winit_window.set_window_level(level);
    }

    pub fn set_click_through(&self, click_through: bool) -> Result<()> {
        self.winit_window.set_cursor_hittest(!click_through).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn set_opacity(&self, opacity: f32) {
        // TODO: Implement platform-specific opacity
        // This requires accessing raw window handles and calling platform APIs
    }
}
