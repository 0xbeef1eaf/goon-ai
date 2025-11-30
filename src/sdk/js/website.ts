// @ts-nocheck

/**
 * Web browser functions
 */
class website {
    /**
     * Open a website matching the provided options.
     *
     * @param options Configuration options for selecting a website
     */
    static async open(options: WebsiteOptions): Promise<void> {
        await Deno.core.ops.op_open_website(options);
    }
}

(globalThis as any).website = website;
(globalThis as any).goon = (globalThis as any).goon || {};
(globalThis as any).goon.website = website;
