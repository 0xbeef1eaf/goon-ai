interface WindowOptions {
    opacity?: number;
    position?: { x: number; y: number };
    size?: { width: number; height: number };
    alwaysOnTop?: boolean;
    clickThrough?: boolean;
    decorations?: boolean;
}

interface PromptOptions extends WindowOptions {
    text?: string; // Optional here because it might be passed as first arg
    fontSize?: number;
    color?: [number, number, number, number];
    background?: [number, number, number, number];
    padding?: number;
    maxWidth?: number;
    alignment?: "left" | "center" | "right";
    duration?: number;
}

if (!(globalThis as any).goon.prompt) (globalThis as any).goon.prompt = {};

(globalThis as any).goon.prompt.show = async function (textOrOptions: string | PromptOptions, options?: PromptOptions) {
    let finalOptions: PromptOptions;
    if (typeof textOrOptions === 'string') {
        finalOptions = { text: textOrOptions, ...options };
    } else {
        finalOptions = textOrOptions;
    }
    return await Deno.core.ops.op_show_prompt(finalOptions);
};
