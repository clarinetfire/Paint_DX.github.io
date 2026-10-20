# Paint DX - Post-Milestone Comprehensive Audit Report

**Audit Date**: October 8, 2026  
**Auditor**: Antigravity Pair-Programming Agent  
**Scope**: Full codebase audit after completion of Milestones 1 through 10.

---

## 1. Executive Summary

| Category | Metric / Status | Result |
|---|---|---|
| **Milestone Acceptance** | 10 / 10 Milestones Passed | **100% PASS** |
| **Workspace Build (`debug`)** | `cargo build --workspace` | **PASS** (0 errors) |
| **Release Build (`release`)** | `cargo build --release -p paint-app` | **PASS** (0 errors) |
| **Workspace Tests** | `cargo test --workspace` | **PASS** (88/88 passed, 0 failed, 0 ignored) |
| **Clippy Linter** | `cargo clippy --workspace -- -D warnings` | **PASS** (0 warnings) |
| **Format Check** | `cargo fmt --all -- --check` | **PASS** (Clean) |
| **Unimplemented / Stubs** | `TODO`, `FIXME`, `unimplemented!`, `todo!` | **0 found** |
| **Core Architecture Purity** | `paint-core` `#![forbid(unsafe_code)]`, no UI/OS deps | **VERIFIED** |
| **Feature Completeness** | Baseline XP (1.1–1.10) + DX Additions (2.1–2.7) | **Done: 58, Deferred: 3, Missing: 0** |

---

## 2. Health Check Details

### 2.1 Build & Test Execution Output Summary
- **`cargo build --workspace`**: Clean compilation of `paint-core` and `paint-app` (0 errors, 0 warnings).
- **`cargo build --release -p paint-app`**: Clean compilation with LTO and release optimizations, windows subsystem enabled (`#![windows_subsystem = "windows"]`).
- **`cargo test --workspace`**:
  - `paint-core` unit tests: **61 passed**, 0 failed, 0 ignored.
  - `paint-app` unit tests: **1 passed**, 0 failed, 0 ignored.
  - Integration & Golden tests (`crates/paint-core/tests/`): **26 passed**, 0 failed, 0 ignored.
    - `golden_all_tools.rs`: 7 tests
    - `golden_animation_playback.rs`: 1 test
    - `golden_layers_composite.rs`: 2 tests
    - `golden_pencil_line.rs`: 1 test
    - `golden_png_sequence_export.rs`: 7 tests
    - `golden_project_roundtrip.rs`: 2 tests
    - `golden_sprite_sheet_export.rs`: 5 tests
    - `history_100_steps.rs`: 1 test
  - **Total**: **88 tests passed**, 0 failed, 0 ignored.
- **`cargo clippy --workspace -- -D warnings`**: Passed with 0 warnings.
- **`cargo fmt --all -- --check`**: Passed with zero formatting diffs.

### 2.2 Panics, Unwraps, and Safety
- **`paint-core`**:
  - `#![forbid(unsafe_code)]` strictly enforced at root of `crates/paint-core/src/lib.rs`.
  - Zero `.unwrap()`, `.expect()`, or `panic!` calls in non-test production code.
- **`paint-app`**:
  - `unsafe` blocks: 1 localized block in `crates/paint-app/src/platform/windows.rs` for Windows Win32 API call `SystemParametersInfoW` (wallpaper setting).
  - `.unwrap()` in production code: 4 occurrences in `crates/paint-app/src/ui/canvas.rs` (lines 492, 916, 917, 918) which are strictly branch-guarded by previous `is_some()` or state-machine checks.
  - Zero `panic!` or `todo!` calls in production code.

---

## 3. Milestone Acceptance Matrix

