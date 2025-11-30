// @ts-nocheck

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
     * @param options Optional additional tags to filter wallpapers
     */
    static async set(options: WallpaperOptions): Promise<void> {
        await Deno.core.ops.op_set_wallpaper(options);
    }
}

(globalThis as any).wallpaper = wallpaper;
(globalThis as any).goon = (globalThis as any).goon || {};
(globalThis as any).goon.wallpaper = wallpaper;
