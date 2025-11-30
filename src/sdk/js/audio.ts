// @ts-nocheck

class AudioHandle {
    constructor(id: string) { this.id = id; }

    async stop(): Promise<void> {
        // TODO
    }

    async setVolume(volume: number): Promise<void> {
        // TODO
    }

    async loop(loop: boolean): Promise<void> {
        // TODO
    }
}

class audio {
    static async play(options: AudioOptions): Promise<AudioHandle> {
        const id = await Deno.core.ops.op_play_audio(options);
        return new AudioHandle(id);
    }
}

(globalThis as any).audio = audio;
(globalThis as any).goon = (globalThis as any).goon || {};
(globalThis as any).goon.audio = audio;
