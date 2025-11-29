interface AudioPlayOptions {
    tags?: string[],
    loop?: boolean,
    volume?: number,
    duration?: number,
}

type AudioHandle = string;

if (!(globalThis as any).goon.audio) (globalThis as any).goon.audio = {};

(globalThis as any).goon.audio.play = async function(options: AudioPlayOptions = {}): Promise<AudioHandle> {
    return await Deno.core.ops.op_play_audio(options);
};

(globalThis as any).goon.audio.stop = async function(handle: AudioHandle): Promise<void> {
    return await Deno.core.ops.op_stop_audio(handle);
};

(globalThis as any).goon.audio.pause = async function(handle: AudioHandle): Promise<void> {
    return await Deno.core.ops.op_pause_audio(handle);
};

(globalThis as any).goon.audio.resume = async function(handle: AudioHandle): Promise<void> {
    return await Deno.core.ops.op_resume_audio(handle);
};

(globalThis as any).goon.audio.setVolume = async function(handle: AudioHandle, volume: number): Promise<void> {
    return await Deno.core.ops.op_set_audio_volume(handle, volume);
};
