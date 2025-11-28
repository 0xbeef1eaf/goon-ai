if (!(globalThis as any).goon.image) (globalThis as any).goon.image = {};

(globalThis as any).goon.image.show = async function(path: string, options?: any) {
    return await Deno.core.ops.op_show_image(path, options);
};