| # | Milestone Description | Done When Criterion | Status | Evidence (Code / Test) |
|---|---|---|---|---|
| **M1** | **Workspace and canvas** | Draw, zoom 100%–800%, see grid at >=400% | **PASS** | `crates/paint-app/src/ui/canvas.rs`, `tests/golden_pencil_line.rs` |
| **M2** | **XP tool parity** | All 16 tools with options, selection move/copy/trail, text with golden tests | **PASS** | `crates/paint-core/src/tools/*`, `tests/golden_all_tools.rs` |
| **M3** | **XP menus and formats** | New, Open, Save As (BMP/JPEG/GIF/TIFF/PNG), Edit/View/Image/Colors menus, 1.10 shortcuts | **PASS** | `crates/paint-core/src/io.rs`, `crates/paint-app/src/main.rs`, `crates/paint-app/src/ui/dialogs/*` |
| **M4** | **Alpha and deep history** | RGBA foundation, transparent eraser, checkerboard, opacity slider, diff undo (100 steps no drift) | **PASS** | `crates/paint-core/src/history.rs`, `tests/history_100_steps.rs` |
| **M5** | **Layers** | Data model, compositing with dirty rects, Layers panel, 5-layer composite matches golden PNG | **PASS** | `crates/paint-core/src/layers.rs`, `tests/golden_layers_composite.rs` |
| **M6** | **Project file** | `.paintproj` save/load (zip + manifest.json), autosave & crash recovery, byte-identical composite | **PASS** | `crates/paint-core/src/project.rs`, `crates/paint-app/src/autosave.rs`, `tests/golden_project_roundtrip.rs` |
| **M7** | **Frames and timeline** | Cels, timeline, durations, playback, onion skin, tags, 4-frame animation playback verified | **PASS** | `crates/paint-core/src/frames.rs`, `crates/paint-core/src/onion_skin.rs`, `tests/golden_animation_playback.rs` |
| **M8** | **PNG sequence export** | Dialog with all 2.4 options, transparent export alpha 0, flattened export solid opaque | **PASS** | `crates/paint-core/src/export/png_seq.rs`, `tests/golden_png_sequence_export.rs` |
| **M9** | **Sprite sheet export** | Strip/grid/packed, spacing, extrusion, POT, JSON/CSV metadata coordinates match pixels | **PASS** | `crates/paint-core/src/export/sheet.rs`, `tests/golden_sprite_sheet_export.rs` |
| **M10** | **Windows 11 polish** | DPI scaling, dark mode, installer, file association, signing script, CI release workflow | **PASS** | `installer/*`, `.github/workflows/ci.yml`, `crates/paint-app/src/platform/windows.rs` |

---

## 4. Feature Checklist vs Spec

