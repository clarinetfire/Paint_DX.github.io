# Paint XP in Rust for Windows 11: feature baseline, layers, frames and sprite export

Oct 8, 2026 · @clarinetfire

This is the build spec for a Rust rewrite of Windows XP Paint (mspaint 5.1) for Windows 11, extended with RGBA layers, frame animation, PNG-sequence export (with or without transparency) and sprite-sheet export. It is written to be handed to Claude Code: Part 1 is the behaviour to reproduce, Part 2 is what to add, then come the Rust plan, the milestones and a CLAUDE.md starter.

## Assumptions to confirm

| Topic | Assumption |
| --- | --- |
| Language | Rust stable, target x86\_64-pc-windows-msvc |
| First platform | Windows 11 (64-bit); the core crate stays platform-neutral so Linux and macOS can follow |
| UI | egui through eframe; the canvas is a CPU RGBA buffer uploaded as a texture |
| Look | Classic XP Paint layout first; layers and timeline as extra panels |
| Pixel model | Straight RGBA8; anti-aliasing off by default to match XP's hard pixels |
| Project format | Own .paintproj file (zip + JSON); PNG, BMP, JPEG, GIF, TIFF stay as import/export formats |
| Name and assets | Not called Paint; no Microsoft icons, strings or resources are copied |

## Part 1: Paint XP feature inventory (baseline to reproduce)

### 1.1 Window and layout

- Single document window: no tabs, no MDI
- Menu bar: File, Edit, View, Image, Colors, Help
- Tool box on the left with 16 tools and an options area below it
- Color box at the bottom: 28 palette swatches plus foreground/background indicator
- Status bar: cursor coordinates, selection size, contextual help
- Canvas with three resize handles (right edge, bottom edge, bottom-right corner) and scroll bars
- Thumbnail window available when zoomed in

### 1.2 The 16 tools

| Tool | Behaviour in XP |
| --- | --- |
| Free-form select | Lasso selection of an irregular area |
| Select | Rectangular selection |
| Eraser / Color eraser | Eraser in 4 sizes paints the background color; right-drag replaces only the foreground color with the background color |
| Fill with color | Flood fill of a contiguous area of one exact color, no tolerance |
| Pick color | Eyedropper; left click sets foreground, right click sets background |
| Magnifier | Presets 1x, 2x, 4x, 6x, 8x, or zoom to a chosen area |
| Pencil | 1-pixel freehand line, no anti-aliasing |
| Brush | 12 tips (round, square, forward and backward slash), 3 sizes each |
| Airbrush | Spray paint in 3 sizes |
| Text | Text box with font, size, bold, italic, underline; opaque or transparent background; font toolbar can be hidden |
| Line | Straight line in 5 widths |
| Curve | Bezier curve with 1 or 2 control points, 5 widths |
| Rectangle | Outline, outline + fill, or fill only |
| Polygon | Multi-sided shape, same 3 fill styles |
| Ellipse | Same 3 fill styles |
| Rounded rectangle | Same 3 fill styles |

Left button draws with the foreground color, right button with the background color. Shift constrains lines to 45 degrees, rectangles to squares and ellipses to circles.

### 1.3 Selection behaviour

- Background mode: opaque, or transparent (the background color becomes see-through while moving)
- Drag to move; Ctrl + drag copies; Shift + drag leaves a trail (stamp effect)
- Resize through the handles
- Stretch and skew through the Image menu

### 1.4 File menu

- New, Open, Save, Save As
- From scanner or camera
- Print Preview, Page Setup, Print
- Send (by e-mail)
- Set as Wallpaper: tiled or centered
- Recent files list, Exit

### 1.5 File formats

- BMP: monochrome, 16 colors, 256 colors, 24-bit
- JPEG
- GIF (no animation, no transparency on save)
- TIFF
- PNG (saved without alpha channel, so no transparency)

### 1.6 Edit menu

- Undo and Redo, limited to 3 levels
- Repeat (F4)
- Cut, Copy, Paste
- Clear Selection (Del), Select All
- Copy To file, Paste From file

### 1.7 View menu

- Show or hide: tool box, color box, status bar, text toolbar
- Zoom: Normal (100%), Large (400%), Custom (100, 200, 400, 600, 800%)
- Show Grid (from 400% zoom)
- Show Thumbnail
- View Bitmap: full-screen view of the image (Ctrl+F)

### 1.8 Image menu

