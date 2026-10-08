//! Windows-specific platform helpers.

#[allow(dead_code)]
pub fn set_desktop_wallpaper(_image_path: &str) -> anyhow::Result<()> {
    // Desktop wallpaper integration will be connected via Windows API in later milestone
    Ok(())
}
