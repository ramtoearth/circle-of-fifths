pub mod music_theory;
pub mod state;
pub mod data;
pub mod audio;
pub mod storage;
pub mod components;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<components::app::App>::new().render();
}
