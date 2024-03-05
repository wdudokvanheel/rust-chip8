use wasm_bindgen::prelude::*;
use crate::application::start_application;

mod utils;
mod chip8;
mod wgpu_runtime;
mod application;

#[wasm_bindgen(start)]
pub fn wasm_start() {
    start_application();
}
