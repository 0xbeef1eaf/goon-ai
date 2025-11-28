if (!(globalThis as any).goon.system) (globalThis as any).goon.system = {};

(globalThis as any).goon.system.getAsset = async function(tag: string) {
    return await Deno.core.ops.op_get_asset(tag);
};
(globalThis as any).goon.system.closeWindow = async function(handle: number) {
    return await Deno.core.ops.op_close_window(handle);
};
(globalThis as any).goon.system.log = function(msg: string) {
    Deno.core.ops.op_log(msg);
};
