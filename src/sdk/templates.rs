pub const TYPES_TS: &str = r#"
/**
 * Base window handle interface
 */
interface WindowHandle {
    /**
     * Closes the window.
     */
    close(): Promise<void>;
}
/**
 * Position and dimensions for window placement
 */
interface Position {
    /** X coordinate in pixels */
    x: number;
    /** Y coordinate in pixels */
    y: number;
    /** Width in pixels */
    width: number;
    /** Height in pixels */
    height: number;
}
"#;

pub const PACK_TS: &str = r#"
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
"#;

pub const IMAGE_TS: &str = r#"
interface ImageOptions {
    /**
     * Additional tags to filter images by, beyond the mood's tags.
     * Images must match the mood tags AND any specified tags.
     */
    tags?: string[];

    /**
     * Coordinates and dimensions for displaying the image.
     * All values are in pixels.
     * If omitted, defaults to the image with its original dimensions randomly placed on the screen.
     * Will be clamped to fit within the screen bounds.
     * Will maintain aspect ratio of the image, by adjusting width or height as necessary.
     */
    position?: {
        x: number;
        y: number;
        width: number;
        height: number;
    };

    /**
     * Whether the window can be closed by the user.
     * Default: true
     */
    closable?: boolean;

    /**
     * Opacity of the window, from 0 (transparent) to 1 (opaque).
     * Default: 1
     */
    opacity?: number;

    /**
     * Time in milliseconds before the window automatically closes.
     * Default: no timeout
     */
    timeout?: number;
}
interface ImageHandle extends WindowHandle {
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
}
/**
 * Image display functions
 */
class image {
    /**
     * Display an image from the pack's assets.
     * The image is automatically selected based on current mood and optional tags.
     */
    static async show(options: ImageOptions): Promise<ImageHandle>;
}
"#;

pub const VIDEO_TS: &str = r#"
interface VideoOptions {
    /**
     * Additional tags to filter videos by, beyond the mood's tags.
     */
    tags?: string[];

    /**
     * Coordinates and dimensions for displaying the video.
     */
    position?: {
        x: number;
        y: number;
        width: number;
        height: number;
    };

    /**
     * Whether the window can be closed by the user.
     * Default: true
     */
    closable?: boolean;

    /**
     * Opacity of the window, from 0 (transparent) to 1 (opaque).
     * Default: 1
     */
    opacity?: number;

    /**
     * Time in milliseconds before the window automatically closes.
     * Default: no timeout
     */
    timeout?: number;

    /**
     * Volume level from 0 (muted) to 1 (maximum).
     * Default: 1
     */
    volume?: number;

    /**
     * Whether to loop the video.
     * Default: false
     */
    loop?: boolean;
}
interface VideoHandle extends WindowHandle {
    setOpacity(opacity: number): Promise<void>;
    moveTo(x: number, y: number): Promise<void>;
    resize(width: number, height: number): Promise<void>;
    setVolume(volume: number): Promise<void>;
    loop(loop: boolean): Promise<void>;
}
class video {
    static async play(options: VideoOptions): Promise<VideoHandle>;
}
"#;

pub const AUDIO_TS: &str = r#"
interface AudioOptions {
    /**
     * Additional tags to filter audio by, beyond the mood's tags.
     */
    tags?: string[];

    /**
     * Volume level from 0 (muted) to 1 (maximum).
     * Default: 1
     */
    volume?: number;

    /**
     * Whether to loop the audio.
     * Default: false
     */
    loop?: boolean;
}
interface AudioHandle {
    stop(): Promise<void>;
    setVolume(volume: number): Promise<void>;
    loop(loop: boolean): Promise<void>;
}
class audio {
    static async play(options: AudioOptions): Promise<AudioHandle>;
}
"#;

pub const PROMPT_TS: &str = r#"
/**
 * Text prompt with optional image display
 */
class textPrompt {
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
     *   { tags: ['calming', 'nature'], opacity: 0.9 }
     * );
     */
    static async show(text: string, image?: ImageOptions): Promise<WindowHandle>;
}
"#;

pub const WALLPAPER_TS: &str = r#"
/**
 * Desktop wallpaper functions
 */
class wallpaper {
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
    static async set(tags?: string[]): Promise<void>;
}
"#;

pub const WEBSITE_TS: &str = r#"
/**
 * Web browser functions
 */
class website {
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
    static async open(tags?: string[]): Promise<void>;
}
"#;
