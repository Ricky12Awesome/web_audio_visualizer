mod ftt;

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::ftt::FFT;

#[derive(Serialize, Deserialize)]
pub struct Config {
    enabled: bool,
}

#[repr(C)]
pub struct AudioBufferState {
    sequence: AtomicU32,
    frames_written: AtomicU32,
    channels: AtomicU32,
    frames: AtomicU32,
}

struct AudioBuffer {
    samples: Vec<f32>,
    state: Box<AudioBufferState>,
}

static mut AUDIO_BUFFER: *mut AudioBuffer = std::ptr::null_mut();

#[unsafe(no_mangle)]
pub extern "C" fn audio_buffer_init(channels: u32, frames: u32) -> *mut f32 {
    let mut buffer = AudioBuffer {
        samples: vec![0.0; channels.saturating_mul(frames) as usize],
        state: Box::new(AudioBufferState {
            sequence: AtomicU32::new(0),
            frames_written: AtomicU32::new(0),
            channels: AtomicU32::new(channels),
            frames: AtomicU32::new(frames),
        }),
    };
    let samples = buffer.samples.as_mut_ptr();
    unsafe {
        AUDIO_BUFFER = Box::into_raw(Box::new(buffer));
    }
    samples
}

#[unsafe(no_mangle)]
pub extern "C" fn audio_buffer_state() -> *mut AudioBufferState {
    unsafe {
        let buffer = AUDIO_BUFFER;
        if buffer.is_null() {
            std::ptr::null_mut()
        } else {
            (*buffer).state.as_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn audio_buffer_samples() -> *mut f32 {
    unsafe {
        let buffer = AUDIO_BUFFER;
        if buffer.is_null() {
            std::ptr::null_mut()
        } else {
            (*buffer).samples.as_mut_ptr()
        }
    }
}

async fn start() {
    let mut fft = FFT::default();

    loop {
        clear_background(Color::from_rgba(0, 0, 0, 0));

        let buffer = unsafe { AUDIO_BUFFER.as_ref() };
        let Some(buffer) = buffer else {
            next_frame().await;
            continue;
        };

        let sequence = buffer.state.sequence.load(Ordering::Acquire);
        if sequence % 2 != 0 {
            next_frame().await;
            continue;
        }

        let channels = buffer.state.channels.load(Ordering::Relaxed) as usize;
        let frames = buffer.state.frames_written.load(Ordering::Relaxed) as usize;
        if channels == 0 || frames == 0 {
            next_frame().await;
            continue;
        }

        draw_text(
            &format!("FPS: {}", get_fps()),
            screen_width() - 120.,
            32.,
            24.,
            crate::WHITE,
        );
        // draw_text(&format!("{:.3}", audio_level()), 20., 40., 32., WHITE);
        draw_text(&format!("{}", buffer.samples.len()), 20., 40., 32., WHITE);

        let radius = 128.;
        let x = screen_width() / 2.;
        let y = screen_height() / 2.;

        let samples = (0..frames)
            .map(|frame| {
                (0..channels)
                    .map(|channel| buffer.samples[channel * frames + frame])
                    .sum::<f32>()
                    / channels as f32
            })
            .collect::<Vec<_>>();

        if sequence != buffer.state.sequence.load(Ordering::Acquire) {
            next_frame().await;
            continue;
        }

        let buffer = fft.process(&samples, 128);

        draw(&buffer, x, y, radius, 128., WHITE);

        next_frame().await;
    }
}

fn draw(buffer: &[f32], x: f32, y: f32, radius: f32, scale: f32, color: Color) {
    let len = buffer.len() / 2;
    let step = (PI * 2.) / len as f32;

    for i in 0..len {
        let angle = i as f32 * step;
        let l = buffer[i].abs() * scale;
        let ix = x + angle.cos() * radius;
        let iy = y + angle.sin() * radius;
        let ox = x + angle.cos() * (radius + l);
        let oy = y + angle.sin() * (radius + l);

        draw_line(ix, iy, ox, oy, 1.0, color);
    }
}

fn conf() -> Conf {
    Conf {
        window_resizable: true,
        high_dpi: true,
        window_title: "".to_string(),
        ..Conf::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    start().await;
}
