/// <reference types="@types/audioworklet" />

type CaptureOptions = {
    sharedBuffer: SharedArrayBuffer;
    stateAddress: number;
    samplesAddress: number;
    blocks: number;
};

class Capture extends AudioWorkletProcessor {
    private sharedBuffer: SharedArrayBuffer;
    private state: Int32Array;
    private samples: Float32Array;
    private configured = false;
    private formatRequested = false;
    private readonly blocks: number;
    private accumulator?: Float32Array;
    private accumulatedFrames = 0;
    private quantumFrames = 0;
    private windowFrames = 0;

    constructor(options?: AudioWorkletNodeOptions) {
        super();
        const capture = options?.processorOptions as CaptureOptions | undefined;
        if (capture === undefined) throw new Error("Audio capture was not configured");
        if (!Number.isInteger(capture.blocks) || capture.blocks < 1) {
            throw new Error("Audio accumulation block count must be a positive integer");
        }
        this.sharedBuffer = capture.sharedBuffer;
        this.blocks = capture.blocks;
        this.state = new Int32Array(this.sharedBuffer, capture.stateAddress, 4);
        this.samples = new Float32Array(this.sharedBuffer, capture.samplesAddress);
        this.port.onmessage = (event: MessageEvent<CaptureOptions>) => {
            this.sharedBuffer = event.data.sharedBuffer;
            this.state = new Int32Array(this.sharedBuffer, event.data.stateAddress, 4);
            this.samples = new Float32Array(this.sharedBuffer, event.data.samplesAddress);
            this.configured = true;
            this.formatRequested = false;
        };
        this.port.postMessage({ type: "ready" });
    }

    process(inputs: Float32Array[][], outputs: Float32Array[][]) {
        const channels = inputs[0];
        const output = outputs[0];
        const frames = channels?.[0]?.length ?? 0;
        const channelCount = channels?.length ?? 0;

        for (let channel = 0; channel < output.length; channel += 1) {
            const input = channels?.[channel];
            if (input === undefined) output[channel].fill(0);
            else output[channel].set(input);
        }

        if (frames === 0 || channelCount === 0) return true;

        if (!this.configured) {
            if (!this.formatRequested) {
                this.formatRequested = true;
                this.quantumFrames = frames;
                this.windowFrames = frames * this.blocks;
                this.accumulator = new Float32Array(channelCount * this.windowFrames);
                this.accumulatedFrames = 0;
                this.port.postMessage({ type: "format", channelCount, frames: this.windowFrames, quantumFrames: frames });
            }
            return true;
        }
        if (Atomics.load(this.state, 2) !== channelCount || Atomics.load(this.state, 3) !== this.windowFrames || frames !== this.quantumFrames) {
            this.configured = false;
            this.formatRequested = false;
            return true;
        }

        const accumulator = this.accumulator;
        if (accumulator === undefined) return true;
        for (let channel = 0; channel < channelCount; channel += 1) {
            accumulator.set(channels[channel], channel * this.windowFrames + this.accumulatedFrames);
        }
        this.accumulatedFrames += frames;

        if (this.accumulatedFrames === this.windowFrames) {
            const sequence = Atomics.load(this.state, 0);
            Atomics.store(this.state, 0, sequence + 1);
            this.samples.set(accumulator);
            Atomics.store(this.state, 1, this.windowFrames);
            Atomics.store(this.state, 0, sequence + 2);
            this.accumulatedFrames = 0;
        }
        return true;
    }
}

registerProcessor("capture", Capture);
