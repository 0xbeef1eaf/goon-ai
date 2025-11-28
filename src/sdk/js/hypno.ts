if (!(globalThis as any).goon.hypno) (globalThis as any).goon.hypno = {};

(globalThis as any).goon.hypno.show = async function(pattern: string) {
    return await Deno.core.ops.op_show_hypno(pattern);
};
