use macroquad::prelude::*;

const SAMPLE_COUNT: usize = 1024;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn audio_samples(pointer: *mut f32, length: usize) -> i32;
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn web_audio_crate_version() -> u32 {
    1
}

fn update_audio_samples(samples: &mut [f32]) -> bool {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: JavaScript receives a pointer to this live slice and writes at most
    // `samples.len()` f32 values before returning synchronously.
    unsafe {
        audio_samples(samples.as_mut_ptr(), samples.len()) != 0
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = samples;
        false
    }
}

#[macroquad::main("Web Audio Visualizer")]
async fn main() {
    let mut samples = vec![0.0; SAMPLE_COUNT];

    loop {
        clear_background(BLACK);

        if update_audio_samples(&mut samples) {
            let width = screen_width();
            let height = screen_height();
            let center_y = height * 0.5;
            let x_step = width / (samples.len() - 1) as f32;

            for (index, pair) in samples.windows(2).enumerate() {
                draw_line(
                    index as f32 * x_step,
                    center_y + pair[0] * center_y,
                    (index + 1) as f32 * x_step,
                    center_y + pair[1] * center_y,
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