- Flip/Rotate: flip horizontal, flip vertical, rotate 90, 180, 270 degrees
- Stretch/Skew: stretch 1 to 500% each axis, skew -89 to 89 degrees each axis
- Invert Colors
- Attributes: width, height, units (inches, cm, pixels), color or black and white, default button
- Clear Image
- Draw Opaque (toggles transparent selection mode)

### 1.9 Colors menu

- Edit Colors: basic palette of 48 colors and 16 custom slots
- Define Custom Colors: hue, saturation, luminance, red, green, blue
- Add to Custom Colors
- Double-click a palette swatch to open the editor

### 1.10 Shortcuts

| Action | Shortcut |
| --- | --- |
| New, Open, Save, Print | Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+P |
| Undo, Redo, Repeat | Ctrl+Z, Ctrl+Y, F4 |
| Cut, Copy, Paste, Select All | Ctrl+X, Ctrl+C, Ctrl+V, Ctrl+A |
| Invert colors | Ctrl+I |
| Attributes, Stretch/Skew | Ctrl+E, Ctrl+W |
| Clear image | Ctrl+Shift+N |
| Grid, Text toolbar, View Bitmap | Ctrl+G, Ctrl+T, Ctrl+F |
| Brush and line size | Ctrl + numeric keypad + / - |
| Cancel current operation | Esc |

### 1.11 Limits of XP Paint (what Part 2 removes)

- Single bitmap: no layers
- No alpha channel, no real transparency
- No anti-aliasing on drawing tools
- Only 3 undo levels
- No frames, timeline or animation
- No batch or multi-file export

## Part 2: Additions to build

### 2.1 RGBA foundation (prerequisite for everything else)

- Every pixel is RGBA, 8 bits per channel; one RGBA buffer per layer
- Eraser sets alpha to 0 on a layer; it only paints the background color on a flattened, opaque image
- Pencil, brush, airbrush, fill, shapes and text all write alpha correctly
- Anti-aliasing toggle per tool, off by default to keep the XP pixel look
- Color picker gains an opacity slider; palette swatches can carry alpha
- Fill gains a tolerance setting and a mode: active layer only, or all visible layers
- Transparent canvas shown as a gray and white checkerboard
- Selection mode Opaque/Transparent from XP is replaced by real alpha compositing

### 2.2 Layers

**Data per layer:** id, name, visible, locked, opacity (0 to 100%), blend mode (Normal first), RGBA bitmap, offset (x, y).

**Operations**

- New, delete, duplicate, rename
- Reorder: move up, move down, drag and drop
- Show or hide, lock, set opacity
- Merge down, merge visible, flatten
- Move layer content with a Move tool

**Compositing**

- Visible layers are blended bottom to top with standard source-over alpha blending
- The composite is cached and only the dirty rectangle is recomputed after an edit
- Zoom, grid and checkerboard are applied on top of the composite

**Existing XP features adapted to layers**

- Select, cut, copy, paste, clear: act on the active layer; paste creates a new layer
- Pick color: sample the active layer or the composite (option)
- Flip, rotate, stretch, skew: active layer or whole image (option)
- Canvas resize (Attributes): applies to every layer
- Undo and redo record layer operations too

**UI:** a Layers panel with thumbnails, eye icon, lock icon, name, and buttons for new, delete, duplicate, up and down; matching Layer menu and shortcuts.

### 2.3 Frames and animation

**Recommended model:** layers are shared by the whole animation; each frame holds one cel (bitmap + offset) per layer. This gives a layers by frames grid, as in dedicated pixel-art tools.

**Data:** frame list with a duration in ms per frame, global FPS, loop on/off, named frame ranges (tags such as walk or jump) that the sprite-sheet export can use.

**Operations**

- Add, delete, duplicate, reorder frames
- New frame empty or copied from the previous one
- Copy and paste a frame or a single cel
- Set duration per frame or for all frames
- Go to first, previous, next, last frame

**Interface**

- Timeline panel: columns are frames, rows are layers, with a playhead
- Play, pause, loop buttons and an FPS field; live preview
- Onion skin: previous and next frames shown semi-transparent (number of frames and opacity adjustable, optional red/blue tint)

**Playback:** timer driven by each frame's duration; the current frame is composited on the fly.

### 2.4 PNG sequence export

For each frame: composite the visible layers into an RGBA bitmap, apply scale, background and trim, encode as PNG, write the file. Show a progress bar with a cancel button.

