interface Mood {
    name: string;
    description: string;
    tags: string[];
}

class Pack {
    static async getCurrentMood(): Promise<Mood> {
        return await Deno.core.ops.op_get_current_mood();
    }

    static async setMood(mood_name: string): Promise<void> {
        return await Deno.core.ops.op_set_current_mood(mood_name);
    }
}

(globalThis as any).pack = Pack;
(globalThis as any).goon = (globalThis as any).goon || {};
(globalThis as any).goon.pack = Pack;
