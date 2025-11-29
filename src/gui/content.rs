use anyhow::Result;
use std::time::Instant;
use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

pub trait Renderable: Send + Sync {
    fn render(&self, encoder: &mut CommandEncoder, view: &TextureView, queue: &Queue, opacity: f32);
    fn update(&mut self, _device: &Device, _queue: &Queue) -> Option<Instant> {
        None
    }
    fn handle_input(&mut self, _event: &winit::event::KeyEvent) -> bool {
        false
    }
    fn resize(&mut self, _device: &Device, _queue: &Queue, _config: &SurfaceConfiguration) {}
}

pub trait ContentConstructor: Send {
    fn create_renderer(
        &self,
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
    ) -> Result<Box<dyn Renderable>>;
}
