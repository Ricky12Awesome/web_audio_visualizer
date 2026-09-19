(() => {
    const workletUrl = new URL("audio-worklet.js", document.currentScript.src);
    const queuedBatches = [];
    let audioContext = null;
    let inputNode = null;
    let tapNode = null;
    let activeBatch = null;
    let activeBatchOffset = 0;
    let channelCount = 0;

    // Pass the stable raw AudioNode from before the playback GainNode, or pass
    // the MediaStream Electron is already using. This is a read-only side tap.
    async function setAudioStream(source) {
        if (audioContext !== null) {
            throw new Error("The audio stream has already been set");
        }

        const AudioContext = window.AudioContext || window.webkitAudioContext;
        if (AudioContext === undefined) {
            throw new Error("Web Audio is not supported by this browser");
        }

        if (source instanceof MediaStream) {
            audioContext = new AudioContext();
            inputNode = audioContext.createMediaStreamSource(source);
        } else if (source instanceof AudioNode) {
            audioContext = source.context;
            inputNode = source;
        } else {
            audioContext = null;
            throw new TypeError(
                "setAudioStream expects a MediaStream or AudioNode",
            );
        }

        await audioContext.audioWorklet.addModule(workletUrl);

        tapNode = new AudioWorkletNode(audioContext, "raw-audio-tap", {
            numberOfInputs: 1,
            numberOfOutputs: 1,
            outputChannelCount: [1],
        });
        tapNode.port.onmessage = ({ data }) => {
            const nextChannelCount = data.channels.length;
            if (nextChannelCount !== channelCount) {
                channelCount = nextChannelCount;
                queuedBatches.length = 0;
                activeBatch = null;
                activeBatchOffset = 0;
            }

            queuedBatches.push(data.channels);

            // The visualizer consumes data in real time. If rendering was paused,
            // discard stale frames instead of allowing the queue to grow forever.
            while (queuedBatches.length > 8) {
                queuedBatches.shift();
            }
        };

        inputNode.connect(tapNode);

        // Chromium only schedules worklet branches which lead to a destination.
        // RawAudioTap emits silence, so this keeps it scheduled without adding it
        // to the audible mix or changing the captured input.
        tapNode.connect(audioContext.destination);

        await audioContext.resume();
    }

    function readAudioSamples(pointer, frameCapacity) {
        if (channelCount === 0) {
            return 0;
        }

        const destination = new Float32Array(
            wasm_memory.buffer,
            pointer,
            frameCapacity * channelCount,
        );
        let framesWritten = 0;

        while (framesWritten < frameCapacity) {
            if (activeBatch === null) {
                activeBatch = queuedBatches.shift() ?? null;
                activeBatchOffset = 0;
            }

            if (activeBatch === null) {
                break;
            }

            const availableFrames = activeBatch[0].length - activeBatchOffset;
            const framesToCopy = Math.min(
                availableFrames,
                frameCapacity - framesWritten,
            );

            for (let frame = 0; frame < framesToCopy; frame += 1) {
                for (let channel = 0; channel < channelCount; channel += 1) {
                    destination[(framesWritten + frame) * channelCount + channel] =
                        activeBatch[channel][activeBatchOffset + frame];
                }
            }

            framesWritten += framesToCopy;
            activeBatchOffset += framesToCopy;

            if (activeBatchOffset === activeBatch[0].length) {
                activeBatch = null;
            }
        }

        return framesWritten;
    }

    function availableAudioFrames() {
        let frameCount = activeBatch === null
            ? 0
            : activeBatch[0].length - activeBatchOffset;
        for (const batch of queuedBatches) {
            frameCount += batch[0].length;
        }
        return frameCount;
    }

    window.setAudioStream = setAudioStream;

    miniquad_add_plugin({
        name: "web_audio",
        version: 1,
        register_plugin(importObject) {
            importObject.env.audio_available_frame_count = availableAudioFrames;
            importObject.env.audio_channel_count = () => channelCount;
            importObject.env.audio_samples = readAudioSamples;
        },
    });
})();
