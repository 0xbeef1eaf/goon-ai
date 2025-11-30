class System {
    static async getAsset(tag: string): Promise<string> {
        return await Deno.core.ops.op_get_asset(tag);
    }

    static async closeWindow(handle: number): Promise<void> {
        return await Deno.core.ops.op_close_window(handle);
    }

    static log(msg: string): void {
        Deno.core.ops.op_log(msg);
    }
}

const system = System;

(globalThis as any).system = System;
(globalThis as any).goon = (globalThis as any).goon || {};
(globalThis as any).goon.system = System;
