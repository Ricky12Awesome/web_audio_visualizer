use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    enabled: bool,
    // more settings will be added in future
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {

}

#[macroquad::main("Web Audio Visualizer")]
async fn main() {
    loop {
        clear_background(Color::from_rgba(0, 0, 0, 0));

        draw_text("", 20., 20., 32., WHITE);

        next_frame().await
    }
}
