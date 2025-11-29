interface WebsiteOpenOptions {
    url: string,
}

if (!(globalThis as any).goon.website) (globalThis as any).goon.website = {};

(globalThis as any).goon.website.open = async function(options: WebsiteOpenOptions) {
    return await Deno.core.ops.op_open_website(options);
};
