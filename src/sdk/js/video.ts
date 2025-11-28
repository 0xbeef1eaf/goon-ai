if (!(globalThis as any).goon.video) (globalThis as any).goon.video = {};

(globalThis as any).goon.video.show = async function(path: string, options?: any) {
    return await Deno.core.ops.op_show_video(path, options);
};
