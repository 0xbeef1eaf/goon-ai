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

interface ImageShowOptions extends WindowOptions {
    tags?: string[],
    closeable?: boolean,
    clickThrough?: boolean,
}

interface PromptShowOptions {
    text: string,
    image?: ImageShowOptions,
}

if (!(globalThis as any).goon.prompt) (globalThis as any).goon.prompt = {};

(globalThis as any).goon.prompt.show = async function (options: PromptShowOptions) {
    return await Deno.core.ops.op_show_prompt(options);
};
