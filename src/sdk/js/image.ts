interface WindowOptions {
    position?: {
        x?: number,
        y?: number,
        width?: number,
        height?: number,
    },
    alwaysOnTop?: boolean,
    opacity?: number,
}

interface ImageShowOptions extends WindowOptions {
    tags?: string[], // Additional tags to filter the images by, e.g., ["nature", "animals"], this will be applied on top of the path
    closeable?: boolean, // Optional flag to make the image window closeable by the user, defaults to false
    clickThrough?: boolean, // Optional flag to allow click-through on the image window, defaults to false
};


if (!(globalThis as any).goon.image) (globalThis as any).goon.image = {};

(globalThis as any).goon.image.show = async function (options: ImageShowOptions = {}) {
    return await Deno.core.ops.op_show_image(options);
};
