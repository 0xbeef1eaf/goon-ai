if (!(globalThis as any).goon.prompt) (globalThis as any).goon.prompt = {};

(globalThis as any).goon.prompt.show = async function(text: string) {
    return await Deno.core.ops.op_show_prompt(text);
};
