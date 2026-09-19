/// <reference types="@types/audioworklet" />

type CaptureOptions = { sharedBuffer: SharedArrayBuffer; stateAddress: number; samplesAddress: number };

class Capture extends AudioWorkletProcessor {
    private readonly sharedBuffer: SharedArrayBuffer;
    private state: Int32Array;
    private samples: Float32Array;
    private configured = false;
    private formatRequested = false;

    constructor(options?: AudioWorkletNodeOptions) {
        super();
        const capture = options?.processorOptions as CaptureOptions | undefined;
        if (capture === undefined) throw new Error("Audio capture was not configured");
        this.sharedBuffer = capture.sharedBuffer;
        this.state = new Int32Array(this.sharedBuffer, capture.stateAddress, 4);
        this.samples = new Float32Array(this.sharedBuffer, capture.samplesAddress);
        this.port.onmessage = (event: MessageEvent<CaptureOptions>) => {
            this.state = new Int32Array(this.sharedBuffer, event.data.stateAddress, 4);
            this.samples = new Float32Array(this.sharedBuffer, event.data.samplesAddress);
            this.configured = true;
            this.formatRequested = false;
        };
        this.port.postMessage({ type: "ready" });
    }

    process(inputs: Float32Array[][]) {
        const channels = inputs[0];
        const frames = channels?.[0]?.length ?? 0;
        const channelCount = channels?.length ?? 0;
        if (frames === 0 || channelCount === 0) return true;

        if (!this.configured) {
            if (!this.formatRequested) {
                this.formatRequested = true;
                this.port.postMessage({ type: "format", channelCount, frames });
            }
            return true;
        }
        if (Atomics.load(this.state, 2) !== channelCount || Atomics.load(this.state, 3) !== frames) {
            this.configured = false;
            this.formatRequested = false;
            return true;
        }

        const sequence = Atomics.load(this.state, 0);
        Atomics.store(this.state, 0, sequence + 1);
        for (let channel = 0; channel < channelCount; channel += 1) {
            this.samples.set(channels[channel], channel * frames);
        }
        Atomics.store(this.state, 1, frames);
        Atomics.store(this.state, 0, sequence + 2);
        return true;
    }
}

registerProcessor("capture", Capture);
