// @ts-nocheck

/**
 * Text prompt with optional image display
 */
class textPrompt {
    /**
     * Displays a text prompt in a window, with optional image.
     * The window will close when the user has copied the text into the prompt window.
     *
     * Note: This ignores the following ImageOptions properties:
     * - position: Always centers the window on screen
     * - timeout: Window remains until user copies text
     * - closable: Window only closes when user copies text
     *
     * The image will be displayed below the text if provided.
     *
     * @param options The prompt options
     * @returns Window handle
     */
    static async show(options: PromptOptions): Promise<WindowHandle> {
        const id = await Deno.core.ops.op_show_prompt(options);
        return {
            close: async () => {
                // TODO
            }
        };
    }
}

(globalThis as any).textPrompt = textPrompt;
(globalThis as any).goon = (globalThis as any).goon || {};
(globalThis as any).goon.textPrompt = textPrompt;
