//! FFmpeg video player implementation
//!
//! Provides a video player that decodes video/audio using FFmpeg and
//! renders frames to Slint windows.

use anyhow::Result;
use futures::{FutureExt, future::OptionFuture};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::audio::AudioPlaybackThread;

/// Control commands for the player
#[derive(Clone, Copy, Debug)]
pub enum ControlCommand {
    Play,
    Pause,
}

/// Unique handle for a video instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoHandle(pub Uuid);

impl VideoHandle {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for VideoHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Video player that uses FFmpeg for decoding
pub struct Player {
    control_sender: smol::channel::Sender<ControlCommand>,
    demuxer_thread: Option<std::thread::JoinHandle<()>>,
    playing: bool,
    playing_changed_callback: Arc<dyn Fn(bool) + Send + Sync>,
}

impl Player {
    /// Start playing a video file
    ///
    /// # Arguments
    /// * `path` - Path to the video file (can be a URL or local path)
    /// * `video_frame_callback` - Called with each decoded video frame
    /// * `playing_changed_callback` - Called when play/pause state changes
    pub fn start<P: Into<PathBuf>>(
        path: P,
        video_frame_callback: impl FnMut(&ffmpeg_next::util::frame::Video) + Send + 'static,
        playing_changed_callback: impl Fn(bool) + Send + Sync + 'static,
    ) -> Result<Self> {
        let path = path.into();
        let (control_sender, control_receiver) = smol::channel::unbounded();
        let playing_changed = Arc::new(playing_changed_callback);
        let playing_changed_for_thread = playing_changed.clone();

        let demuxer_thread = std::thread::Builder::new()
            .name("video demuxer thread".into())
            .spawn(move || {
                smol::block_on(async move {
                    // Open input
                    let path_str = path.to_string_lossy().to_string();
                    let mut input_context = match ffmpeg_next::format::input(&path_str) {
                        Ok(ctx) => ctx,
                        Err(e) => {
                            eprintln!("Failed to open video file: {}", e);
                            return;
                        }
                    };

                    // Find video stream
                    let video_stream = match input_context
                        .streams()
                        .best(ffmpeg_next::media::Type::Video)
                    {
                        Some(s) => s,
                        None => {
                            eprintln!("No video stream found");
                            return;
                        }
                    };
                    let video_stream_index = video_stream.index();
                    let video_playback_thread = match VideoPlaybackThread::start(
                        &video_stream,
                        Box::new(video_frame_callback),
                    ) {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("Failed to start video playback: {}", e);
                            return;
                        }
                    };

                    // Find audio stream (optional)
                    let audio_info = input_context
                        .streams()
                        .best(ffmpeg_next::media::Type::Audio)
                        .and_then(|audio_stream| {
                            let audio_stream_index = audio_stream.index();
                            AudioPlaybackThread::start(&audio_stream)
                                .ok()
                                .map(|thread| (audio_stream_index, thread))
                        });

                    let mut playing = true;

                    // Packet forwarding future
                    let packet_forwarder_impl = async {
                        for (stream, packet) in input_context.packets() {
                            if let Some((audio_idx, ref audio_thread)) = audio_info
                                && stream.index() == audio_idx
                            {
                                audio_thread.receive_packet(packet.clone()).await;
                                continue;
                            }
                            if stream.index() == video_stream_index {
                                video_playback_thread.receive_packet(packet).await;
                            }
                        }
                    }
                    .fuse()
                    .shared();

                    loop {
                        let packet_forwarder: OptionFuture<_> = if playing {
                            Some(packet_forwarder_impl.clone())
                        } else {
                            None
                        }
                        .into();

                        smol::pin!(packet_forwarder);

                        futures::select! {
                            _ = packet_forwarder => {
                                // Playback finished
                                break;
                            },
                            received_command = control_receiver.recv().fuse() => {
                                match received_command {
                                    Ok(command) => {
                                        video_playback_thread.send_control_message(command).await;
                                        if let Some((_, ref audio_thread)) = audio_info {
                                            audio_thread.send_control_message(command).await;
                                        }
                                        match command {
                                            ControlCommand::Play => {
                                                playing = true;
                                            },
                                            ControlCommand::Pause => {
                                                playing = false;
                                            }
                                        }
                                        playing_changed_for_thread(playing);
                                    }
                                    Err(_) => {
                                        // Channel closed -> quit
                                        break;
                                    }
                                }
                            }
                        }
                    }
                })
            })?;

        let playing = true;
        playing_changed(playing);

        Ok(Self {
            control_sender,
            demuxer_thread: Some(demuxer_thread),
            playing,
            playing_changed_callback: playing_changed,
        })
    }

    /// Toggle between play and pause
    pub fn toggle_pause_playing(&mut self) {
        if self.playing {
            self.playing = false;
            let _ = self.control_sender.send_blocking(ControlCommand::Pause);
        } else {
            self.playing = true;
            let _ = self.control_sender.send_blocking(ControlCommand::Play);
        }
        (self.playing_changed_callback)(self.playing);
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.playing {
            self.playing = false;
            let _ = self.control_sender.send_blocking(ControlCommand::Pause);
            (self.playing_changed_callback)(self.playing);
        }
    }

    /// Resume playback
    pub fn resume(&mut self) {
        if !self.playing {
            self.playing = true;
            let _ = self.control_sender.send_blocking(ControlCommand::Play);
            (self.playing_changed_callback)(self.playing);
        }
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.control_sender.close();
        if let Some(decoder_thread) = self.demuxer_thread.take() {
            let _ = decoder_thread.join();
        }
    }
}

