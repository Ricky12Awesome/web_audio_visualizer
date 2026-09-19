use macroquad::prelude::*;

const SAMPLE_COUNT: usize = 1024;
const NORMALIZED_PEAK: f32 = 0.9;
const SILENCE_THRESHOLD: f32 = 0.0001;

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

fn normalize_audio_samples(samples: &mut [f32]) {
    let peak = samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f32, f32::max);

    if peak < SILENCE_THRESHOLD {
        samples.fill(0.0);
        return;
    }

    let gain = NORMALIZED_PEAK / peak;
    for sample in samples {
        *sample *= gain;
    }
}

#[macroquad::main("Web Audio Visualizer")]
async fn main() {
    let mut samples = vec![0.0; SAMPLE_COUNT];

    loop {
        clear_background(BLACK);

        if update_audio_samples(&mut samples) {
            normalize_audio_samples(&mut samples);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_does_not_change_normalized_waveform() {
        let mut quiet = [-0.1, -0.05, 0.0, 0.05, 0.1];
        let mut loud = [-1.0, -0.5, 0.0, 0.5, 1.0];

        normalize_audio_samples(&mut quiet);
        normalize_audio_samples(&mut loud);

        for (quiet_sample, loud_sample) in quiet.iter().zip(loud) {
            assert!((quiet_sample - loud_sample).abs() < 0.000001);
        }
    }

    #[test]
    fn near_silence_is_not_amplified() {
        let mut samples = [0.00001, -0.00001];

        normalize_audio_samples(&mut samples);

        assert_eq!(samples, [0.0, 0.0]);
    }
}
