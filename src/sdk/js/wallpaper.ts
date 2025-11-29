if (!(globalThis as any).goon.wallpaper) (globalThis as any).goon.wallpaper = {};

(globalThis as any).goon.wallpaper.set = async function(options: WallpaperOptions = {}) {
    return await Deno.core.ops.op_set_wallpaper(options);
};
