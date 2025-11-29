interface AudioPlayOptions {
    tags?: string[],
    loop?: boolean,
    volume?: number,
}

if (!(globalThis as any).goon.audio) (globalThis as any).goon.audio = {};

(globalThis as any).goon.audio.play = async function(options: AudioPlayOptions = {}) {
    return await Deno.core.ops.op_play_audio(options);
};
