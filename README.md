# Web Audio Visualizer

Rust-backed audio visualizer for web-based technologies

I'm making this for use in [OsuMediaPlayer](https://github.com/Ricky12Awesome/OsuMediaPlayer)

Audio capture accumulation is configured with the `accumulationBlocks` variable in
`src/audio.ts`. It controls how many render quanta are published per shared-memory window.

Bar retention is configured with `BAR_RETENTION_SECONDS` in `src/main.rs`. A value of
`0.0` disables retention; larger values make bars take longer to settle after a peak.