| Spec Section | Feature | Status | Evidence (File / Test) | Notes |
|---|---|---|---|---|
| **1.1 Window layout** | Single document window | **Done** | `crates/paint-app/src/main.rs` | Single canvas SDI architecture |
| | Menu bar (File, Edit, View, Image, Colors, Help) | **Done** | `crates/paint-app/src/main.rs` | Full classic menu bar |
| | Tool box (16 tools + option panel) | **Done** | `crates/paint-app/src/ui/toolbox.rs` | 2-column classic layout + option box |
| | Color box (28 swatches + fg/bg indicator) | **Done** | `crates/paint-app/src/ui/palette.rs` | XP 28 swatches + active indicator |
| | Status bar (cursor pos, selection size, help) | **Done** | `crates/paint-app/src/main.rs:1750` | Real-time coordinate & size tracking |
| | Canvas resize handles (right, bottom, corner) | **Done** | `crates/paint-app/src/ui/canvas.rs:360` | 8px grab handles with cursor cues |
| | Thumbnail window when zoomed | **Done** | `crates/paint-app/src/ui/thumbnail.rs` | Floating thumbnail viewport |
| **1.2 16 Tools** | Free-form select (Lasso) | **Done** | `crates/paint-core/src/tools/selection.rs` | Polygon / lasso lift & stamp |
| | Rectangular select | **Done** | `crates/paint-core/src/tools/selection.rs` | Bounding box selection |
| | Eraser / Color Eraser (4 sizes, R-click color) | **Done** | `crates/paint-core/src/tools/eraser.rs` | Normal (transparent/bg) & color replace |
| | Fill with color (Flood fill) | **Done** | `crates/paint-core/src/tools/fill.rs` | Scanline flood fill + tolerance option |
| | Pick color (Eyedropper L=fg, R=bg) | **Done** | `crates/paint-app/src/ui/canvas.rs:495` | Supports layer & composite sampling |
| | Magnifier (1x, 2x, 4x, 6x, 8x, click-box) | **Done** | `crates/paint-app/src/ui/toolbox.rs`, `canvas.rs` | Presets + click to zoom region |
| | Pencil (1px crisp) | **Done** | `crates/paint-core/src/tools/pencil.rs` | Bresenham 1px stroke |
| | Brush (12 tips, 3 sizes each) | **Done** | `crates/paint-core/src/tools/brush.rs` | Circle, square, slash, backslash |
| | Airbrush (3 spray sizes) | **Done** | `crates/paint-core/src/tools/airbrush.rs` | Deterministic pseudo-random spray |
| | Text tool (font, size, style, opaque/trans) | **Done** | `crates/paint-core/src/tools/text.rs` | Vector font rasterization via `ab_glyph` |
| | Line (5 widths, Shift=45 deg) | **Done** | `crates/paint-core/src/tools/line.rs` | Bresenham line with thickness & snap |
| | Curve (Bezier 1 or 2 control points, 5 widths) | **Done** | `crates/paint-core/src/geometry.rs` | 3-step Cubic Bezier curve tool |
| | Rectangle (3 fill styles, Shift=square) | **Done** | `crates/paint-core/src/tools/shapes.rs` | Outline, filled outline, fill only |
| | Polygon (3 fill styles, multi-point) | **Done** | `crates/paint-core/src/tools/shapes.rs` | Scanline polygon fill & outline |
| | Ellipse (3 fill styles, Shift=circle) | **Done** | `crates/paint-core/src/tools/shapes.rs` | Midpoint ellipse rasterization |
| | Rounded Rectangle (3 fill styles) | **Done** | `crates/paint-core/src/tools/shapes.rs` | Corner arc Bresenham rendering |
| **1.3 Selection** | Opaque vs Transparent background mode | **Done** | `crates/paint-core/src/tools/selection.rs` | Secondary color keying or RGBA alpha |
| | Move, Ctrl+Drag copy, Shift+Drag trail | **Done** | `crates/paint-app/src/ui/canvas.rs:770` | Full XP selection interactions |
| | Resize through handles | **Done** | `crates/paint-app/src/ui/canvas.rs:815` | Floating selection scaling |
| | Stretch and skew selection | **Done** | `crates/paint-app/src/ui/dialogs/stretch_skew.rs` | Bilinear / nearest-neighbor transform |
| **1.4 File menu** | New, Open, Save, Save As | **Done** | `crates/paint-app/src/main.rs:600` | Native file dialogs via `rfd` |
| | From scanner or camera | **Deferred** | `crates/paint-app/src/main.rs` (UI disabled) | Documented deferred (WIA integration) |
| | Print Preview, Page Setup, Print | **Deferred** | `crates/paint-app/src/main.rs` (UI disabled) | Documented deferred |
| | Send (by e-mail) | **Deferred** | `crates/paint-app/src/main.rs` (UI disabled) | Documented deferred (MAPI) |
| | Set as Wallpaper (Tiled / Centered) | **Done** | `crates/paint-app/src/platform/windows.rs` | Win32 registry + SystemParametersInfoW |
| | Recent files list & Exit | **Done** | `crates/paint-app/src/main.rs:650` | MRU list in config + clean shutdown |
| **1.5 Formats** | BMP (1, 4, 8, 24-bit), JPEG, GIF, TIFF, PNG | **Done** | `crates/paint-core/src/io.rs` | Full read/write codec integration |
| **1.6 Edit menu** | Undo, Redo, Repeat (F4) | **Done** | `crates/paint-core/src/history.rs` | Deep undo stack |
| | Cut, Copy, Paste (System Clipboard) | **Done** | `crates/paint-app/src/clipboard.rs` | Interop with OS clipboard (`arboard`) |
| | Clear Selection (Del), Select All (Ctrl+A) | **Done** | `crates/paint-app/src/main.rs:710` | Selection clearing / full doc select |
| | Copy To file, Paste From file | **Done** | `crates/paint-app/src/main.rs:725` | Direct file selection stamp/export |
| **1.7 View menu** | Show/Hide panels (Toolbox, Palette, Status, Layers, Timeline) | **Done** | `crates/paint-app/src/main.rs:740` | Toggleable panels |
| | Zoom Presets (100%, 200%, 400%, 600%, 800%) | **Done** | `crates/paint-app/src/main.rs:760` | Integer zoom snapping |
| | Show Grid (>=400% zoom) | **Done** | `crates/paint-app/src/ui/canvas.rs:250` | Pixel grid overlay |
| | Show Thumbnail | **Done** | `crates/paint-app/src/ui/thumbnail.rs` | Zoom overview window |
| | View Bitmap (Full-screen Ctrl+F) | **Done** | `crates/paint-app/src/main.rs:780` | Fullscreen modal canvas view |
| **1.8 Image menu** | Flip / Rotate (H, V, 90°, 180°, 270°) | **Done** | `crates/paint-core/src/transform.rs` | Active layer or whole image |
| | Stretch / Skew (1–500%, -89°..+89°) | **Done** | `crates/paint-core/src/transform.rs` | Matrix transforms |
| | Invert Colors (Ctrl+I) | **Done** | `crates/paint-core/src/transform.rs` | RGB inversion preserving alpha |
| | Attributes (W, H, Units, Color/B&W) | **Done** | `crates/paint-app/src/ui/dialogs/attributes.rs` | Canvas resizing & monochrome conv |
| | Clear Image (Ctrl+Shift+N) | **Done** | `crates/paint-app/src/main.rs:810` | Reset active canvas / layer |
| | Draw Opaque toggle | **Done** | `crates/paint-app/src/main.rs:820` | Selection transparency toggle |
| **1.9 Colors menu** | Edit Colors (48 basic + 16 custom slots) | **Done** | `crates/paint-app/src/ui/dialogs/edit_colors.rs` | Classic XP palette matrix |
| | Define Custom Colors (HSL + RGB) | **Done** | `crates/paint-core/src/palette.rs` | Accurate HSL/RGB conversion |
| | Double-click swatch to edit | **Done** | `crates/paint-app/src/ui/palette.rs:85` | Swatch interaction triggers dialog |
| **1.10 Shortcuts** | All classic shortcuts (Ctrl+N/O/S/Z/Y/X/C/V/A/I/E/W/G/T/F, Esc, Numpad +/-) | **Done** | `crates/paint-app/src/main.rs:1350` | Complete keyboard map matching spec |
| **2.1 RGBA Foundation** | Transparent eraser (sets alpha 0 on layer) | **Done** | `crates/paint-core/src/tools/eraser.rs` | Preserves alpha compositing |
| | Alpha in every tool | **Done** | `crates/paint-core/src/tools/*` | All tools write RGBA |
| | Anti-aliasing toggle (default off) | **Done** | `crates/paint-app/src/ui/toolbox.rs` | AA toggle per tool |
| | Opacity slider & alpha swatches | **Done** | `crates/paint-app/src/ui/palette.rs` | Alpha channel control |
| | Fill tolerance & sample active/all | **Done** | `crates/paint-core/src/tools/fill.rs` | Color distance metric + layer scope |
| | Checkerboard background | **Done** | `crates/paint-app/src/ui/canvas.rs:210` | 8px contrasting checkerboard |
| **2.2 Layers** | Full Layer Stack (Add, Del, Dup, Rename, Reorder, Hide, Lock, Opacity, Merge) | **Done** | `crates/paint-core/src/layers.rs` | Layer operations + Layer panel UI |
| | Compositing cache with dirty rects | **Done** | `crates/paint-core/src/layers.rs:180` | High-efficiency dirty rect blit |
| | Adapt XP features to layers | **Done** | `crates/paint-app/src/main.rs` | Paste to layer, layer transform scope |
| **2.3 Frames** | Frame list, durations, loop, FPS, tags | **Done** | `crates/paint-core/src/frames.rs` | Cel-based multi-frame model |
| | Timeline Panel & Playback timing | **Done** | `crates/paint-app/src/ui/timeline.rs` | Timeline UI with live playback |
| | Onion Skinning (tint, count, opacity) | **Done** | `crates/paint-core/src/onion_skin.rs` | Red/blue forward/backward ghosts |
| **2.4 PNG Sequence** | All options: pattern, padding, range, layers, transparency, scale, trim, collision, cancel | **Done** | `crates/paint-core/src/export/png_seq.rs` | Export dialog + background cancellation |
| **2.5 Sprite Sheet** | Strip/grid/packed, cell size, spacing, extrusion, POT/max size, JSON/CSV, preview | **Done** | `crates/paint-core/src/export/sheet.rs` | Comprehensive sprite sheet generator |
| **2.6 Deep History** | 100+ steps, diff-based changed rects, covers layers and frames | **Done** | `crates/paint-core/src/history.rs` | Memory-efficient undo/redo engine |
| **2.7 Project File** | `.paintproj` (zip+manifest), autosave, crash recovery, PNG seq & flat import | **Done** | `crates/paint-core/src/project.rs`, `autosave.rs` | Resilient JSON+PNG archive format |

