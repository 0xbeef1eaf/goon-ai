use super::content::{ContentConstructor, Renderable};
use super::renderer::Renderer;
use super::window_manager::WindowOptions;
use anyhow::Result;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::sync::Arc;
use std::time::Instant;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window as WinitWindow, WindowAttributes};

pub struct Window {
    pub winit_window: Arc<WinitWindow>,
    pub renderer: Option<Renderer>,
    pub content_renderer: Option<Box<dyn Renderable>>,
    pub opacity: f32,
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

        let renderer = pollster::block_on(Renderer::new(winit_window.clone()))?;

        Ok(Self {
            winit_window,
            renderer: Some(renderer),
            content_renderer: None,
            opacity: options.opacity,
        })
    }

    pub async fn init_renderer(&mut self) -> Result<()> {
        let renderer = Renderer::new(self.winit_window.clone()).await?;
        self.renderer = Some(renderer);
        Ok(())
    }

    pub fn set_content(&mut self, content: Box<dyn ContentConstructor>) -> Result<()> {
        if let Some(renderer) = &self.renderer {
            let content_renderer =
                content.create_renderer(&renderer.device, &renderer.queue, &renderer.config)?;
            self.content_renderer = Some(content_renderer);
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let opacity = self.get_render_opacity();
        if let Some(renderer) = &mut self.renderer {
            // Update content
            if let Some(content) = &mut self.content_renderer {
                content.update(&renderer.queue);
            }

            let content_renderer = self.content_renderer.as_ref();

            renderer.render(opacity, |encoder, view, queue| {
                if let Some(cr) = content_renderer {
                    cr.render(encoder, view, queue, opacity);
                } else {
                    // Clear pass
                    let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                }
            })?;
        }
        Ok(())
    }

    pub fn get_next_update_time(&mut self) -> Option<Instant> {
        if let Some(renderer) = &self.renderer
            && let Some(content) = &mut self.content_renderer
        {
            return content.update(&renderer.queue);
        }
        None
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
        self.winit_window
            .set_cursor_hittest(!click_through)
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
        let handle_wrapper = match self.winit_window.window_handle() {
            Ok(h) => h,
            Err(_) => return,
        };
        let handle = handle_wrapper.as_raw();

        #[cfg(target_os = "windows")]
        if let RawWindowHandle::Win32(handle) = handle {
            use windows_sys::Win32::Foundation::HWND;
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                GWL_EXSTYLE, GetWindowLongPtrW, LWA_ALPHA, SetLayeredWindowAttributes,
                SetWindowLongPtrW, WS_EX_LAYERED,
            };

            let hwnd = handle.hwnd.get() as HWND;
            unsafe {
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                // Ensure WS_EX_LAYERED is set
                if (style as u32 & WS_EX_LAYERED) == 0 {
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED as isize);
                }
                let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
                SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
            }
        }

        #[cfg(target_os = "macos")]
        if let RawWindowHandle::AppKit(handle) = handle {
            use objc::{msg_send, sel, sel_impl};
            let ns_window = handle.ns_window.as_ptr() as *mut objc::runtime::Object;
            unsafe {
                let _: () = msg_send![ns_window, setAlphaValue: opacity.clamp(0.0, 1.0) as f64];
            }
        }

        #[cfg(target_os = "linux")]
        match handle {
            RawWindowHandle::Xlib(handle) => {
                use x11rb::connection::Connection;
                use x11rb::protocol::xproto::{
                    AtomEnum, ConnectionExt as XProtoConnectionExt, PropMode,
                };
                use x11rb::wrapper::ConnectionExt;

                if let Ok((conn, _)) = x11rb::connect(None) {
                    let window_id = handle.window as u32;
                    let atom_name = b"_NET_WM_WINDOW_OPACITY";

                    if let Some(reply) = conn
                        .intern_atom(false, atom_name)
                        .ok()
                        .and_then(|c| c.reply().ok())
                    {
                        let atom = reply.atom;
                        let opacity_u32 = (opacity.clamp(0.0, 1.0) * 0xFFFFFFFFu32 as f32) as u32;

                        let _ = conn.change_property32(
                            PropMode::REPLACE,
                            window_id,
                            atom,
                            AtomEnum::CARDINAL,
                            &[opacity_u32],
                        );
                        let _ = conn.flush();
                    }
                }
            }
            RawWindowHandle::Wayland(_) => {
                // Wayland does not support server-side window opacity via standard protocols.
                // Opacity must be handled during rendering by applying alpha to the content.
                // The self.opacity field is updated and should be used by the renderer.
            }
            _ => {}
        }
    }

    pub fn get_render_opacity(&self) -> f32 {
        #[cfg(target_os = "linux")]
        if self
            .winit_window
            .window_handle()
            .is_ok_and(|h| matches!(h.as_raw(), RawWindowHandle::Wayland(_)))
        {
            return self.opacity;
        }
        1.0
    }
}
