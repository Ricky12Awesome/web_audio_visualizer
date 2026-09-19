# Web Audio Visualizer

Rust-backed audio visualizer for web-based technologies

I'm making this for use in [OsuMediaPlayer](https://github.com/Ricky12Awesome/OsuMediaPlayer)

Audio capture accumulation is configured with the `accumulationBlocks` variable in
`src/audio.ts`. It controls how many render quanta are published per shared-memory window.
