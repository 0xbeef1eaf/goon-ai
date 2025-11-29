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

interface VideoShowOptions extends WindowOptions {
    tags?: string[],
    loop?: boolean,
    volume?: number,
    autoplay?: boolean,
    closeable?: boolean,
    clickThrough?: boolean,
}

if (!(globalThis as any).goon.video) (globalThis as any).goon.video = {};

(globalThis as any).goon.video.show = async function(options: VideoShowOptions = {}) {
    return await Deno.core.ops.op_show_video(options);
};
