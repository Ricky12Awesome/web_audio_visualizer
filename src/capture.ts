/// <reference types="@types/audioworklet" />

class Capture extends AudioWorkletProcessor {
    process(inputs: Float32Array[][]) {
        const input = inputs[0];

        if (input.length > 0) {
            const channels = input.map((channel) => new Float32Array(channel));
            this.port.postMessage(channels, channels.map((c) => c.buffer));
        }

        return true;
    }
}

registerProcessor("capture", Capture);