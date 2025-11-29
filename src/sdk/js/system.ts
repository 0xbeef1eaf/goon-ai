if (!(globalThis as any).goon.system) (globalThis as any).goon.system = {};

(globalThis as any).goon.system.getAsset = async function (tag: string) {
    return await Deno.core.ops.op_get_asset(tag);
};
(globalThis as any).goon.system.closeWindow = async function (handle: number) {
    return await Deno.core.ops.op_close_window(handle);
};
(globalThis as any).goon.system.log = function (msg: string) {
    Deno.core.ops.op_log(msg);
};


interface WindowOptions {
    position?: {
        x?: number, // Optional x coordinate of the window, defaults to a random position
        y?: number, // Optional y coordinate of the window, defaults to a random position
        // If both x and y are provided, the window will be positioned at that exact location
        width?: number, // Optional width of the window, defaults to auto size to the content, maintains aspect ratio
        height?: number, // Optional height of the window, defaults to auto size to the content, maintains aspect ratio
        // If both width and height are provided, the window will adjust to fit within those dimensions while maintaining aspect ratio
    },
    alwaysOnTop?: boolean, // Optional flag to keep the window always on top of other windows, defaults to true
    opacity?: number, // Optional opacity level of the window (0.0 to 1.0), defaults to 1.0 (fully opaque)
}