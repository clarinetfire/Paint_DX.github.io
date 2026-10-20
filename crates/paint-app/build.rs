fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.set_manifest_file("paint-app.manifest");
        res.set("FileDescription", "Paint DX");
        res.set("ProductName", "Paint DX");
        res.set("OriginalFilename", "Paint DX.exe");
        res.set("LegalCopyright", "Copyright (c) 2026 Paint DX Contributors");
        res.set("ProductVersion", "0.1.0.0");
        res.set("FileVersion", "0.1.0.0");

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile windows resource: {}", e);
        }
    }
}
