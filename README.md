# Paint DX

**Paint DX** is a modern, high-performance Rust recreation of classic Windows XP Paint for Windows 11. It combines the nostalgic, pixel-crisp workflow of XP Paint with modern creative power: full layer stacks, cel animation timeline, non-destructive undo/redo history, and game engine export (PNG sequences and sprite sheets).

![Paint DX Icon](crates/paint-app/assets/app.png)

## Features

- **Pixel-Perfect XP Toolset**: All 16 classic tools (Pencil, Brush with 12 tips, Airbrush, Line, Curve, Rectangle, Polygon, Ellipse, Rounded Rectangle, Fill, Color Picker, Magnifier, Eraser/Color Eraser, Text, Rectangular & Freeform Selection).
- **Layers & Compositing**: Multi-layer canvas with opacity control, blend modes, visibility toggling, layer reordering, and dirty-rect incremental compositing.
- **Animation Timeline & Frames**: Frame cels, per-frame durations, animation tags, real-time playback, and onion skinning.
- **Deep History**: Diff-based 100+ undo/redo steps with zero pixel drift.
- **Export Formats**:
  - Classic image formats: BMP, PNG, JPEG, GIF, TIFF.
  - Project file: `.paintproj` (multi-layer, multi-frame lossless archives with autosave and crash recovery).
  - PNG Sequence: transparent/flattened export, frame trimming, FPS timing.
  - Sprite Sheet: Packed, Grid, Horizontal & Vertical strips with JSON and CSV engine metadata.
- **Windows 11 Native Integration**:
  - High-DPI per-monitor awareness.
  - Dark mode and Light (XP classic) theme switching.
  - No pop-up console window (`windows` subsystem in release).
  - File associations for `.paintproj` and Windows Explorer context menu integration.
  - Desktop wallpaper setting (Centered & Tiled).

## Building & Running

### Requirements
- Rust 1.80+ (MSVC toolchain on Windows recommended)
- `cargo`

### Quick Start
```bash
# Run Paint DX
cargo run -p paint-app --release

# Run Tests
cargo test --workspace

# Lint Check
cargo clippy --workspace -- -D warnings
```

## Installation & File Associations

To install locally and associate `.paintproj` files with Paint DX:
```powershell
powershell -ExecutionPolicy Bypass -File installer/install.ps1
```

To uninstall:
```powershell
powershell -ExecutionPolicy Bypass -File installer/uninstall.ps1
```

## License
Licensed under MIT or Apache-2.0.
