// src/wasm.rs
use wasm_bindgen::prelude::*;

// We just re-use the same run() from lib
#[wasm_bindgen(start)]
pub fn start() {
    asteroids::run();
}
