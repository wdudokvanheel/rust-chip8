#[macro_use]
extern crate log;

use wasm_bindgen::prelude::*;

use crate::application::start_application;

mod utils;
mod chip8;
mod wgpu_runtime;
mod application;

fn main() {
    start_application();
}
