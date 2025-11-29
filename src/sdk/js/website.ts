export class Website {
    /**
     * Open a website matching the provided options.
     *
     * @param options Configuration options for selecting a website
     */
    static async open(options?: WebsiteOptions): Promise<void> {
        // @ts-ignore
        await Deno.core.ops.op_open_website(options);
    }
}
