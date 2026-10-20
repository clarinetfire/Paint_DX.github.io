//! Platform integration modules.

#[cfg(target_arch = "wasm32")]
pub mod web;

pub mod windows;

pub fn play_startup_sound() {
    #[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
    windows::play_startup_sound();

    #[cfg(target_arch = "wasm32")]
    web::play_startup_sound();

    #[cfg(all(not(target_os = "windows"), not(target_arch = "wasm32")))]
    windows::play_startup_sound();
}
