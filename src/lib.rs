use wasm_bindgen::prelude::*;

use crate::app::start;

mod utils;
mod app;
mod vertex;
mod chip8;

#[wasm_bindgen(start)]
pub fn wasm_start() {
    start();
}
