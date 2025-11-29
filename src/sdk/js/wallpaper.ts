interface WallpaperSetOptions {
    tags?: string[],
}

if (!(globalThis as any).goon.wallpaper) (globalThis as any).goon.wallpaper = {};

(globalThis as any).goon.wallpaper.set = async function(options: WallpaperSetOptions = {}) {
    return await Deno.core.ops.op_set_wallpaper(options);
};
