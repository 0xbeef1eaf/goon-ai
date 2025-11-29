use crate::sdk::{audio, image, prompt, types, video, wallpaper, website};
use ts_rs::TS;

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

pub fn pack_ts() -> String {
    r#"
/**
 * Mood information
 */
interface Mood {
    /** Name of the mood */
    name: string;

    /** Description of the mood */
    description: string;

    /** Tags associated with the mood */
    tags: string[];
}
/**
 * Pack and mood management functions
 * Note: Moods are included in the LLM prompt for context
 */
class pack {
    /**
     * Gets the current mood for the pack.
     * This determines which assets are selected by default.
     *
     * @returns The current mood
     *
     * @example
     * const currentMood = await pack.getCurrentMood();
     * console.log(`Current mood: ${currentMood.name}`);
     * console.log(`Tags: ${currentMood.tags.join(', ')}`);
     */
    static async getCurrentMood(): Promise<Mood>;

    /**
     * Sets the current mood for the pack.
     * This will affect which assets are selected for all subsequent operations.
     *
     * @param mood_name Name of the mood to activate
     *
     * @example
     * // Switch to relaxation mode
     * await pack.setMood("Nature");
     *
     * // Now all asset selection will use "Nature" mood tags
     * await image.show({}); // Will show nature-themed images
     */
    static async setMood(mood_name: string): Promise<void>;
}
"#
    .to_string()
}

pub fn image_ts() -> String {
    let options_interface = image::ImageOptions::decl();
    format!(
        r#"
{}
interface ImageHandle extends WindowHandle {{
    /**
     * Sets the opacity of the image window, from 0 (transparent) to 1 (opaque).
     */
    setOpacity(opacity: number): Promise<void>;

    /**
     * Moves the image window to the specified (x, y) coordinates.
     */
    moveTo(x: number, y: number): Promise<void>;

    /**
     * Resizes the image window to the specified width and height.
     * Maintains aspect ratio if needed.
     */
    resize(width: number, height: number): Promise<void>;
}}
/**
 * Image display functions
 */
class image {{
    /**
     * Display an image from the pack's assets.
     * The image is automatically selected based on current mood and optional tags.
     */
    static async show(options: ImageOptions): Promise<ImageHandle>;
}}
"#,
        options_interface
    )
}

pub fn video_ts() -> String {
    let options_interface = video::VideoOptions::decl();
    format!(
        r#"
{}
interface VideoHandle extends WindowHandle {{
    setOpacity(opacity: number): Promise<void>;
    moveTo(x: number, y: number): Promise<void>;
    resize(width: number, height: number): Promise<void>;
    setVolume(volume: number): Promise<void>;
    loop(loop: boolean): Promise<void>;
}}
class video {{
    static async play(options: VideoOptions): Promise<VideoHandle>;
}}
"#,
        options_interface
    )
}

pub fn audio_ts() -> String {
    let options_interface = audio::AudioOptions::decl();
    format!(
        r#"
{}
interface AudioHandle {{
    stop(): Promise<void>;
    setVolume(volume: number): Promise<void>;
    loop(loop: boolean): Promise<void>;
}}
class audio {{
    static async play(options: AudioOptions): Promise<AudioHandle>;
}}
"#,
        options_interface
    )
}

pub fn prompt_ts() -> String {
    let options_interface = prompt::PromptOptions::decl();
    format!(
        r#"
/**
 * Text prompt with optional image display
 */
{}
class textPrompt {{
    /**
     * Displays a text prompt in a window, with optional image.
     * The window will close when the user has copied the text into the prompt window.
     *
     * Note: This ignores the following ImageOptions properties:
     * - position: Always centers the window on screen
     * - timeout: Window remains until user copies text
     * - closable: Window only closes when user copies text
     *
     * The image will be displayed below the text if provided.
     *
     * @param text The text to display
     * @param image Optional image options (position, timeout, closable ignored)
     * @returns Window handle
     *
     * @example
     * // Simple text prompt
     * await textPrompt.show("Take a deep breath and relax...");
     *
     * @example
     * // Text prompt with mood-based image
     * await textPrompt.show(
     *   "Focus on this image and breathe slowly...",
     *   {{ tags: ['calming', 'nature'], opacity: 0.9 }}
     * );
     */
    static async show(options: PromptOptions): Promise<WindowHandle>;
}}
"#,
        options_interface
    )
}

pub fn wallpaper_ts() -> String {
    let options_interface = wallpaper::WallpaperOptions::decl();
    format!(
        r#"
{}
/**
 * Desktop wallpaper functions
 */
class wallpaper {{
    /**
     * Sets the desktop wallpaper to an image matching the specified mood and tags.
     * The image is automatically selected based on:
     * 1. Current mood's tags
     * 2. Additional tags specified (if any)
     *
     * @param tags Optional additional tags to filter wallpapers
     *
     * @example
     * // Set wallpaper from current mood
     * await wallpaper.set();
     *
     * @example
     * // Set wallpaper with specific tags
     * await wallpaper.set(['mountain', 'sunset']);
     */
    static async set(options: WallpaperOptions): Promise<void>;
}}
"#,
        options_interface
    )
}

pub fn website_ts() -> String {
    let options_interface = website::WebsiteOptions::decl();
    format!(
        r#"
{}
/**
 * Web browser functions
 */
class website {{
    /**
     * Opens a website in the default browser matching the specified mood and tags.
     * Websites are defined in the pack configuration and filtered by:
     * 1. Current mood's tags
     * 2. Additional tags specified (if any)
     *
     * @param tags Optional tags to filter websites
     *
     * @example
     * // Open website from current mood
     * await website.open();
     *
     * @example
     * // Open conservation-related website
     * await website.open(['conservation', 'nature']);
     */
    static async open(options: WebsiteOptions): Promise<void>;
}}
"#,
        options_interface
    )
}