| Option | Values | Default |
| --- | --- | --- |
| Output folder | Any folder | Project folder / export |
| Name pattern | Prefix + zero-padded index, e.g. name\_0001.png | name\_0000.png |
| Start index and padding | Integer, 1 to 8 digits | 0, 4 digits |
| Frame range | All, selected frames, one tag | All |
| Layers | Visible composite, one layer only, or one sequence per layer | Visible composite |
| Transparency | Keep transparency (RGBA PNG), or flatten onto a solid color (opaque PNG) | Keep transparency |
| Background color | Color picker, used only when flattening | White |
| Scale | 1x to 8x, nearest-neighbor | 1x |
| Trim | None, per frame, or union of all frames | None |
| Padding | 0 to N pixels around each image | 0 |
| Existing files | Overwrite, skip, auto-rename | Ask |

Union trim crops every frame by the same box so the animation stays aligned; per-frame trim must write the offset into the metadata.

### 2.5 Sprite sheet export

All chosen frames are composited and placed on one RGBA bitmap that starts fully transparent (or a solid color if flattening), then saved as PNG, with an optional metadata file.

| Option | Values |
| --- | --- |
| Layout | Horizontal strip, vertical strip, grid with N columns, or packed (MaxRects-style) |
| Cell size | Canvas size, or the largest trimmed frame |
| Spacing and border | Pixels between cells; pixels around the sheet |
| Edge extrusion | 0 or 1 pixel, avoids bleeding when engines filter textures |
| Sheet size | Automatic, power of two, or a maximum with an error on overflow |
| Transparency | Transparent sheet, or solid background color |
| Scale | 1x to 8x, nearest-neighbor |
| Range and order | All, selection, or one tag; ordered by frame or by tag |
| Metadata | JSON, CSV, or none |

Proposed JSON metadata (schema versioned, kept simple; an Aseprite-style variant can be added later):

```json
{
  "version": 1,
  "image": "hero.png",
  "size": { "w": 256, "h": 64 },
  "frames": [
    { "index": 0, "x": 0, "y": 0, "w": 64, "h": 64, "duration_ms": 100, "trim_offset": { "x": 0, "y": 0 } }
  ],
  "tags": [ { "name": "walk", "from": 0, "to": 3 } ]
}
```

Show a preview of the sheet before writing it.

### 2.6 Undo and redo

- Replace XP's 3 levels with a deep history (at least 100 steps, bounded by memory)
- Store changed pixel rectangles (diffs), not full layer copies
- Layer and frame operations are history entries too
- Optional history panel

### 2.7 Project file

- Needed because PNG, BMP and JPEG cannot hold layers or frames
- .paintproj = zip archive with a manifest.json and one PNG per cel
- Manifest: canvas size, custom palette, layers, frames and durations, tags, last export settings, format version
- Opening a PNG, BMP, JPEG, GIF or TIFF creates a one-layer, one-frame project
- Importing a PNG sequence creates frames
- Autosave and crash recovery

## Rust technical plan

### 3.1 Crates

| Need | Crate | Note |
| --- | --- | --- |
| Window and UI | eframe / egui | Immediate-mode UI; canvas drawn as a texture with nearest-neighbor sampling |
| Image buffers and codecs | image | RGBA buffers; PNG, BMP, JPEG, GIF, TIFF read/write |
| Project manifest | serde, serde\_json | Versioned manifest |
| Project archive | zip | .paintproj container |
| File dialogs | rfd | Native Windows 11 dialogs |
| Clipboard images | arboard | Copy and paste bitmaps with other apps |
| Optional anti-aliased shapes | tiny-skia | Behind the anti-aliasing toggle; XP-style shapes use own Bresenham code |
| Text tool | cosmic-text or ab\_glyph | Rasterize text into the layer |
| Sprite packing | rectangle-pack, or own grid/strip packer | Start with grid and strips, add packing later |
| Errors | thiserror, anyhow | Library and app errors |
| Windows APIs | windows | Wallpaper, later printing and scanning (WIA) |
| Packaging | cargo-wix or MSIX tooling | Installer, file association |

### 3.2 Workspace layout

