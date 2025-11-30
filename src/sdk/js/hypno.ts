// @ts-nocheck

class hypno {
    static async show(options: HypnoShowOptions): Promise<void> {
        await Deno.core.ops.op_show_hypno(options);
    }
}

(globalThis as any).hypno = hypno;
(globalThis as any).goon = (globalThis as any).goon || {};
(globalThis as any).goon.hypno = hypno;
