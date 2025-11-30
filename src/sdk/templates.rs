use crate::sdk::{audio, hypno, image, pack, prompt, system, types, video, wallpaper, website};
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
    extract_definitions(system::TS_SOURCE)
}

pub fn pack_ts() -> String {
    extract_definitions(pack::TS_SOURCE)
}

pub fn image_ts() -> String {
    let options_interface = image::ImageOptions::decl();
    let source = extract_definitions(image::TS_SOURCE);
    format!("{}\n{}", options_interface, source)
}

pub fn video_ts() -> String {
    let options_interface = video::VideoOptions::decl();
    let source = extract_definitions(video::TS_SOURCE);
    format!("{}\n{}", options_interface, source)
}

pub fn audio_ts() -> String {
    let options_interface = audio::AudioOptions::decl();
    let source = extract_definitions(audio::TS_SOURCE);
    format!("{}\n{}", options_interface, source)
}

pub fn prompt_ts() -> String {
    let options_interface = prompt::PromptOptions::decl();
    let source = extract_definitions(prompt::TS_SOURCE);
    format!("{}\n{}", options_interface, source)
}

pub fn wallpaper_ts() -> String {
    let options_interface = wallpaper::WallpaperOptions::decl();
    let source = extract_definitions(&wallpaper::get_source());
    format!("{}\n{}", options_interface, source)
}

pub fn website_ts() -> String {
    let options_interface = website::WebsiteOptions::decl();
    let source = extract_definitions(&website::get_source());
    format!("{}\n{}", options_interface, source)
}

pub fn hypno_ts() -> String {
    let options_interface = hypno::HypnoOptions::decl();
    let source = extract_definitions(hypno::TS_SOURCE);
    format!("{}\n{}", options_interface, source)
}