```text
paint-rs/
  Cargo.toml                 (workspace)
  CLAUDE.md
  docs/xp-paint.md           (Part 1 of this spec)
  crates/
    paint-core/              (no UI, no OS code)
      src/image.rs           RGBA buffer, blending
      src/tools/             pencil, brush, fill, shapes, select, text
      src/layers.rs
      src/frames.rs
      src/history.rs
      src/project.rs         .paintproj read/write
      src/export/png_seq.rs
      src/export/sheet.rs
    paint-app/               (egui UI, Windows integration)
      src/main.rs
      src/ui/                canvas, toolbox, palette, layers, timeline
      src/platform/windows.rs
  tests/golden/              reference PNGs
```

### 3.3 Core data model (sketch)

```rust
pub struct Document {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,        // bottom to top, shared by all frames
    pub frames: Vec<Frame>,
    pub fps: u32,
    pub tags: Vec<AnimTag>,        // named frame ranges
}

pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: u8,               // 0..=100
    pub blend: BlendMode,
}

pub struct Frame {
    pub duration_ms: u32,
    pub cels: HashMap<LayerId, Cel>,
}

pub struct Cel {
    pub x: i32,
    pub y: i32,
    pub image: image::RgbaImage,
}

pub enum ExportBackground {
    Transparent,
    Solid(image::Rgba<u8>),
}
```

### 3.4 Windows 11 specifics

- Target x86\_64-pc-windows-msvc with the MSVC build tools installed; aarch64 can follow
- Per-monitor DPI awareness; the canvas snaps to integer zoom with nearest-neighbor filtering
- Follow the system light or dark theme
- Release build uses the windows GUI subsystem (no console window)
- Wallpaper (tiled, centered) through the windows crate; print and scanner support deferred
- Installer with .paintproj file association; code signing before public release
- CI on GitHub Actions windows-latest: fmt, clippy, tests, release build

## Milestones for Claude Code

One milestone per session; each ends with passing tests and a commit.

1. **Workspace and canvas.** Cargo workspace, paint-core RGBA buffer, XP 28-color palette, pencil and line, egui window with canvas, zoom 100 to 800% and grid. Done when you can draw, zoom and see the grid at 800%.
2. **XP tool parity.** All 16 tools with their options, selection move/copy/trail, text. Done when each tool has a golden-image test in tests/golden.
3. **XP menus and formats.** New, Open, Save As for BMP, JPEG, GIF, TIFF, PNG; Edit, View, Image and Colors menus; shortcuts from 1.10. Done when every menu item in Part 1 works or is explicitly listed as deferred.
4. **Alpha and deep history.** RGBA everywhere, transparent eraser, checkerboard, opacity slider, diff-based undo. Done when 100 undo steps round-trip without pixel drift.
5. **Layers.** Data model, compositing with dirty rectangles, Layers panel, tools acting on the active layer. Done when a 5-layer image composites identically to a reference PNG.
6. **Project file.** .paintproj save and load, autosave. Done when save then load gives a byte-identical composite.
7. **Frames and timeline.** Cels, timeline, durations, playback, onion skin, tags. Done when a 4-frame animation plays at the set durations.
8. **PNG sequence export.** Dialog with all 2.4 options. Done when a transparent export has alpha 0 on empty pixels and a flattened export has none.
9. **Sprite sheet export.** Strip, grid, spacing, extrusion, JSON metadata, preview. Done when metadata coordinates match the pixels of the generated sheet in a test.
10. **Windows 11 polish.** DPI, dark mode, installer, file association, signing, CI release. Done when a clean Windows 11 machine installs and runs it.

## CLAUDE.md starter

```markdown
# Project: paint-rs
Rust rewrite of Windows XP Paint for Windows 11, with layers, frames,
PNG-sequence and sprite-sheet export. Spec: docs/xp-paint.md and the build spec.

## Commands
- cargo build --workspace
- cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cargo fmt --all -- --check
- cargo run -p paint-app

## Rules
- paint-core has no UI, no OS calls, no unsafe.
- Pixel behaviour changes need a golden PNG in tests/golden.
- Match XP behaviour from docs/xp-paint.md unless the spec says otherwise.
- Anti-aliasing is off by default.
- Ask before adding a dependency; note why in the commit message.
- Do one milestone per session, then commit.
- Never copy Microsoft icons, strings or resources.
```

## Open questions

- Confirm egui, or prefer a native Win32/Direct2D UI (more work, more native look)?
- Frame model: shared layers with one cel per frame (recommended), or a full layer stack per frame?
- Should the app name and icon be decided now, so nothing resembles Microsoft Paint branding?
- Which engine will consume the sprite sheets (Unity, Godot, Phaser, custom)? This decides the metadata schema.
