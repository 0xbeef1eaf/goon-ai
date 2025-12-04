use crate::sdk::{audio, hypno, image, runtime_gen, types, video, wallpaper, website, write_lines};
use ts_rs::TS;

fn extract_definitions(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim().starts_with("(globalThis") && !line.contains("@ts-nocheck"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn types_ts() -> String {
    let position_decl = types::Position::decl();
    let size_decl = types::Size::decl();
    let window_options_decl = types::WindowOptions::decl();

    format!(
        r#"
/**
 * Base window handle interface
 */
interface WindowHandle {{
    /**
     * Closes the window.
     */
    close(): Promise<void>;
}}

{}

{}

{}
"#,
        position_decl, size_decl, window_options_decl
    )
}

pub fn system_ts() -> String {
    extract_definitions(&runtime_gen::generate_system_runtime())
}

pub fn pack_ts() -> String {
    extract_definitions(&runtime_gen::generate_pack_runtime())
}

pub fn image_ts() -> String {
    let options_interface = image::ImageOptions::decl();
    let source = extract_definitions(&runtime_gen::generate_image_runtime());
    format!("{}\n{}", options_interface, source)
}

pub fn video_ts() -> String {
    let options_interface = video::VideoOptions::decl();
    let source = extract_definitions(&runtime_gen::generate_video_runtime());
    format!("{}\n{}", options_interface, source)
}

pub fn audio_ts() -> String {
    let options_interface = audio::AudioOptions::decl();
    let source = extract_definitions(&runtime_gen::generate_audio_runtime());
    format!("{}\n{}", options_interface, source)
}

pub fn write_lines_ts() -> String {
    let options_interface = write_lines::WriteLinesOptions::decl();
    let source = extract_definitions(&runtime_gen::generate_write_lines_runtime());
    format!("{}\n{}", options_interface, source)
}

pub fn wallpaper_ts() -> String {
    let options_interface = wallpaper::WallpaperOptions::decl();
    let source = extract_definitions(&runtime_gen::generate_wallpaper_runtime());
    format!("{}\n{}", options_interface, source)
}

pub fn website_ts() -> String {
    let options_interface = website::WebsiteOptions::decl();
    let source = extract_definitions(&runtime_gen::generate_website_runtime());
    format!("{}\n{}", options_interface, source)
}

pub fn hypno_ts() -> String {
    let options_interface = hypno::HypnoOptions::decl();
    let source = extract_definitions(&runtime_gen::generate_hypno_runtime());
    format!("{}\n{}", options_interface, source)
}
