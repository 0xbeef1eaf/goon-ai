use super::VideoWindow;
use super::types::WindowHandle;
use crate::media::video::player::{self, Player, Rescaler};
use anyhow::Result;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::ComponentHandle;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Video player state
pub struct VideoState {
    pub window: Rc<VideoWindow>,
    pub player: Arc<Mutex<Player>>,
}

pub fn spawn(
    handle: WindowHandle,
    path: &std::path::Path,
    width: Option<u32>,
    height: Option<u32>,
    opacity: f32,
) -> Result<VideoState> {
    let window = VideoWindow::new()?;
    let window = Rc::new(window);

    // Set initial properties
    window.set_video_opacity(opacity);
    if let Some(w) = width {
        window.set_video_width(w as i32);
    }
    if let Some(h) = height {
        window.set_video_height(h as i32);
    }

    // RGB rescaler for converting frames
    let mut to_rgb_rescaler: Option<Rescaler> = None;

    // Create player with frame callback
    let window_weak = window.as_weak();
    let player = Player::start(
        path.to_path_buf(),
        move |new_frame| {
            // Rebuild rescaler if format changed
            let rebuild_rescaler = to_rgb_rescaler.as_ref().is_none_or(|existing_rescaler| {
                existing_rescaler.input().format != new_frame.format()
            });

            if rebuild_rescaler {
                to_rgb_rescaler = Some(player::rgb_rescaler_for_frame(new_frame));
            }

            let rescaler = to_rgb_rescaler.as_mut().unwrap();

            let mut rgb_frame = ffmpeg_next::util::frame::Video::empty();
            rescaler.run(new_frame, &mut rgb_frame).unwrap();

            let pixel_buffer = player::video_frame_to_pixel_buffer(&rgb_frame);
            let _ = window_weak.upgrade_in_event_loop(move |window| {
                window.set_video_frame(slint::Image::from_rgb8(pixel_buffer));
            });
        },
        {
            let window_weak = window.as_weak();
            move |playing| {
                let _ = window_weak.upgrade_in_event_loop(move |window| {
                    window.set_playing(playing);
                });
            }
        },
    )?;

    let player = Arc::new(Mutex::new(player));

    // Show window
    window.show()?;

    // Configure native window properties asynchronously
    let window_weak = window.as_weak();
    let _ = slint::spawn_local(async move {
        if let Some(window) = window_weak.upgrade()
            && let Ok(winit_window) = window.window().winit_window().await
        {
            winit_window
                .set_window_level(i_slint_backend_winit::winit::window::WindowLevel::AlwaysOnTop);
            winit_window.set_resizable(false);
            winit_window.set_decorations(false);
            winit_window.set_window_icon(None);
        }
    });

    debug!("Spawned video window: {:?}", handle);
    Ok(VideoState { window, player })
}
