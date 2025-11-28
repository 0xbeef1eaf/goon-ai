if (!(globalThis as any).goon.website) (globalThis as any).goon.website = {};

(globalThis as any).goon.website.open = async function(url: string) {
    return await Deno.core.ops.op_open_website(url);
};
