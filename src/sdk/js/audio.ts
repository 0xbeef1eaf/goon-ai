if (!(globalThis as any).goon.audio) (globalThis as any).goon.audio = {};

(globalThis as any).goon.audio.play = async function(path: string) {
    return await Deno.core.ops.op_play_audio(path);
};