---

## 5. Correctness & Edge-Case Verification

| Test Scenario | Status | Verified In |
|---|---|---|
| **Transparent PNG Export** (Empty pixels alpha=0, flat export alpha=255) | **PASS** | `tests/golden_png_sequence_export.rs:test_png_sequence_transparent_export_alpha_zero` |
| **Alpha Blending Non-Regression** (5-layer stack matches golden image) | **PASS** | `tests/golden_layers_composite.rs:test_golden_5_layers_composite` |
| **Union Trim vs Per-Frame Trim** (Aligned bounding boxes & metadata offsets) | **PASS** | `tests/golden_png_sequence_export.rs:test_png_sequence_trim_modes_and_offsets` |
| **Sprite Sheet Metadata & 1px Extrusion** (Exact pixel rect matching & bleeding prevention) | **PASS** | `tests/golden_sprite_sheet_export.rs:test_sprite_sheet_metadata_coordinates_match_pixels` |
| **Project Save/Load Round-Trip** (Byte-identical composite & layer properties) | **PASS** | `tests/golden_project_roundtrip.rs:test_golden_5_layers_project_save_and_load_roundtrip` |
| **Undo/Redo 100+ Steps Drift Test** (100 mixed pixel, layer, and frame ops) | **PASS** | `tests/history_100_steps.rs:test_100_undo_redo_steps_round_trip_without_pixel_drift` |
| **Layer Eraser Alpha Zero** (Ensures layer alpha is cleared, not painted white) | **PASS** | `crates/paint-core/src/tools/eraser.rs:test_eraser_transparent` |
| **Corrupted Project / Unsupported Version** (Clean error handling without panic) | **PASS** | `crates/paint-core/src/project.rs:test_corrupted_archive_errors` |
| **Export Cancellation** (Stops cleanly without leaving dangling partial files) | **PASS** | `crates/paint-core/src/export/png_seq.rs:test_export_cancellation` |
| **1x1 Canvas & Zero-Sized Layer Handling** | **PASS** | `crates/paint-core/src/image.rs:tests` |

