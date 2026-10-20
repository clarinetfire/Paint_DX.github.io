//! Windows-specific platform helpers (Wallpaper, Registry, File Associations).

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperStyle {
    Tiled,
    Centered,
}

#[cfg(target_os = "windows")]
pub fn set_desktop_wallpaper(image_path: &Path, style: WallpaperStyle) -> anyhow::Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::process::Command;

    let (tile_val, style_val) = match style {
        WallpaperStyle::Tiled => ("1", "0"),
        WallpaperStyle::Centered => ("0", "0"),
    };

    // Update Registry for Wallpaper Style
    let _ = Command::new("reg")
        .args([
            "add",
            r"HKCU\Control Panel\Desktop",
            "/v",
            "TileWallpaper",
            "/t",
            "REG_SZ",
            "/d",
            tile_val,
            "/f",
        ])
        .output();

    let _ = Command::new("reg")
        .args([
            "add",
            r"HKCU\Control Panel\Desktop",
            "/v",
            "WallpaperStyle",
            "/t",
            "REG_SZ",
            "/d",
            style_val,
            "/f",
        ])
        .output();

    // Convert path to wide string
    let wide_path: Vec<u16> = OsStr::new(image_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Call SystemParametersInfoW through windows user32 dll
    #[link(name = "user32")]
    extern "system" {
        fn SystemParametersInfoW(
            uiAction: u32,
            uiParam: u32,
            pvParam: *const u16,
            fWinIni: u32,
        ) -> i32;
    }

    const SPI_SETDESKWALLPAPER: u32 = 0x0014;
    const SPIF_UPDATEINIFILE: u32 = 0x01;
    const SPIF_SENDCHANGE: u32 = 0x02;

    let success = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            wide_path.as_ptr(),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    };

    if success != 0 {
        Ok(())
    } else {
        anyhow::bail!("SystemParametersInfoW failed to set wallpaper")
    }
}

#[cfg(target_os = "windows")]
pub fn register_file_associations() -> anyhow::Result<()> {
    use std::process::Command;

    let current_exe = std::env::current_exe()?;
    let exe_path_str = current_exe.to_string_lossy().to_string();
    let open_cmd = format!("\"{}\" \"%1\"", exe_path_str);
    let icon_cmd = format!("\"{}\",0", exe_path_str);

    // Register .paintproj extension in current user classes
    let commands = [
        // .paintproj -> PaintDX.Project
        vec![
            "add",
            r"HKCU\Software\Classes\.paintproj",
            "/ve",
            "/d",
            "PaintDX.Project",
            "/f",
        ],
        // PaintDX.Project friendly name
        vec![
            "add",
            r"HKCU\Software\Classes\PaintDX.Project",
            "/ve",
            "/d",
            "Paint DX Project",
            "/f",
        ],
        // DefaultIcon
        vec![
            "add",
            r"HKCU\Software\Classes\PaintDX.Project\DefaultIcon",
            "/ve",
            "/d",
            &icon_cmd,
            "/f",
        ],
        // Shell Open Command
        vec![
            "add",
            r"HKCU\Software\Classes\PaintDX.Project\shell\open\command",
            "/ve",
            "/d",
            &open_cmd,
            "/f",
        ],
        // Context menu for common image types
        vec![
            "add",
            r"HKCU\Software\Classes\SystemFileAssociations\image\shell\EditWithPaintDX",
            "/ve",
            "/d",
            "Edit with Paint DX",
            "/f",
        ],
        vec![
            "add",
            r"HKCU\Software\Classes\SystemFileAssociations\image\shell\EditWithPaintDX\DefaultIcon",
            "/ve",
            "/d",
            &icon_cmd,
            "/f",
        ],
        vec![
            "add",
            r"HKCU\Software\Classes\SystemFileAssociations\image\shell\EditWithPaintDX\command",
            "/ve",
            "/d",
            &open_cmd,
            "/f",
        ],
    ];

    for args in commands {
        let status = Command::new("reg").args(&args).status()?;
        if !status.success() {
            anyhow::bail!("Failed to register registry key: {:?}", args);
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn unregister_file_associations() -> anyhow::Result<()> {
    use std::process::Command;

    let keys_to_delete = [
        r"HKCU\Software\Classes\.paintproj",
        r"HKCU\Software\Classes\PaintDX.Project",
        r"HKCU\Software\Classes\SystemFileAssociations\image\shell\EditWithPaintDX",
    ];

    for key in keys_to_delete {
        let _ = Command::new("reg").args(["delete", key, "/f"]).status();
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn set_desktop_wallpaper(_image_path: &Path, _style: WallpaperStyle) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn register_file_associations() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn unregister_file_associations() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn play_startup_sound() {
    static STARTUP_WAV: &[u8] = include_bytes!("../../assets/startup.wav");

    #[link(name = "winmm")]
    extern "system" {
        fn PlaySoundW(pszSound: *const u16, hmod: *const std::ffi::c_void, fdwSound: u32) -> i32;
    }

    const SND_ASYNC: u32 = 0x0001;
    const SND_NODEFAULT: u32 = 0x0002;
    const SND_MEMORY: u32 = 0x0004;

    unsafe {
        PlaySoundW(
            STARTUP_WAV.as_ptr() as *const u16,
            std::ptr::null(),
            SND_ASYNC | SND_MEMORY | SND_NODEFAULT,
        );
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn play_startup_sound() {}
