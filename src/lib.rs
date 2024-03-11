use std::sync::mpsc::Sender;

use wasm_bindgen::prelude::*;

use crate::application::{AppCommand, RuntimeData, start_application};
use crate::application::AppCommand::RESET;
use crate::wgpu_runtime::WgpuRuntime;

mod utils;
mod chip8;
mod wgpu_runtime;
mod application;

#[wasm_bindgen]
pub struct WasmRuntime {
    runtime: WgpuRuntime<RuntimeData, AppCommand>,
}

#[wasm_bindgen]
pub struct CallBack {
    sender: Sender<AppCommand>,
}

#[wasm_bindgen]
impl CallBack {
    pub fn reset(&mut self) {
        self.sender.send(RESET).unwrap();
    }
}

#[wasm_bindgen]
impl WasmRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let runtime = start_application();

        return WasmRuntime {
            runtime,
        };
    }

    pub fn get_sender(&mut self) -> CallBack {
        CallBack {
            sender: self.runtime.get_command_sender()
        }
    }

    pub fn start(mut self) {
        self.runtime.start();
    }
}