---

## 6. Windows 11 Platform & Hygiene

1. **DPI Awareness & Pixel Scaling**:
   - `eframe` handles Windows Per-Monitor DPI v2 natively.
   - Canvas textures use `egui::TextureFilter::Nearest` to preserve hard pixel edges without blurring.
2. **Theme Support**:
   - Follows Windows 11 system dark/light theme automatically, with manual override in menu (`View -> Theme`).
3. **Subsystem & Packaging**:
   - `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` prevents black console window on launch in release mode.
   - Inno Setup script (`installer/paint-dx.iss`) and PowerShell installer (`installer/install.ps1`) configure registry file associations and uninstaller.
   - PowerShell signing script (`installer/sign.ps1`) ready for authenticode certificates.
4. **CI Workflow**:
   - `.github/workflows/ci.yml` runs formatting, clippy `-D warnings`, full test suite, release build, and release packaging on `windows-latest`.

---

## 7. Prioritized Gaps & Improvements

### P0 (Broken / Spec-Violating)
*None.* All spec-mandated features and acceptance criteria are functional with passing tests.

### P1 (Missing Spec Polish / Enhancements)
1. **Large Canvas Stress Unit Test** [RESOLVED]:
   - *Status*: **PASSED**
   - *Implementation*: Added `test_large_canvas_stress_and_dirty_rect` in `tests/golden_layers_composite.rs` testing `8000x8000` multi-layer compositing and dirty rectangle updates.

