// @ts-nocheck

class VideoHandle {
    constructor(id: string) { this.id = id; }

    async close(): Promise<void> {
        // TODO
    }

    async setOpacity(opacity: number): Promise<void> {
        // TODO
    }

    async moveTo(x: number, y: number): Promise<void> {
        // TODO
    }

    async resize(width: number, height: number): Promise<void> {
        // TODO
    }

    async setVolume(volume: number): Promise<void> {
        // TODO
    }

    async loop(loop: boolean): Promise<void> {
        // TODO
    }
}

class video {
    static async play(options: VideoOptions): Promise<VideoHandle> {
        const id = await Deno.core.ops.op_show_video(options);
        return new VideoHandle(id);
    }
}

(globalThis as any).video = video;
(globalThis as any).goon = (globalThis as any).goon || {};
(globalThis as any).goon.video = video;
