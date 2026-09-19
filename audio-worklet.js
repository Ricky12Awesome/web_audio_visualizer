const BATCH_FRAME_COUNT = 1024;

class RawAudioTap extends AudioWorkletProcessor {
    constructor() {
        super();
        this.channelBuffers = null;
        this.writeOffset = 0;
    }

    process(inputs, outputs) {
        for (const output of outputs[0]) {
            output.fill(0);
        }

        const channels = inputs[0];
        if (channels.length === 0 || channels[0].length === 0) {
            return true;
        }

        if (
            this.channelBuffers === null ||
            this.channelBuffers.length !== channels.length
        ) {
            this.channelBuffers = Array.from(
                { length: channels.length },
                () => new Float32Array(BATCH_FRAME_COUNT),
            );
            this.writeOffset = 0;
        }

        let readOffset = 0;
        while (readOffset < channels[0].length) {
            const frameCount = Math.min(
                channels[0].length - readOffset,
                BATCH_FRAME_COUNT - this.writeOffset,
            );

            for (let channel = 0; channel < channels.length; channel += 1) {
                this.channelBuffers[channel].set(
                    channels[channel].subarray(
                        readOffset,
                        readOffset + frameCount,
                    ),
                    this.writeOffset,
                );
            }

            readOffset += frameCount;
            this.writeOffset += frameCount;

            if (this.writeOffset === BATCH_FRAME_COUNT) {
                const completedBuffers = this.channelBuffers;
                this.channelBuffers = null;
                this.writeOffset = 0;
                this.port.postMessage({ channels: completedBuffers });
            }
        }

        return true;
    }
}

registerProcessor("raw-audio-tap", RawAudioTap);
