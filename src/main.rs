mod ftt;

use crate::ftt::FFT;
use macroquad::prelude::*;
use palette::rgb::Rgb;
use palette::{Hsl, IntoColor, RgbHue};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicU32, Ordering};

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

#[derive(Default)]
struct Retention {
    buf: Vec<f32>,
    delta_seconds: f32,
    retention_seconds: f32,
}

impl Retention {
    fn update(&mut self, len: usize) {
        if self.retention_seconds == 0.0 {
            return;
        }

        self.delta_seconds = get_frame_time();
        if len > self.buf.len() {
            self.buf.resize(len, 0.0);
        }
    }

    fn retain(&mut self, i: usize, target: f32) -> f32 {
        if self.retention_seconds == 0.0 {
            return target;
        }

        self.buf[i] = Self::settle_bar(
            self.buf[i],
            target,
            self.delta_seconds,
            self.retention_seconds,
        );

        self.buf[i].max(1.)
    }

    fn settle_bar(current: f32, target: f32, delta_seconds: f32, retention_seconds: f32) -> f32 {
        if target >= current || retention_seconds <= 0.0 {
            return target;
        }

        (current - (current - target) * (delta_seconds / retention_seconds)).max(target)
    }
}

fn rgb_hue(i: usize, len: usize, _: f32) -> Color {
    // WHITE

    let hsl = Hsl::new_srgb(RgbHue::new((360. / len as f32) * i as f32), 1.0, 0.5);
    let Rgb {
        red, green, blue, ..
    } = hsl.into_color();

    Color::new(red, green, blue, 1.0)
}

async fn start() {
    let mut fft = FFT::default();
    let mut retention = Retention {
        // retention_seconds: 16.667 / 1000.,
        retention_seconds: 0.,
        ..Retention::default()
    };

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

        let radius = (screen_height() / 2.5).min(screen_width() / 2.5);
        let scale = 256.0;

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

        let buffer = &samples;
        let buffer = fft.process(&buffer, 512);

        retention.update(buffer.len());

        let half_l = &buffer[..buffer.len() / 2];
        let half_r = &buffer[buffer.len() / 2..];

        draw_text(
            format!("{} --- {}", half_l.len(), half_r.len()),
            20.,
            70.,
            32.,
            WHITE,
        );

        let skip = 2;
        let iter = half_l
            .iter()
            .skip(skip)
            .rev()
            .chain(half_r.iter().rev().skip(skip))
            .map(|bar| (1.0 + bar.abs() * 1.0).ln() * scale);

        // let iter = iter.clone().chain(iter.rev());
        // let mut n = 2;
        // let iter = iter.clone().chain(iter.rev());
        // n *= 2;
        // let iter = iter.clone().chain(iter.rev());
        // n *= 2;
        // let iter = iter.clone().chain(iter.rev());
        // n *= 2;
        // let iter = iter.clone().chain(iter.rev());
        // n *= 2;
        // let iter = iter.clone().chain(iter.rev());
        // n *= 2;
        // let iter = iter.clone().chain(iter.rev());
        // n *= 2;
        // let iter = iter.clone().chain(iter.rev());
        // n *= 2;

        // draw_line_visualizer(
        //     iter,
        //     buffer.len() - (n * 2),
        //     screen_height() / 2.,
        //     &mut retention,
        //     VisualizerMode::Vertical,
        //     rgb_hue,
        // );

        // draw_circle_visualizer(
        //     iter,
        //     buffer.len() * n - (skip * 2 * n),
        //     screen_width() / 2.,
        //     screen_height() / 2.,
        //     radius,
        //     &mut retention,
        //     rgb_hue,
        // );

        // draw_circle_v(
        //     &buffer,
        //     screen_width() / 2.,
        //     screen_height() / 2.,
        //     radius,
        //     128.,
        //     WHITE,
        // );

        test(
            iter,
            buffer.len() - (skip * 2),
            screen_width() / 2.,
            screen_height() / 2.,
            radius,
            &mut retention,
            rgb_hue,
        );

        draw_text(
            format!("{:}", get_frame_time() * 1000.),
            200.,
            32.,
            24.,
            WHITE,
        );

        next_frame().await;
    }
}

#[derive(Serialize, Deserialize)]
enum VisualizerMode {
    Vertical,
    Up,
    Down,
}

