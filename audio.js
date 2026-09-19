(() => {
    let analyser = null;
    let audioContext = null;
    let inputNode = null;

    function setAudioStream(source) {
        if (analyser !== null) {
            throw new Error("The audio stream has already been set");
        }

        const AudioContext = window.AudioContext || window.webkitAudioContext;
        if (AudioContext === undefined) {
            throw new Error("Web Audio is not supported by this browser");
        }

        let input;
        let shouldConnectToSpeakers = false;

        if (source instanceof HTMLMediaElement) {
            audioContext = new AudioContext();
            input = audioContext.createMediaElementSource(source);
            shouldConnectToSpeakers = true;

            // Browsers may suspend a context until playback begins after a user gesture.
            source.addEventListener("play", () => audioContext.resume());
        } else if (source instanceof MediaStream) {
            audioContext = new AudioContext();
            input = audioContext.createMediaStreamSource(source);
        } else if (source instanceof AudioNode) {
            audioContext = source.context;
            input = source;
        } else {
            throw new TypeError(
                "setAudioStream expects an HTMLMediaElement, MediaStream, or AudioNode",
            );
        }

        analyser = audioContext.createAnalyser();
        analyser.fftSize = 1024;
        inputNode = input;
        inputNode.connect(analyser);

        // createMediaElementSource takes over the element's normal audio output.
        // MediaStreams are intentionally not routed to the speakers (avoids feedback),
        // and AudioNodes remain under the caller's routing control.
        if (shouldConnectToSpeakers) {
            analyser.connect(audioContext.destination);
        }

        audioContext.resume();
    }

    window.setAudioStream = setAudioStream;

    miniquad_add_plugin({
        name: "web_audio",
        version: 1,
        register_plugin(importObject) {
            importObject.env.audio_samples = (pointer, length) => {
                if (analyser === null) {
                    return 0;
                }

                const samples = new Float32Array(
                    wasm_memory.buffer,
                    pointer,
                    length,
                );
                analyser.getFloatTimeDomainData(samples);
                return 1;
            };
        },
    });
})();