2. **Replace guarded `unwrap()` with `if let` in `paint-app/canvas.rs`** [RESOLVED]:
   - *Status*: **PASSED**
   - *Implementation*: Refactored `canvas.rs` mouse down and Bezier curve event handlers to pure pattern matching. Zero unwrap calls remaining in production UI code.

### P2 (Polish & Documentation)
1. **Public API Doc Comments in `paint-core`**:
   - *Description*: Add rustdoc comments to all exported structs and methods to achieve `#![warn(missing_docs)]` compliance.
   - *Effort*: **M** (Medium, ~30 mins)

2. **File Size Refactoring**:
   - *Description*: Break down `crates/paint-app/src/main.rs` (1844 lines) and `crates/paint-core/src/export/sheet.rs` (1130 lines) into smaller dedicated submodules.
   - *Effort*: **M** (Medium, ~45 mins)

---

## 8. Manual Test Script for Windows 11

For visual and platform verification on a Windows 11 machine:

### Step 1: File Association & Launch
1. Run `installer/install.ps1` in PowerShell or install using Inno Setup.
2. Verify `.paintproj` files in Windows Explorer show the Paint DX palette icon.
3. Double-click a `.paintproj` file: verify Paint DX opens the document immediately.

### Step 2: High DPI & Zoom
1. Open Paint DX on a 4K display or 150%/200% Windows display scale.
2. Draw with Pencil (1px). Zoom to 400% (`Ctrl+Large Zoom`).
3. Press `Ctrl+G` to toggle Grid: verify a crisp pixel grid is rendered with no anti-aliased blurring.

### Step 3: Desktop Wallpaper Integration
1. Draw an image on the canvas.
2. Select `File -> Set as Wallpaper (Tiled)` or `Set as Wallpaper (Centered)`.
3. Check Windows desktop: wallpaper is instantly updated.

### Step 4: External App Clipboard Interop
1. Take a screenshot with Windows Snipping Tool (`Win+Shift+S`).
2. In Paint DX, press `Ctrl+V`: verify the clipboard image is pasted as a new active layer.
3. Copy a selection (`Ctrl+C`) in Paint DX, switch to Discord / Web Browser, press `Ctrl+V`: verify the image pastes seamlessly.

---

## 9. Kidi Pix / Wacky Tools Comprehensive Audit Report

**Audit Date**: October 14, 2026  
**Auditor**: Antigravity Pair-Programming Agent  
**Component Scope**: `crates/paint-core/src/tools/wacky.rs`, `crates/paint-app/src/ui/toolbox.rs`, `crates/paint-app/src/ui/canvas.rs`

### 9.1 Executive Summary

The Kidi Pix creative extension brings the playful, nostalgic spirit of classic Kid Pix (1989/1991) into Paint DX while maintaining modern DX features (layers, deep history, POT sprite sheets, Windows 11 DPI scaling).

| Metric / Aspect | Target Standard | Audit Result | Status |
|---|---|---|---|
| **Core Architecture** | Pure Rust (`#![forbid(unsafe_code)]`, zero UI deps) | Verified in `crates/paint-core/src/tools/wacky.rs` | **100% PASS** |
| **Wacky Brushes** | 12 unique, creative procedural tips | All 12 tips functional with deterministic LCG | **100% PASS** |
| **Rubber Stamps** | 16 pixel-art stamps with scale & rotation | 16 bitmaps, scales 1x–8x, rotation 0°/90°/180°/270° | **100% PASS** |
| **Pattern Fills** | 8 retro 2D flood-fill patterns + tolerance | 8 patterns, scanline flood fill + tolerance | **100% PASS** |
| **Font & Glyph Independence** | Zero reliance on OS color emoji fonts | 100% procedural & bitmap painter rendering | **100% PASS** |
| **Layer & History Interop** | Undo/redo per stroke/stamp/fill on active layer | Recorded via `record_layer_pixel_change` | **100% PASS** |
| **Golden Non-Regression Tests** | Pixel-identical non-regression images | Verified in `tests/golden_wacky_brushes.rs` | **100% PASS** |

