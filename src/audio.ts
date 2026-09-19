declare function miniquad_add_plugin(plugin: {
  name: string;
  version: number;
  register_plugin: (importObject: Record<string, unknown>) => void;
}): void;

declare const wasm_memory: WebAssembly.Memory;




miniquad_add_plugin({
  name: "web_audio",
  version: 1,
  register_plugin(importObject) {
    // const env = importObject.env as Record<string, unknown>;

    // env.audio_available_frame_count = availableAudioFrames;
    // env.audio_channel_count = () => channelCount;
    // env.audio_samples = readAudioSamples;
  },
});
