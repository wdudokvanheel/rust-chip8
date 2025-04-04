use std::sync::mpsc::Sender;

use wasm_bindgen::prelude::*;

use crate::application::{AppCommand, RuntimeData, start_application};
use crate::application::AppCommand::{LOAD_ROM, RESET};
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

    pub fn load_rom(&mut self, id: u8) {
        self.sender.send(LOAD_ROM(id)).unwrap();
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

    pub fn get_roms(&self) -> Vec<JsValue> {
        let mut roms = vec![];

        if let Some(data) = self.runtime.data.as_ref() {
            roms = data.rom_list();
        }

        return roms.iter().map(|name| JsValue::from_str(&format!("{}", name))).collect();
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