fn draw_line_visualizer(
    buffer: impl Iterator<Item = f32>,
    buffer_len: usize,
    y: f32,
    retention: &mut Retention,
    mode: VisualizerMode,
    color: impl Fn(usize, usize, f32) -> Color,
) {
    let width = screen_width();
    let offset = (screen_width() - width) / 2.;
    let step = width / buffer_len as f32;

    for (i, bar) in buffer.enumerate() {
        let bar = retention.retain(i, bar);
        let color = color(i, buffer_len, bar);
        let pos = offset + (step * i as f32);

        match mode {
            VisualizerMode::Vertical => {
                draw_line(pos, y - bar / 2., pos, y + bar / 2., step, color);
            }
            VisualizerMode::Up => {
                draw_line(pos, y - bar, pos, y, step, color);
            }
            VisualizerMode::Down => {
                draw_line(pos, y, pos, y + bar, step, color);
            }
        }
    }
}

fn draw_circle_visualizer(
    buffer: impl Iterator<Item = f32>,
    buffer_len: usize,
    x: f32,
    y: f32,
    radius: f32,
    retention: &mut Retention,
    color: impl Fn(usize, usize, f32) -> Color,
) {
    let step = (PI * 2.) / buffer_len as f32;

    for (i, bar) in buffer.enumerate() {
        let bar = retention.retain(i, bar);
        let color = color(i, buffer_len, bar);
        let angle = i as f32 * step;
        let ix = x + angle.cos() * (radius - (bar / 2.));
        let iy = y + angle.sin() * (radius - (bar / 2.));
        let ox = x + angle.cos() * (radius + (bar / 2.));
        let oy = y + angle.sin() * (radius + (bar / 2.));

        draw_line(ix, iy, ox, oy, radius * step, color);
    }
}

fn test(
    buffer: impl Iterator<Item = f32>,
    buffer_len: usize,
    x: f32,
    y: f32,
    radius: f32,
    retention: &mut Retention,
    color: impl Fn(usize, usize, f32) -> Color,
) {
    let mut vertices = Vec::with_capacity(buffer_len + 1);
    let mut indices = Vec::with_capacity(buffer_len * 3);

    let width = screen_width();
    let offset = (screen_width() - width) / 2.;
    let step = width / buffer_len as f32;

    // Center vertex.
    vertices.push(Vertex::new(offset, y, 0.0, 0.0, 0.0, WHITE));

    for (i, bar) in buffer.enumerate() {
        let bar = retention.retain(i, bar);
        let color = color(i, buffer_len, bar);
        let pos = offset + (step * i as f32);

        vertices.push(Vertex::new(pos, y - bar, 0.0, 0.0, 0.0, color));

        let current = i + 1;
        let next = ((i + 1) % buffer_len) + 1;

        if next == buffer_len {
            draw_text(
                format!("{} -- {} --- {}", current, next, buffer_len),
                20.,
                102.,
                32.,
                WHITE,
            );

            indices.extend_from_slice(&[0, current as _]);
            continue;
        } else {
            indices.extend_from_slice(&[(current - 1) as _, current as _, next as _]);
        }
    }

    // Make a triangle fan around the center.
    // for i in 0..buffer_len {
    //     let current = i + 1;
    //     let next = ((i + 1) % buffer_len) + 1;
    //
    //     indices.extend_from_slice(&[0, current as u16, next as u16]);
    // }

    let mesh = Mesh {
        vertices,
        indices,
        texture: None,
    };

    draw_mesh(&mesh);
}

fn draw_line_v(buffer: &[f32], y: f32, scale: f32, color: Color, retention: &mut Retention) {
    let step = screen_width() / buffer.len() as f32;

    for (i, bar) in buffer.iter().enumerate() {
        let hsl = Hsl::new_srgb(
            RgbHue::new((360. / buffer.len() as f32) * i as f32),
            1.0,
            0.5,
        );
        let Rgb {
            red, green, blue, ..
        } = hsl.into_color();

        let pos = step * i as f32;
        let l = retention.retain(i, bar.abs() * scale);

        // draw_rectangle(pos, y - l / 2., step, l, Color::new(red, green, blue, 1.0));
        draw_line(
            pos,
            y - l / 2.,
            pos,
            y + l / 2.,
            step,
            Color::new(red, green, blue, 1.0),
        );
    }
}

fn draw_circle_v(buffer: &[f32], x: f32, y: f32, radius: f32, scale: f32, color: Color) {
    let len = buffer.len() / 2;
    let step = (PI * 2.) / len as f32;

    for i in 0..len {
        let angle = i as f32 * step;
        let l = buffer[i].abs() * scale;
        let ix = x + angle.cos() * (radius - (l / 2.));
        let iy = y + angle.sin() * (radius - (l / 2.));
        let ox = x + angle.cos() * (radius + (l / 2.));
        let oy = y + angle.sin() * (radius + (l / 2.));

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
