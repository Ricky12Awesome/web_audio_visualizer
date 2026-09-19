declare function miniquad_add_plugin(plugin: { name: string; version: number; register_plugin: (imports: Record<string, unknown>) => void; on_init?: () => void }): void;
declare const wasm_memory: WebAssembly.Memory;
declare const wasm_exports: { audio_buffer_init(channels: number, frames: number): number; audio_buffer_state(): number; audio_buffer_samples(): number };

type CaptureMessage = { type: "ready" } | { type: "format"; channelCount: number; frames: number; quantumFrames: number };

let accumulationBlocks = 16;

let captureNode: AudioWorkletNode | undefined;

async function startCapture(): Promise<void> {
    const element = document.querySelector<HTMLAudioElement>("#audio");
    if (element === null) throw new Error("Audio element #audio was not found");
    if (!(wasm_memory.buffer instanceof SharedArrayBuffer)) {
        throw new Error("WASM must be built with shared memory and served cross-origin isolated");
    }
    const audioContext = new AudioContext();
    await audioContext.audioWorklet.addModule(new URL("./capture.ts", import.meta.url));
    const source = audioContext.createMediaElementSource(element);
    captureNode = new AudioWorkletNode(audioContext, "capture", {
        channelCountMode: "explicit",
        channelCount: audioContext.destination.channelCount,
        processorOptions: {
            sharedBuffer: wasm_memory.buffer,
            stateAddress: 0,
            samplesAddress: 0,
            blocks: accumulationBlocks,
        },
    });
    captureNode.port.onmessage = (event: MessageEvent<CaptureMessage>) => {
        if (event.data.type !== "format") return;
        const samplesAddress = wasm_exports.audio_buffer_init(event.data.channelCount, event.data.frames);
        captureNode?.port.postMessage({
            sharedBuffer: wasm_memory.buffer,
            stateAddress: wasm_exports.audio_buffer_state(),
            samplesAddress,
        });
    };
    source.connect(captureNode);
    captureNode.connect(audioContext.destination);
    await audioContext.resume();
}

miniquad_add_plugin({
    name: "web_audio",
    version: 1,
    register_plugin(imports) {
        (imports.env as Record<string, unknown>).get_buffer = () => wasm_exports.audio_buffer_samples();
    },
    on_init() {
        void startCapture().catch((error: unknown) => console.error("Unable to start raw audio capture", error));
    },
});
