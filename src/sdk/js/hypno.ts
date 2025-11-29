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

interface HypnoShowOptions extends WindowOptions {
    tags?: string[],
    duration?: number,
    closeable?: boolean,
    clickThrough?: boolean,
}

if (!(globalThis as any).goon.hypno) (globalThis as any).goon.hypno = {};

(globalThis as any).goon.hypno.show = async function(options: HypnoShowOptions = {}) {
    return await Deno.core.ops.op_show_hypno(options);
};
