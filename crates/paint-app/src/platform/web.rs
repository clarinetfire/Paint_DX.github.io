//! Web platform helpers (audio playback, browser integration).

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = playStartupSound, catch)]
    fn js_play_startup_sound() -> Result<(), JsValue>;

    #[wasm_bindgen(js_name = hideLoadingScreen, catch)]
    fn js_hide_loading_screen() -> Result<(), JsValue>;
}

pub fn play_startup_sound() {
    let _ = js_play_startup_sound();
}

pub fn hide_loading_screen() {
    let _ = js_hide_loading_screen();
}
