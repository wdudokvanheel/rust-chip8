#[macro_use]
extern crate log;

use std::process;
use std::usize;

use wasm_bindgen::prelude::*;

use crate::app::start;

mod utils;
mod app;
mod vertex;
mod chip8;

fn main() {
    start();
}
