// @ts-nocheck

/**
 * Handle to a window displaying an image.
 */
class ImageHandle {
    constructor(id: string) { this.id = id; }

    /**
     * Closes the window.
     */
    async close(): Promise<void> {
        await Deno.core.ops.op_close_window(this.id);
    }

    /**
     * Sets the opacity of the image window, from 0 (transparent) to 1 (opaque).
     */
    async setOpacity(opacity: number): Promise<void> {
        // TODO: Implement op
    }

    /**
     * Moves the image window to the specified (x, y) coordinates.
     */
    async moveTo(x: number, y: number): Promise<void> {
        // TODO: Implement op
    }

    /**
     * Resizes the image window to the specified width and height.
     * Maintains aspect ratio if needed.
     */
    async resize(width: number, height: number): Promise<void> {
        // TODO: Implement op
    }
}

/**
 * Image display functions
 */
class image {
    /**
     * Display an image from the pack's assets.
     * The image is automatically selected based on current mood and optional tags.
     */
    static async show(options: ImageOptions): Promise<ImageHandle> {
        const id = await Deno.core.ops.op_show_image(options);
        return new ImageHandle(id);
    }
}

(globalThis as any).image = image;
(globalThis as any).goon = (globalThis as any).goon || {};
(globalThis as any).goon.image = image;