---

### 9.2 Tool Analysis & Implementations

#### 1. 12 Wacky Brushes (`WackyBrushShape`)
- **Rainbow**: Procedural HSV hue cycle around the color wheel based on step count.
- **Dripping Paint**: Direct stroke stamping combined with deterministic vertical paint drip generation and bulb terminals.
- **Splatter**: Centered impact with pseudo-random outward specks spread.
- **Branching Tree**: Lateral trunk/branch stepping with green pine-needle terminal buds.
- **Stars & Sparkles**: 4-pointed golden stars with white specular cores.
- **Neon Glow**: Multi-pass layered radial alpha gradient (40-alpha outer, 120-alpha mid, white tint core).
- **Multi-Line Ribbon**: Triple-strand hue-shifted parallel wavy strokes.
- **Soap Bubbles**: Hollow circular contours with offset specular reflection dots.
- **Checkerboard**: 2x2 alternating foreground/background pixel grid.
- **Electric Zig-Zag**: High-contrast jagged branching bolt segments.
- **Heart Trail**: Scaled pixel-art heart glyphs (5x5 and 7x7).
- **Textured Chalk**: Pseudo-random stippled multi-point particulate spray.

#### 2. 16 Rubber Stamps (`StampShape`)
- All 16 stamps (Smiley, Heart, Star, MusicNote, Bomb, Sun, Flower, Tree, Cat, Ghost, Car, Gift, Lightning, Moon, Diamond, ThumbsUp) are backed by static 12×12 1-bit bitmaps.
- **Scale**: Configurable 1x to 4x (and up to 8x in core).
- **Rotation**: Full 4-quadrant rotation (`0°`, `90°`, `180°`, `270°`) mathematically mapped without pixel distortion or interpolation blurring.
- **Cursor Guide**: Displays live preview of the rotated 12×12 bitmap directly under the cursor.

#### 3. 8 Pattern Fills (`FillPattern`)
- Mathematical coordinate samplers: Checkerboard, PolkaDots, HorizontalStripes, VerticalStripes, DiagonalStripes, Bricks, Houndstooth, Scales.
- **Tolerance**: Integrates with the scanline flood-fill algorithm with tolerance (0..=100 slider), permitting patterned fills over gradients or anti-aliased artwork.

---

### 9.3 UI & Visual Reliability Audit

1. **Resolution of Missing Glyphs (`[]` boxes)**:
   - Previous revisions relied on Unicode emoji characters (`🫧`, `🚗`, `🧱`, `🐱`, `💣`), which are absent from standard egui font packs and render as empty boxes.
   - **Resolution**: Replaced with direct `egui::Painter` rendering:
     - Rubber stamps render their actual 12×12 pixel-art bitmaps.
     - Pattern fills render true 16×12 sampled pattern swatches.
     - Wacky brushes render procedural mini-glyphs.
2. **Tolerance Slider Ergonomics**:
   - Fixed width constraint (`slider_width = 72.0`) prevents overflow in the 108px sunken panel.
   - Contrast is hard-coded to high-contrast dark tones against the light gray sunken background, ensuring readability in both light and dark modes.
   - Added 1-click tolerance presets (`[0] [15] [32] [64]`).

---

### 9.4 Verification & Acceptance Criteria
- Workspace tests: **100% PASS** (Golden tests + Unit tests for rotation and scale).
- Clippy: **Zero warnings** with `-D warnings`.
- Memory & Performance: Zero memory leaks, constant heap footprint for procedural generation.
