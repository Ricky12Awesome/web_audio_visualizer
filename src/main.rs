use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
const SAMPLE_COUNT: usize = 16384;

#[derive(Serialize, Deserialize)]
pub struct Config {
    enabled: bool,
    // more settings will be added in future
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn audio_available_frame_count() -> usize;
    fn audio_channel_count() -> usize;
    fn audio_samples(pointer: *mut f32, frame_capacity: usize) -> usize;
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn web_audio_crate_version() -> u32 {
    1
}

fn update_audio_samples(samples: &mut Vec<f32>, channels: &mut usize) -> usize {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: JavaScript receives a pointer to this live allocation and writes at
    // most SAMPLE_COUNT * channel_count f32 values before returning synchronously.
    unsafe {
        if audio_available_frame_count() == 0 {
            return 0;
        }

        *channels = audio_channel_count();
        if *channels == 0 {
            return 0;
        }

        samples.resize(SAMPLE_COUNT * *channels, 0.0);
        let frame_count = audio_samples(samples.as_mut_ptr(), SAMPLE_COUNT);
        samples.truncate(frame_count * *channels);
        frame_count
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = samples;
        let _ = channels;
        0
    }
}

#[macroquad::main("Web Audio Visualizer")]
async fn main() {
    let mut samples = Vec::new();
    let mut channel_count = 0;
    let mut displayed_frame_count = 0;

    loop {
        clear_background(Color::from_rgba(0, 0, 0, 0));


        draw_text(samples.len().to_string(), 0.0, 64.0, 48.0, WHITE);

        let new_frame_count = update_audio_samples(&mut samples, &mut channel_count);
        if new_frame_count > 0 {
            displayed_frame_count = new_frame_count;
        }

        if displayed_frame_count >= 2 {
            let width = screen_width();
            let height = screen_height();
            let center_y = height * 0.5;
            let x_step = width / (displayed_frame_count - 1) as f32;

            for index in 0..displayed_frame_count - 1 {
                let current_sample = samples[index * channel_count];
                let next_sample = samples[(index + 1) * channel_count];
                draw_line(
                    index as f32 * x_step,
                    center_y + current_sample * center_y,
                    (index + 1) as f32 * x_step,
                    center_y + next_sample * center_y,
                    2.0,
                    GREEN,
                );
            }
        } else {
            draw_text("Waiting for an audio stream", 20.0, 40.0, 30.0, WHITE);
        }

        next_frame().await
    }
}