/// Video playback thread that handles decoding and timing
struct VideoPlaybackThread {
    control_sender: smol::channel::Sender<ControlCommand>,
    packet_sender: smol::channel::Sender<ffmpeg_next::codec::packet::packet::Packet>,
    receiver_thread: Option<std::thread::JoinHandle<()>>,
}

impl VideoPlaybackThread {
    fn start(
        stream: &ffmpeg_next::format::stream::Stream,
        mut video_frame_callback: Box<dyn FnMut(&ffmpeg_next::util::frame::Video) + Send>,
    ) -> Result<Self> {
        let (control_sender, control_receiver) = smol::channel::unbounded();
        let (packet_sender, packet_receiver) = smol::channel::bounded(128);

        let decoder_context = ffmpeg_next::codec::Context::from_parameters(stream.parameters())?;
        let mut packet_decoder = decoder_context.decoder().video()?;

        let clock = StreamClock::new(stream);

        let receiver_thread = std::thread::Builder::new()
            .name("video playback thread".into())
            .spawn(move || {
                smol::block_on(async move {
                    let packet_receiver_impl = async {
                        loop {
                            let Ok(packet) = packet_receiver.recv().await else {
                                break;
                            };

                            smol::future::yield_now().await;

                            packet_decoder.send_packet(&packet).unwrap();

                            let mut decoded_frame = ffmpeg_next::util::frame::Video::empty();

                            while packet_decoder.receive_frame(&mut decoded_frame).is_ok() {
                                if let Some(delay) =
                                    clock.convert_pts_to_instant(decoded_frame.pts())
                                {
                                    smol::Timer::after(delay).await;
                                }

                                video_frame_callback(&decoded_frame);
                            }
                        }
                    }
                    .fuse()
                    .shared();

                    let mut playing = true;

                    loop {
                        let packet_receiver: OptionFuture<_> = if playing {
                            Some(packet_receiver_impl.clone())
                        } else {
                            None
                        }
                        .into();

                        smol::pin!(packet_receiver);

                        futures::select! {
                            _ = packet_receiver => {},
                            received_command = control_receiver.recv().fuse() => {
                                match received_command {
                                    Ok(ControlCommand::Pause) => {
                                        playing = false;
                                    }
                                    Ok(ControlCommand::Play) => {
                                        playing = true;
                                    }
                                    Err(_) => {
                                        // Channel closed -> quit
                                        return;
                                    }
                                }
                            }
                        }
                    }
                })
            })?;

        Ok(Self {
            control_sender,
            packet_sender,
            receiver_thread: Some(receiver_thread),
        })
    }

    async fn receive_packet(&self, packet: ffmpeg_next::codec::packet::packet::Packet) -> bool {
        match self.packet_sender.send(packet).await {
            Ok(_) => true,
            Err(smol::channel::SendError(_)) => false,
        }
    }

    async fn send_control_message(&self, message: ControlCommand) {
        let _ = self.control_sender.send(message).await;
    }
}

impl Drop for VideoPlaybackThread {
    fn drop(&mut self) {
        self.control_sender.close();
        if let Some(receiver_join_handle) = self.receiver_thread.take() {
            let _ = receiver_join_handle.join();
        }
    }
}

/// Clock for synchronizing video playback to presentation timestamps
struct StreamClock {
    time_base_seconds: f64,
    start_time: Instant,
}

impl StreamClock {
    fn new(stream: &ffmpeg_next::format::stream::Stream) -> Self {
        let time_base = stream.time_base();
        let time_base_seconds = time_base.numerator() as f64 / time_base.denominator() as f64;

        let start_time = Instant::now();

        Self {
            time_base_seconds,
            start_time,
        }
    }

    fn convert_pts_to_instant(&self, pts: Option<i64>) -> Option<Duration> {
        pts.and_then(|pts| {
            let pts_since_start = Duration::from_secs_f64(pts as f64 * self.time_base_seconds);
            self.start_time.checked_add(pts_since_start)
        })
        .map(|absolute_pts| absolute_pts.saturating_duration_since(Instant::now()))
    }
}

// Work around https://github.com/zmwangx/rust-ffmpeg/issues/102
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Rescaler(ffmpeg_next::software::scaling::Context);

// Safety: The scaling context doesn't hold references to external mutable state
unsafe impl std::marker::Send for Rescaler {}

/// Create a rescaler to convert frames to RGB24 format for display
pub fn rgb_rescaler_for_frame(frame: &ffmpeg_next::util::frame::Video) -> Rescaler {
    Rescaler(
        ffmpeg_next::software::scaling::Context::get(
            frame.format(),
            frame.width(),
            frame.height(),
            ffmpeg_next::format::Pixel::RGB24,
            frame.width(),
            frame.height(),
            ffmpeg_next::software::scaling::Flags::BILINEAR,
        )
        .unwrap(),
    )
}

/// Convert an FFmpeg video frame to a Slint pixel buffer
pub fn video_frame_to_pixel_buffer(
    frame: &ffmpeg_next::util::frame::Video,
) -> slint::SharedPixelBuffer<slint::Rgb8Pixel> {
    let mut pixel_buffer =
        slint::SharedPixelBuffer::<slint::Rgb8Pixel>::new(frame.width(), frame.height());

    let ffmpeg_line_iter = frame.data(0).chunks_exact(frame.stride(0));
    let slint_pixel_line_iter = pixel_buffer
        .make_mut_bytes()
        .chunks_mut(frame.width() as usize * core::mem::size_of::<slint::Rgb8Pixel>());

    for (source_line, dest_line) in ffmpeg_line_iter.zip(slint_pixel_line_iter) {
        dest_line.copy_from_slice(&source_line[..dest_line.len()])
    }

    pixel_buffer
}
