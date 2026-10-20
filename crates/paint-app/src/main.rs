#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Paint DX - Rust rewrite of classic XP Paint for Windows 11.

mod autosave;
mod clipboard;
mod platform;
mod ui;

use std::path::PathBuf;
use web_time::{Duration, Instant};

use autosave::AutosaveManager;
use eframe::egui;
use paint_core::frames::AnimationDocument;
use paint_core::history::History;
#[cfg(not(target_arch = "wasm32"))]
use paint_core::image::PixelBuffer;
use paint_core::io::{load_from_path, save_to_path, ImageFileFormat};
use paint_core::layers::LayerStack;
use paint_core::palette::{ColorRgba, WHITE};
use paint_core::project::{load_project_from_path, save_project_to_path, ProjectData};
use paint_core::tools::selection::SelectionMode;
use paint_core::tools::{
    BrushShape, EraserSize, FillPattern, FillStyle, FontFamilyChoice, MixerEffect, StampShape,
    ToolKind, WackyBrushShape,
};
use platform::windows::{set_desktop_wallpaper, WallpaperStyle};
use ui::canvas::{CanvasView, SymmetryMode};
use ui::dialogs::attributes::{AttributesDialog, ColorModeChoice};
use ui::dialogs::edit_colors::EditColorsDialog;
use ui::dialogs::export_gif::ExportGifDialog;
use ui::dialogs::export_png_seq::ExportPngSeqDialog;
use ui::dialogs::export_sheet::ExportSheetDialog;
use ui::dialogs::flip_rotate::{FlipRotateAction, FlipRotateDialog};
use ui::dialogs::new_project::NewProjectDialog;
use ui::dialogs::stretch_skew::StretchSkewDialog;
use ui::layers::{LayerAction, LayersView};
use ui::palette::PaletteView;
use ui::thumbnail::ThumbnailView;
use ui::timeline::{TimelineAction, TimelineView};
use ui::toolbox::{handle_hover_scroll, ToolboxMode, ToolboxView};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingFileAction {
    New,
    Open,
    Exit,
    LoadRecent(usize),
}

pub struct PaintApp {
    doc: AnimationDocument,
    layer_stack: LayerStack,
    file_path: Option<PathBuf>,
    history: History,
    canvas: CanvasView,
    toolbox: ToolboxView,
    palette: PaletteView,
    layers: LayersView,
    timeline: TimelineView,
    thumbnail: ThumbnailView,

    // Project & Dirty State
    is_dirty: bool,
    autosave: AutosaveManager,
    recovery_dialog_open: bool,
    recovery_project: Option<ProjectData>,
    confirm_close_dialog_open: bool,
    pending_action: Option<PendingFileAction>,

    // Dialogs
    flip_rotate_dlg: FlipRotateDialog,
    stretch_skew_dlg: StretchSkewDialog,
    attributes_dlg: AttributesDialog,
    edit_colors_dlg: EditColorsDialog,
    export_png_seq_dlg: ExportPngSeqDialog,
    export_sheet_dlg: ExportSheetDialog,
    export_gif_dlg: ExportGifDialog,
    new_project_dlg: NewProjectDialog,
    editing_swatch_idx: Option<usize>,

    // View & Theme toggles
    show_toolbox: bool,
    show_palette: bool,
    show_statusbar: bool,
    show_toolbar: bool,
    is_view_bitmap_fullscreen: bool,
    theme_preference: ThemePreference,

    // Splash screen state
    splash_start: Instant,
    splash_texture: Option<egui::TextureHandle>,
    splash_dismissed: bool,
    splash_sound_played: bool,

    // File state
    recent_files: Vec<PathBuf>,
    about_open: bool,
    help_open: bool,
    mixer_seed: u64,
}

impl Default for PaintApp {
    fn default() -> Self {
        let autosave = AutosaveManager::new();
        let (recovery_dialog_open, recovery_project) = if autosave.has_recoverable_session() {
            if let Ok(proj) = autosave.load_recovery_project() {
                (true, Some(proj))
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };

        let doc = AnimationDocument::new(800, 600, WHITE);
        let layer_stack = doc.to_layer_stack(0);

        Self {
            doc,
            layer_stack,
            file_path: None,
            history: History::new(100),
            canvas: CanvasView::default(),
            toolbox: ToolboxView::default(),
            palette: PaletteView::default(),
            layers: LayersView::default(),
            timeline: TimelineView::default(),
            thumbnail: ThumbnailView::default(),
            is_dirty: false,
            autosave,
            recovery_dialog_open,
            recovery_project,
            confirm_close_dialog_open: false,
            pending_action: None,
            flip_rotate_dlg: FlipRotateDialog::default(),
            stretch_skew_dlg: StretchSkewDialog::default(),
            attributes_dlg: AttributesDialog::default(),
            edit_colors_dlg: EditColorsDialog::default(),
            export_png_seq_dlg: ExportPngSeqDialog::default(),
            export_sheet_dlg: ExportSheetDialog::default(),
            export_gif_dlg: ExportGifDialog::default(),
            new_project_dlg: NewProjectDialog::default(),
            editing_swatch_idx: None,
            show_toolbox: true,
            show_palette: true,
            show_statusbar: true,
            show_toolbar: true,
            is_view_bitmap_fullscreen: false,
            theme_preference: ThemePreference::default(),
            splash_start: Instant::now(),
            splash_texture: None,
            splash_dismissed: false,
            splash_sound_played: false,
            recent_files: Vec::new(),
            about_open: false,
            help_open: false,
            mixer_seed: 1337,
        }
    }
}

impl PaintApp {
    pub fn with_initial_file(initial_file: Option<PathBuf>) -> Self {
        let mut app = Self::default();
        if let Some(path) = initial_file {
            if path.exists() {
                app.load_file(path);
            }
        } else if !app.recovery_dialog_open {
            app.new_project_dlg.open();
        }
        app
    }

    pub fn apply_mixer_effect(&mut self, effect: MixerEffect) {
        let before = self.layer_stack.clone();
        self.canvas
            .apply_mixer_effect(&mut self.layer_stack, effect, self.mixer_seed, false);
        self.mixer_seed = self.mixer_seed.wrapping_add(1013904223);
        self.history
            .record_layer_stack_change(&before, &self.layer_stack);
        self.mark_dirty();
        self.canvas.mark_dirty();
    }

    fn render_splash_screen(&mut self, ctx: &egui::Context) {
        if self.splash_dismissed {
            return;
        }

        // Trigger startup sound once
        if !self.splash_sound_played {
            self.splash_sound_played = true;
            platform::play_startup_sound();
        }

        let elapsed = self.splash_start.elapsed().as_secs_f32();
        if elapsed >= 3.5 {
            self.splash_dismissed = true;
            self.splash_texture = None;
            ctx.request_repaint();
            return;
        }

        // Request continuous repainting during splash and fade-out animation
        ctx.request_repaint();

        // Allow skipping with mouse click or key press
        if ctx.input(|i| {
            i.pointer.any_pressed()
                || i.key_pressed(egui::Key::Escape)
                || i.key_pressed(egui::Key::Space)
                || i.key_pressed(egui::Key::Enter)
        }) {
            self.splash_dismissed = true;
            self.splash_texture = None;
            ctx.request_repaint();
            return;
        }

        let alpha = if elapsed <= 3.0 {
            1.0_f32
        } else {
            ((3.5 - elapsed) / 0.5).clamp(0.0, 1.0)
        };

        if alpha <= 0.001 {
            self.splash_dismissed = true;
            self.splash_texture = None;
            ctx.request_repaint();
            return;
        }

        let tex = self.splash_texture.get_or_insert_with(|| {
            let img = image::load_from_memory(include_bytes!("../assets/splash.png"))
                .expect("Failed to load embedded splash.png")
                .into_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            let pixels = img.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            ctx.load_texture("splash_texture", color_image, egui::TextureOptions::LINEAR)
        });

        let screen_rect = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("splash_layer"),
        ));

        // Center the splash image (transparency reveals the app underneath)
        let tex_size = tex.size_vec2();
        let splash_rect = egui::Rect::from_center_size(screen_rect.center(), tex_size);

        let tint = egui::Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * alpha) as u8);
        painter.image(
            tex.id(),
            splash_rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
            tint,
        );
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
        self.autosave.mark_dirty();
    }

    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
        self.autosave.mark_clean();
    }

    fn request_new(&mut self) {
        if self.is_dirty {
            self.pending_action = Some(PendingFileAction::New);
            self.confirm_close_dialog_open = true;
        } else {
            self.execute_new();
        }
    }

    fn execute_new(&mut self) {
        self.new_project_dlg.open();
    }

    fn create_new_project(&mut self, width: u32, height: u32, fps: u32, bg_color: ColorRgba) {
        self.canvas.clear_selection();
        self.doc = AnimationDocument::new(width, height, bg_color);
        self.doc.fps = fps;
        self.layer_stack = self.doc.to_layer_stack(0);
        self.history.clear();
        self.file_path = None;
        self.mark_clean();
        self.canvas.mark_dirty();
    }

    fn request_open(&mut self) {
        if self.is_dirty {
            self.pending_action = Some(PendingFileAction::Open);
            self.confirm_close_dialog_open = true;
        } else {
            self.open_file_dialog();
        }
    }

    fn request_load_recent(&mut self, idx: usize) {
        if self.is_dirty {
            self.pending_action = Some(PendingFileAction::LoadRecent(idx));
            self.confirm_close_dialog_open = true;
        } else if let Some(path) = self.recent_files.get(idx).cloned() {
            self.load_file(path);
        }
    }

    fn request_exit(&mut self, ctx: &egui::Context) {
        if self.is_dirty {
            self.pending_action = Some(PendingFileAction::Exit);
            self.confirm_close_dialog_open = true;
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn execute_pending_action(&mut self, ctx: &egui::Context, action: PendingFileAction) {
        match action {
            PendingFileAction::New => self.execute_new(),
            PendingFileAction::Open => self.open_file_dialog(),
            PendingFileAction::LoadRecent(idx) => {
                if let Some(path) = self.recent_files.get(idx).cloned() {
                    self.load_file(path);
                }
            }
            PendingFileAction::Exit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "All Supported Files",
                &[
                    "paintproj",
                    "png",
                    "bmp",
                    "dib",
                    "jpg",
                    "jpeg",
                    "gif",
                    "tif",
                    "tiff",
                ],
            )
            .add_filter("Paint DX Project (*.paintproj)", &["paintproj"])
            .add_filter("PNG Image (*.png)", &["png"])
            .add_filter("Bitmap Image (*.bmp)", &["bmp", "dib"])
            .add_filter("JPEG Image (*.jpg, *.jpeg)", &["jpg", "jpeg", "jfif"])
            .add_filter("GIF Image (*.gif)", &["gif"])
            .add_filter("TIFF Image (*.tif, *.tiff)", &["tif", "tiff"])
            .pick_file()
        {
            self.load_file(path);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn open_file_dialog(&mut self) {}

    fn load_file(&mut self, path: PathBuf) {
        let is_paintproj = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("paintproj"))
            .unwrap_or(false);

        if is_paintproj {
            match load_project_from_path(&path) {
                Ok(proj) => {
                    self.canvas.clear_selection();
                    self.history.clear();
                    self.doc = proj.doc;
                    self.layer_stack = self.doc.to_layer_stack(self.doc.active_frame_index);
                    if let Some(palette) = proj.palette {
                        self.palette.swatches = palette;
                    }
                    self.file_path = Some(path.clone());
                    self.mark_clean();
                    if !self.recent_files.contains(&path) {
                        self.recent_files.insert(0, path);
                        self.recent_files.truncate(8);
                    }
                    self.canvas.mark_dirty();
                }
                Err(e) => {
                    eprintln!("Error loading .paintproj project: {}", e);
                }
            }
        } else {
            match load_from_path(&path) {
                Ok(loaded_buf) => {
                    self.canvas.clear_selection();
                    self.history.clear();
                    let mut stack = LayerStack::new(loaded_buf.width(), loaded_buf.height(), WHITE);
                    if let Some(layer) = stack.active_layer_mut() {
                        layer.buffer = loaded_buf;
                    }
                    self.doc = AnimationDocument::from_layer_stack(stack.clone(), 12);
                    self.layer_stack = stack;
                    self.file_path = Some(path.clone());
                    self.mark_clean();
                    if !self.recent_files.contains(&path) {
                        self.recent_files.insert(0, path);
                        self.recent_files.truncate(8);
                    }
                    self.canvas.mark_dirty();
                }
                Err(e) => {
                    eprintln!("Error loading image: {}", e);
                }
            }
        }
    }

    fn save_file(&mut self) {
        if let Some(path) = self.file_path.clone() {
            let is_paintproj = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("paintproj"))
                .unwrap_or(false);

            if is_paintproj {
                self.doc
                    .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
                let project = ProjectData::from_doc(self.doc.clone(), Some(self.palette.swatches));
                if let Err(e) = save_project_to_path(&project, &path) {
                    eprintln!("Error saving .paintproj project: {}", e);
                } else {
                    self.mark_clean();
                }
            } else {
                let format = ImageFileFormat::from_path(&path).unwrap_or(ImageFileFormat::Png);
                let composite = self.layer_stack.composite();
                if let Err(e) = save_to_path(&composite, &path, format) {
                    eprintln!("Error saving image: {}", e);
                } else {
                    self.mark_clean();
                }
            }
        } else {
            self.save_file_as_dialog();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_file_as_dialog(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("Paint DX Project (*.paintproj)", &["paintproj"])
            .add_filter("PNG Image (*.png)", &["png"])
            .add_filter("Bitmap Image (*.bmp)", &["bmp"])
            .add_filter("JPEG Image (*.jpg, *.jpeg)", &["jpg", "jpeg"])
            .add_filter("GIF Image (*.gif)", &["gif"])
            .add_filter("TIFF Image (*.tif, *.tiff)", &["tif", "tiff"]);

        if let Some(ref path) = self.file_path {
            if let Some(parent) = path.parent() {
                dialog = dialog.set_directory(parent);
            }
        }

        if let Some(path) = dialog.save_file() {
            let is_paintproj = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("paintproj"))
                .unwrap_or(false);

            if is_paintproj {
                self.doc
                    .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
                let project = ProjectData::from_doc(self.doc.clone(), Some(self.palette.swatches));
                if let Err(e) = save_project_to_path(&project, &path) {
                    eprintln!("Error saving .paintproj project: {}", e);
                } else {
                    self.file_path = Some(path.clone());
                    self.mark_clean();
                    if !self.recent_files.contains(&path) {
                        self.recent_files.insert(0, path);
                        self.recent_files.truncate(8);
                    }
                }
            } else {
                let format = ImageFileFormat::from_path(&path).unwrap_or(ImageFileFormat::Png);
                let composite = self.layer_stack.composite();
                if let Err(e) = save_to_path(&composite, &path, format) {
                    eprintln!("Error saving image: {}", e);
                } else {
                    self.file_path = Some(path.clone());
                    self.mark_clean();
                    if !self.recent_files.contains(&path) {
                        self.recent_files.insert(0, path);
                        self.recent_files.truncate(8);
                    }
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn save_file_as_dialog(&mut self) {
        let composite = self.layer_stack.composite();
        if let Ok(bytes) = paint_core::io::save_to_memory(&composite, ImageFileFormat::Png) {
            download_bytes("drawing.png", &bytes);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn copy_to_file_dialog(&mut self) {
        let snippet = self.canvas.copy_selection_or_canvas(&self.layer_stack);
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG Image (*.png)", &["png"])
            .add_filter("Bitmap Image (*.bmp)", &["bmp"])
            .save_file()
        {
            let format = ImageFileFormat::from_path(&path).unwrap_or(ImageFileFormat::Png);
            let _ = save_to_path(&snippet, path, format);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn copy_to_file_dialog(&mut self) {
        let snippet = self.canvas.copy_selection_or_canvas(&self.layer_stack);
        if let Ok(bytes) = paint_core::io::save_to_memory(&snippet, ImageFileFormat::Png) {
            download_bytes("selection.png", &bytes);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn paste_from_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "All Supported Images",
                &["png", "bmp", "jpg", "jpeg", "gif", "tif", "tiff"],
            )
            .pick_file()
        {
            if let Ok(loaded_buf) = load_from_path(&path) {
                let before = self.layer_stack.clone();
                self.canvas.paste_as_selection(
                    &mut self.layer_stack,
                    loaded_buf,
                    &self.toolbox,
                    self.palette.bg_color,
                );
                self.history
                    .record_layer_stack_change(&before, &self.layer_stack);
                self.mark_dirty();
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn paste_from_file_dialog(&mut self) {}

    #[cfg(not(target_arch = "wasm32"))]
    pub fn import_image_as_layer_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "All Supported Images",
                &["png", "bmp", "jpg", "jpeg", "gif", "tif", "tiff"],
            )
            .pick_file()
        {
            if let Ok(loaded_buf) = load_from_path(&path) {
                let before = self.layer_stack.clone();
                let layer_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Imported Layer")
                    .to_string();

                let (doc_w, doc_h) = (self.layer_stack.width, self.layer_stack.height);
                let mut layer_buf = PixelBuffer::new(doc_w, doc_h, ColorRgba::TRANSPARENT);

                let src_w = loaded_buf.width();
                let src_h = loaded_buf.height();
                let off_x = ((doc_w as i32 - src_w as i32) / 2).max(0);
                let off_y = ((doc_h as i32 - src_h as i32) / 2).max(0);

                for y in 0..src_h {
                    for x in 0..src_w {
                        let dst_x = off_x + x as i32;
                        let dst_y = off_y + y as i32;
                        if dst_x >= 0 && dst_x < doc_w as i32 && dst_y >= 0 && dst_y < doc_h as i32
                        {
                            if let Some(px) = loaded_buf.get_pixel(x, y) {
                                layer_buf.set_pixel(dst_x as u32, dst_y as u32, px);
                            }
                        }
                    }
                }

                let new_id = self.layer_stack.add_layer(layer_name);
                if let Some(layer) = self.layer_stack.get_layer_mut(new_id) {
                    layer.buffer = layer_buf;
                }

                self.history
                    .record_layer_stack_change(&before, &self.layer_stack);
                self.doc
                    .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
                self.mark_dirty();
                self.canvas.mark_dirty();
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn import_image_as_layer_dialog(&mut self) {}

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // Fullscreen toggle (Ctrl+F)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::F,
            ))
        }) {
            self.is_view_bitmap_fullscreen = !self.is_view_bitmap_fullscreen;
        }

        // Exit fullscreen on Esc
        if self.is_view_bitmap_fullscreen && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.is_view_bitmap_fullscreen = false;
        }

        if self.is_view_bitmap_fullscreen {
            return;
        }

        // New Document (Ctrl+N)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::N,
            ))
        }) {
            self.request_new();
        }

        // Toggle Tool Options Toolbar (Ctrl+T)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::T,
            ))
        }) {
            self.show_toolbar = !self.show_toolbar;
        }

        // New Layer (Ctrl+Shift+N)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::N,
            ))
        }) {
            let before = self.layer_stack.clone();
            let count = self.layer_stack.layers.len() + 1;
            self.layer_stack.add_layer(format!("Layer {}", count));
            self.history
                .record_layer_stack_change(&before, &self.layer_stack);
            self.mark_dirty();
            self.canvas.mark_dirty();
        }

        // Import Image as Layer (Ctrl+Shift+I)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::I,
            ))
        }) {
            self.import_image_as_layer_dialog();
        }

        // Merge Down (Ctrl+E)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::E,
            ))
        }) {
            let before = self.layer_stack.clone();
            if self
                .layer_stack
                .merge_down(self.layer_stack.active_layer_id)
            {
                self.history
                    .record_layer_stack_change(&before, &self.layer_stack);
                self.mark_dirty();
                self.canvas.mark_dirty();
            }
        }

        // Open (Ctrl+O)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::O,
            ))
        }) {
            self.request_open();
        }

        // Save (Ctrl+S)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::S,
            ))
        }) {
            self.save_file();
        }

        // Export PNG Sequence (Ctrl+Shift+E)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::E,
            ))
        }) {
            self.export_png_seq_dlg
                .open(self.file_path.as_deref(), &self.doc);
        }

        // Export Sprite Sheet (Ctrl+Shift+X)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::X,
            ))
        }) {
            self.export_sheet_dlg
                .open(self.file_path.as_deref(), &self.doc);
        }

        // Export Animated GIF (Ctrl+Shift+G)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::G,
            ))
        }) {
            self.doc
                .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
            self.export_gif_dlg
                .open(self.file_path.as_deref(), &self.doc);
        }

        // Center Canvas (Ctrl+0)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Num0,
            ))
        }) {
            self.canvas.zoom = 1.0;
            self.canvas.mark_dirty();
        }

        // Zoom 100% (Ctrl+PageUp)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::PageUp,
            ))
        }) {
            self.canvas.set_zoom(1.0);
        }

        // Zoom 400% (Ctrl+PageDown)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::PageDown,
            ))
        }) {
            self.canvas.set_zoom(4.0);
        }

        // Zoom In (Ctrl+= or Ctrl+Plus)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Plus,
            )) || i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Equals,
            ))
        }) {
            let zoom_levels: [f32; 13] = [
                0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0,
            ];
            let current_idx = zoom_levels
                .iter()
                .enumerate()
                .min_by(|(_, &a), (_, &b)| {
                    (a - self.canvas.zoom)
                        .abs()
                        .partial_cmp(&(b - self.canvas.zoom).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
                .unwrap_or(1);
            if current_idx + 1 < zoom_levels.len() {
                self.canvas.set_zoom(zoom_levels[current_idx + 1]);
            }
        }

        // Zoom Out (Ctrl+Minus)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Minus,
            ))
        }) {
            let zoom_levels: [f32; 13] = [
                0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0,
            ];
            let current_idx = zoom_levels
                .iter()
                .enumerate()
                .min_by(|(_, &a), (_, &b)| {
                    (a - self.canvas.zoom)
                        .abs()
                        .partial_cmp(&(b - self.canvas.zoom).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
                .unwrap_or(1);
            if current_idx > 0 {
                self.canvas.set_zoom(zoom_levels[current_idx - 1]);
            }
        }

        // Apply any pending Electric Mixer action from toolbox or canvas click
        if let Some(effect) = self.toolbox.pending_mixer_effect.take() {
            self.apply_mixer_effect(effect);
        }

        // Electric Mixer random effect (Ctrl+M)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::M,
            ))
        }) {
            let rand_idx = (self.mixer_seed as usize) % MixerEffect::ALL.len();
            let effect = MixerEffect::ALL[rand_idx];
            self.apply_mixer_effect(effect);
        }

        // Grid (Ctrl+G)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::G,
            ))
        }) {
            self.canvas.show_grid = !self.canvas.show_grid;
        }

        // Toolbox toggle (Ctrl+T)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::T,
            ))
        }) {
            self.show_toolbox = !self.show_toolbox;
        }

        // Palette toggle (Ctrl+L)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::L,
            ))
        }) {
            self.show_palette = !self.show_palette;
        }

        // Invert (Ctrl+I)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::I,
            ))
        }) {
            let before = self.layer_stack.clone();
            self.canvas
                .apply_invert_colors(&mut self.layer_stack, false);
            self.history
                .record_layer_stack_change(&before, &self.layer_stack);
            self.mark_dirty();
        }

        // Stretch/Skew (Ctrl+W)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::W,
            ))
        }) {
            self.stretch_skew_dlg.open();
        }

        // Flip/Rotate (Ctrl+R)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::R,
            ))
        }) {
            self.flip_rotate_dlg.open();
        }

        // Cut (Ctrl+X)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::X,
            ))
        }) {
            let before = self.layer_stack.clone();
            let cut_buf = self
                .canvas
                .cut_selection_or_canvas(&mut self.layer_stack, self.palette.bg_color);
            self.history
                .record_layer_stack_change(&before, &self.layer_stack);
            self.mark_dirty();
            let _ = clipboard::copy_to_clipboard(&cut_buf);
        }

        // Copy (Ctrl+C)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::C,
            ))
        }) {
            let copy_buf = self.canvas.copy_selection_or_canvas(&self.layer_stack);
            let _ = clipboard::copy_to_clipboard(&copy_buf);
        }

        // Paste (Ctrl+V)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::V,
            ))
        }) {
            if let Ok(pasted_buf) = clipboard::paste_from_clipboard() {
                let before = self.layer_stack.clone();
                self.canvas.paste_as_selection(
                    &mut self.layer_stack,
                    pasted_buf,
                    &self.toolbox,
                    self.palette.bg_color,
                );
                self.history
                    .record_layer_stack_change(&before, &self.layer_stack);
                self.mark_dirty();
            }
        }

        // Select All (Ctrl+A)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::A,
            ))
        }) {
            let before = self.layer_stack.clone();
            self.canvas
                .select_all(&mut self.layer_stack, self.palette.bg_color);
            self.history
                .record_layer_stack_change(&before, &self.layer_stack);
            self.mark_dirty();
        }

        // Undo (Ctrl+Z)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Z,
            ))
        }) && self.history.undo_layer_stack(&mut self.layer_stack)
        {
            self.canvas.clear_selection();
            self.mark_dirty();
            self.canvas.mark_dirty();
        }

        // Redo / Repeat (Ctrl+Y or F4)
        let is_redo = ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Y,
            ))
        }) || ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::F4,
            ))
        });
        if is_redo && self.history.redo_layer_stack(&mut self.layer_stack) {
            self.canvas.clear_selection();
            self.mark_dirty();
            self.canvas.mark_dirty();
        }

        // Zoom 100% (Ctrl+PageUp)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::PageUp,
            ))
        }) {
            self.canvas.set_zoom(1.0);
        }

        // Zoom 400% (Ctrl+PageDown)
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::PageDown,
            ))
        }) {
            self.canvas.set_zoom(4.0);
        }

        // Delete key (Clear Selection)
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) && self.canvas.has_floating_selection() {
            let before = self.layer_stack.clone();
            self.canvas.clear_selection();
            self.history
                .record_layer_stack_change(&before, &self.layer_stack);
            self.mark_dirty();
        }

        // Esc key (Commit selection or cancel tool)
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.canvas.commit_any_selection(
                &mut self.layer_stack,
                &self.toolbox,
                self.palette.bg_color,
            );
        }

        // Tool size increase/decrease shortcuts (Ctrl + Plus / Ctrl + Equals, Ctrl + Minus)
        let is_size_inc = ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Plus,
            )) || i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Equals,
            ))
        });
        if is_size_inc {
            self.toolbox.increase_tool_size();
            self.canvas.mark_dirty();
        }

        let is_size_dec = ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Minus,
            ))
        });
        if is_size_dec {
            self.toolbox.decrease_tool_size();
            self.canvas.mark_dirty();
        }

        // Animation & Tool Shortcuts
        let is_text_focused = self.canvas.active_text_pos.is_some() || ctx.wants_keyboard_input();
        if !is_text_focused {
            // Bracket shortcuts [ and ] for fine-grained size adjustment
            let shift = ctx.input(|i| i.modifiers.shift);
            let delta = if shift { 5 } else { 1 };

            if ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::OpenBracket)
                    || i.consume_key(egui::Modifiers::SHIFT, egui::Key::OpenBracket)
            }) {
                if self.toolbox.active_tool == ToolKind::WackyBrush {
                    self.toolbox.wacky_brush_size =
                        self.toolbox.wacky_brush_size.saturating_sub(delta).max(1);
                    self.canvas.mark_dirty();
                } else if self.toolbox.active_tool == ToolKind::Stamp {
                    self.toolbox.stamp_scale =
                        self.toolbox.stamp_scale.saturating_sub(delta).max(1);
                    self.canvas.mark_dirty();
                } else if self.toolbox.active_tool == ToolKind::PatternFill {
                    self.toolbox.pattern_scale =
                        self.toolbox.pattern_scale.saturating_sub(delta).max(1);
                    self.canvas.mark_dirty();
                } else {
                    self.toolbox.decrease_tool_size();
                    self.canvas.mark_dirty();
                }
            }

            if ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::CloseBracket)
                    || i.consume_key(egui::Modifiers::SHIFT, egui::Key::CloseBracket)
            }) {
                if self.toolbox.active_tool == ToolKind::WackyBrush {
                    self.toolbox.wacky_brush_size =
                        (self.toolbox.wacky_brush_size + delta).min(128);
                    self.canvas.mark_dirty();
                } else if self.toolbox.active_tool == ToolKind::Stamp {
                    self.toolbox.stamp_scale = (self.toolbox.stamp_scale + delta).min(16);
                    self.canvas.mark_dirty();
                } else if self.toolbox.active_tool == ToolKind::PatternFill {
                    self.toolbox.pattern_scale = (self.toolbox.pattern_scale + delta).min(8);
                    self.canvas.mark_dirty();
                } else {
                    self.toolbox.increase_tool_size();
                    self.canvas.mark_dirty();
                }
            }

            // Shift+Wheel: Rotate stamp angle or adjust wacky brush angle / spacing
            // Alt+Wheel: Adjust density or fine scale
            let (shift_down, alt_down, scroll_y) =
                ctx.input(|i| (i.modifiers.shift, i.modifiers.alt, i.raw_scroll_delta.y));
            if scroll_y.abs() > 0.0 {
                let sign = if scroll_y > 0.0 { 1.0_f32 } else { -1.0_f32 };
                if shift_down {
                    if self.toolbox.active_tool == ToolKind::Stamp {
                        self.toolbox.stamp_free_rotation =
                            (self.toolbox.stamp_free_rotation + sign * 15.0).rem_euclid(360.0);
                        self.canvas.mark_dirty();
                    } else if self.toolbox.active_tool == ToolKind::WackyBrush {
                        if !self.toolbox.wacky_auto_tangent {
                            self.toolbox.wacky_angle =
                                (self.toolbox.wacky_angle + sign * 15.0).rem_euclid(360.0);
                        } else {
                            self.toolbox.wacky_scatter_radius =
                                (self.toolbox.wacky_scatter_radius as f32 + sign * 4.0)
                                    .clamp(0.0, 100.0) as u32;
                        }
                        self.canvas.mark_dirty();
                    }
                } else if alt_down {
                    if self.toolbox.active_tool == ToolKind::WackyBrush {
                        self.toolbox.wacky_density =
                            (self.toolbox.wacky_density + sign * 0.25).clamp(0.25, 4.0);
                        self.canvas.mark_dirty();
                    } else if self.toolbox.active_tool == ToolKind::Stamp {
                        self.toolbox.stamp_fine_scale =
                            (self.toolbox.stamp_fine_scale + sign * 0.25).clamp(0.25, 4.0);
                        self.canvas.mark_dirty();
                    }
                }
            }

            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                self.handle_timeline_action(TimelineAction::TogglePlay);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Comma)) {
                self.handle_timeline_action(TimelineAction::PrevFrame);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Period)) {
                self.handle_timeline_action(TimelineAction::NextFrame);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Home)) {
                self.handle_timeline_action(TimelineAction::FirstFrame);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::End)) {
                self.handle_timeline_action(TimelineAction::LastFrame);
            }
        }
    }

    pub fn handle_timeline_action(&mut self, action: TimelineAction) {
        match action {
            TimelineAction::SelectFrame(idx) => {
                if idx < self.doc.frames.len() && idx != self.doc.active_frame_index {
                    self.canvas.commit_any_selection(
                        &mut self.layer_stack,
                        &self.toolbox,
                        self.palette.bg_color,
                    );
                    self.doc
                        .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
                    self.doc.active_frame_index = idx;
                    self.layer_stack = self.doc.to_layer_stack(idx);
                    self.canvas.mark_dirty();
                }
            }
            TimelineAction::SelectLayer(id) => {
                self.layer_stack.set_active_layer(id);
                self.doc.active_layer_id = id;
                self.canvas.mark_dirty();
            }
            TimelineAction::AddFrame { copy_from_prev } => {
                self.canvas.commit_any_selection(
                    &mut self.layer_stack,
                    &self.toolbox,
                    self.palette.bg_color,
                );
                self.doc
                    .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
                let duration = self
                    .doc
                    .frames
                    .get(self.doc.active_frame_index)
                    .map(|f| f.duration_ms)
                    .unwrap_or(83);
                let new_idx = self.doc.add_frame(duration, copy_from_prev);
                self.doc.active_frame_index = new_idx;
                self.layer_stack = self.doc.to_layer_stack(new_idx);
                self.mark_dirty();
                self.canvas.mark_dirty();
            }
            TimelineAction::DuplicateFrame(idx) => {
                self.canvas.commit_any_selection(
                    &mut self.layer_stack,
                    &self.toolbox,
                    self.palette.bg_color,
                );
                self.doc
                    .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
                let new_idx = self.doc.duplicate_frame(idx);
                self.doc.active_frame_index = new_idx;
                self.layer_stack = self.doc.to_layer_stack(new_idx);
                self.mark_dirty();
                self.canvas.mark_dirty();
            }
            TimelineAction::DeleteFrame(idx) => {
                if self.doc.frames.len() > 1 {
                    self.canvas.commit_any_selection(
                        &mut self.layer_stack,
                        &self.toolbox,
                        self.palette.bg_color,
                    );
                    let _ = self.doc.remove_frame(idx);
                    let new_idx = self.doc.active_frame_index.min(self.doc.frames.len() - 1);
                    self.doc.active_frame_index = new_idx;
                    self.layer_stack = self.doc.to_layer_stack(new_idx);
                    self.mark_dirty();
                    self.canvas.mark_dirty();
                }
            }
            TimelineAction::MoveFrame { from, to } => {
                let _ = self.doc.reorder_frame(from, to);
                self.layer_stack = self.doc.to_layer_stack(self.doc.active_frame_index);
                self.mark_dirty();
                self.canvas.mark_dirty();
            }
            TimelineAction::SetFrameDuration { frame, duration_ms } => {
                self.doc.set_frame_duration(frame, duration_ms);
                self.mark_dirty();
            }
            TimelineAction::SetAllDurations(duration_ms) => {
                self.doc.set_all_durations(duration_ms);
                self.mark_dirty();
            }
            TimelineAction::SetFps(fps) => {
                self.doc.set_fps(fps);
                self.mark_dirty();
            }
            TimelineAction::TogglePlay => {
                self.timeline.is_playing = !self.timeline.is_playing;
                self.timeline.last_tick = Some(Instant::now());
                self.timeline.playback_accum_ms = 0.0;
            }
            TimelineAction::Stop => {
                self.timeline.is_playing = false;
                self.handle_timeline_action(TimelineAction::SelectFrame(0));
            }
            TimelineAction::FirstFrame => {
                self.handle_timeline_action(TimelineAction::SelectFrame(0));
            }
            TimelineAction::PrevFrame => {
                let total = self.doc.frames.len();
                if total > 0 {
                    let prev = if self.doc.active_frame_index > 0 {
                        self.doc.active_frame_index - 1
                    } else if self.timeline.loop_playback {
                        total - 1
                    } else {
                        0
                    };
                    self.handle_timeline_action(TimelineAction::SelectFrame(prev));
                }
            }
            TimelineAction::NextFrame => {
                let total = self.doc.frames.len();
                if total > 0 {
                    let next = if self.doc.active_frame_index + 1 < total {
                        self.doc.active_frame_index + 1
                    } else if self.timeline.loop_playback {
                        0
                    } else {
                        total - 1
                    };
                    self.handle_timeline_action(TimelineAction::SelectFrame(next));
                }
            }
            TimelineAction::LastFrame => {
                let total = self.doc.frames.len();
                if total > 0 {
                    self.handle_timeline_action(TimelineAction::SelectFrame(total - 1));
                }
            }
            TimelineAction::AddTag(tag) => {
                self.doc.add_tag(tag);
                self.mark_dirty();
            }
            TimelineAction::DeleteTag(idx) => {
                self.doc.remove_tag(idx);
                self.mark_dirty();
            }
            TimelineAction::SelectTag(tag_name) => {
                self.timeline.active_tag = tag_name;
            }
            TimelineAction::ClearCel { frame, layer_id } => {
                if let Some(cel) = self.doc.get_cel_mut(frame, layer_id) {
                    cel.clear();
                    if frame == self.doc.active_frame_index {
                        self.layer_stack = self.doc.to_layer_stack(frame);
                    }
                    self.mark_dirty();
                    self.canvas.mark_dirty();
                }
            }
            TimelineAction::ExportSequence => {
                self.export_png_seq_dlg
                    .open(self.file_path.as_deref(), &self.doc);
            }
            TimelineAction::ExportSpriteSheet => {
                self.export_sheet_dlg
                    .open(self.file_path.as_deref(), &self.doc);
            }
            TimelineAction::ExportGif => {
                self.doc
                    .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
                self.export_gif_dlg
                    .open(self.file_path.as_deref(), &self.doc);
            }
        }
    }

    fn render_tool_options_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let (icon, name) = match self.toolbox.active_tool {
                ToolKind::Pencil => ("✏", "Pencil"),
                ToolKind::Brush => ("🖌", "Brush"),
                ToolKind::Airbrush => ("💨", "Airbrush"),
                ToolKind::Eraser => ("⌫", "Eraser"),
                ToolKind::Fill => ("🪣", "Fill"),
                ToolKind::PickColor => ("💧", "Color Picker"),
                ToolKind::Magnifier => ("🔍", "Magnifier"),
                ToolKind::Line => ("╱", "Line"),
                ToolKind::Curve => ("∿", "Curve"),
                ToolKind::Rectangle => ("▭", "Rectangle"),
                ToolKind::Polygon => ("⬡", "Polygon"),
                ToolKind::Ellipse => ("◯", "Ellipse"),
                ToolKind::RoundedRectangle => ("▢", "Rounded Rect"),
                ToolKind::Select => ("⬚", "Select"),
                ToolKind::FreeformSelect => ("✂", "Free Select"),
                ToolKind::Move => ("✋", "Move Layer"),
                ToolKind::Text => ("A", "Text Tool"),
                ToolKind::WackyBrush => ("🪄", "Wacky Brush"),
                ToolKind::Stamp => ("🌟", "Rubber Stamp"),
                ToolKind::PatternFill => ("🪣", "Pattern Fill"),
                ToolKind::ElectricMixer => ("⚡", "Electric Mixer"),
            };
            ui.label(
                egui::RichText::new(format!("{} {}", icon, name))
                    .strong()
                    .size(12.0),
            );
            ui.separator();

            match self.toolbox.active_tool {
                ToolKind::Pencil => {
                    ui.label(egui::RichText::new("Pencil Size:").size(11.0));
                    let resp = ui.add(
                        egui::DragValue::new(&mut self.toolbox.pencil_size)
                            .range(1..=64)
                            .suffix(" px"),
                    );
                    handle_hover_scroll(ui, &resp, &mut self.toolbox.pencil_size, 1, 1, 64);
                    for &s in &[1, 2, 3, 4, 8, 16] {
                        if ui
                            .selectable_label(self.toolbox.pencil_size == s, format!("{}px", s))
                            .clicked()
                        {
                            self.toolbox.pencil_size = s;
                        }
                    }
                }
                ToolKind::Text => {
                    ui.label(egui::RichText::new("Font:").size(11.0));
                    egui::ComboBox::from_id_salt("toolbar_font_family")
                        .selected_text(self.toolbox.font_family.display_name())
                        .show_ui(ui, |ui| {
                            for fam in FontFamilyChoice::ALL {
                                if ui
                                    .selectable_value(
                                        &mut self.toolbox.font_family,
                                        fam,
                                        fam.display_name(),
                                    )
                                    .clicked()
                                {
                                    self.canvas.text_font_family = fam;
                                    self.canvas.mark_dirty();
                                }
                            }
                        });

                    ui.label(egui::RichText::new("Size:").size(11.0));
                    let resp = ui.add(
                        egui::DragValue::new(&mut self.toolbox.font_size)
                            .range(8..=128)
                            .suffix(" pt"),
                    );
                    handle_hover_scroll(ui, &resp, &mut self.toolbox.font_size, 1, 8, 128);
                    if resp.changed() {
                        self.canvas.text_font_size = self.toolbox.font_size;
                        self.canvas.mark_dirty();
                    }
                    for &s in &[12, 16, 24, 32, 48] {
                        if ui
                            .selectable_label(self.toolbox.font_size == s, format!("{}", s))
                            .clicked()
                        {
                            self.toolbox.font_size = s;
                            self.canvas.text_font_size = s;
                            self.canvas.mark_dirty();
                        }
                    }
                    ui.separator();

                    ui.label(egui::RichText::new("Rotation:").size(11.0));
                    for &deg in &[0, 90, 180, 270] {
                        if ui
                            .selectable_label(
                                self.toolbox.text_rotation == deg,
                                format!("{}°", deg),
                            )
                            .clicked()
                        {
                            self.toolbox.text_rotation = deg;
                            self.canvas.text_rotation = deg;
                            self.canvas.mark_dirty();
                        }
                    }
                    let rot_resp = ui.button("⟳ +90°");
                    if rot_resp.clicked() {
                        self.toolbox.text_rotation = (self.toolbox.text_rotation + 90) % 360;
                        self.canvas.text_rotation = self.toolbox.text_rotation;
                        self.canvas.mark_dirty();
                    }
                    handle_hover_scroll(ui, &rot_resp, &mut self.toolbox.text_rotation, 90, 0, 360);
                    ui.separator();

                    let is_opaque = self.toolbox.selection_mode == SelectionMode::Opaque;
                    if ui.selectable_label(!is_opaque, "Transparent").clicked() {
                        self.toolbox.selection_mode = SelectionMode::Transparent;
                        self.canvas.text_is_opaque = false;
                        self.canvas.mark_dirty();
                    }
                    if ui.selectable_label(is_opaque, "Opaque").clicked() {
                        self.toolbox.selection_mode = SelectionMode::Opaque;
                        self.canvas.text_is_opaque = true;
                        self.canvas.mark_dirty();
                    }

                    if self.canvas.active_text_pos.is_some() {
                        ui.separator();
                        if ui
                            .button(
                                egui::RichText::new("✓ Apply Text (Enter)")
                                    .color(egui::Color32::from_rgb(0, 140, 0)),
                            )
                            .clicked()
                        {
                            self.canvas.commit_any_text(
                                &mut self.layer_stack,
                                &mut self.history,
                                &self.toolbox,
                                self.palette.fg_color,
                                self.palette.bg_color,
                            );
                            self.mark_dirty();
                        }
                        if ui
                            .button(
                                egui::RichText::new("✗ Cancel (Esc)")
                                    .color(egui::Color32::from_rgb(180, 0, 0)),
                            )
                            .clicked()
                        {
                            self.canvas.active_text_pos = None;
                        }
                    }
                }
                ToolKind::Stamp => {
                    ui.label(egui::RichText::new("Stamp:").size(11.0));
                    egui::ComboBox::from_id_salt("toolbar_stamp_shape")
                        .selected_text(self.toolbox.stamp_shape.display_name())
                        .show_ui(ui, |ui| {
                            for shape in StampShape::ALL {
                                if ui
                                    .selectable_value(
                                        &mut self.toolbox.stamp_shape,
                                        shape,
                                        shape.display_name(),
                                    )
                                    .clicked()
                                {
                                    self.canvas.mark_dirty();
                                }
                            }
                        });

                    ui.separator();
                    ui.label(egui::RichText::new("Scale:").size(11.0));
                    let scale_resp =
                        ui.add(egui::Slider::new(&mut self.toolbox.stamp_scale, 1..=8).suffix("x"));
                    handle_hover_scroll(ui, &scale_resp, &mut self.toolbox.stamp_scale, 1, 1, 8);
                    for &s in &[1, 2, 3, 4] {
                        if ui
                            .selectable_label(self.toolbox.stamp_scale == s, format!("{}x", s))
                            .clicked()
                        {
                            self.toolbox.stamp_scale = s;
                        }
                    }
                    ui.separator();

                    ui.label(egui::RichText::new("Rotation:").size(11.0));
                    for &deg in &[0, 90, 180, 270] {
                        if ui
                            .selectable_label(
                                self.toolbox.stamp_rotation == deg,
                                format!("{}°", deg),
                            )
                            .clicked()
                        {
                            self.toolbox.stamp_rotation = deg;
                        }
                    }
                    let rot_resp = ui.button("⟳ +90°");
                    if rot_resp.clicked() {
                        self.toolbox.stamp_rotation = (self.toolbox.stamp_rotation + 90) % 360;
                    }
                    handle_hover_scroll(
                        ui,
                        &rot_resp,
                        &mut self.toolbox.stamp_rotation,
                        90,
                        0,
                        360,
                    );
                }
                ToolKind::ElectricMixer => {
                    ui.label(egui::RichText::new("Mixer:").size(11.0));
                    egui::ComboBox::from_id_salt("toolbar_mixer_effect")
                        .selected_text(format!(
                            "{} {}",
                            self.toolbox.active_mixer_effect.icon(),
                            self.toolbox.active_mixer_effect.display_name()
                        ))
                        .show_ui(ui, |ui| {
                            for effect in MixerEffect::ALL {
                                if ui
                                    .selectable_value(
                                        &mut self.toolbox.active_mixer_effect,
                                        effect,
                                        format!("{} {}", effect.icon(), effect.display_name()),
                                    )
                                    .clicked()
                                {
                                    self.apply_mixer_effect(effect);
                                }
                            }
                        });
                }
                ToolKind::Select | ToolKind::FreeformSelect => {
                    let is_opaque = self.toolbox.selection_mode == SelectionMode::Opaque;
                    if ui.selectable_label(!is_opaque, "Transparent").clicked() {
                        self.toolbox.selection_mode = SelectionMode::Transparent;
                    }
                    if ui.selectable_label(is_opaque, "Opaque").clicked() {
                        self.toolbox.selection_mode = SelectionMode::Opaque;
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Rotate Selection:").size(11.0));
                    if ui.button("90°").clicked() {
                        let before = self.layer_stack.clone();
                        self.canvas.apply_rotate_90(&mut self.layer_stack, false);
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                    }
                    if ui.button("180°").clicked() {
                        let before = self.layer_stack.clone();
                        self.canvas.apply_rotate_180(&mut self.layer_stack, false);
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                    }
                    if ui.button("270°").clicked() {
                        let before = self.layer_stack.clone();
                        self.canvas.apply_rotate_270(&mut self.layer_stack, false);
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                    }
                    ui.separator();
                    if ui.button("Flip H").clicked() {
                        let before = self.layer_stack.clone();
                        self.canvas
                            .apply_flip_horizontal(&mut self.layer_stack, false);
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                    }
                    if ui.button("Flip V").clicked() {
                        let before = self.layer_stack.clone();
                        self.canvas
                            .apply_flip_vertical(&mut self.layer_stack, false);
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                    }
                    if ui.button("Scale / Stretch...").clicked() {
                        self.stretch_skew_dlg.open();
                    }
                }
                ToolKind::WackyBrush => {
                    ui.label(egui::RichText::new("Tip:").size(11.0));
                    egui::ComboBox::from_id_salt("toolbar_wacky_shape")
                        .selected_text(format!(
                            "{} {}",
                            self.toolbox.wacky_brush_shape.icon(),
                            self.toolbox.wacky_brush_shape.display_name()
                        ))
                        .show_ui(ui, |ui| {
                            for shape in WackyBrushShape::ALL {
                                ui.selectable_value(
                                    &mut self.toolbox.wacky_brush_shape,
                                    shape,
                                    format!("{} {}", shape.icon(), shape.display_name()),
                                );
                            }
                        });
                    ui.separator();
                    ui.label(egui::RichText::new("Size:").size(11.0));
                    let size_resp = ui.add(
                        egui::Slider::new(&mut self.toolbox.wacky_brush_size, 1..=24).suffix(" px"),
                    );
                    handle_hover_scroll(
                        ui,
                        &size_resp,
                        &mut self.toolbox.wacky_brush_size,
                        1,
                        1,
                        24,
                    );
                }
                ToolKind::Brush => {
                    ui.label(egui::RichText::new("Shape:").size(11.0));
                    egui::ComboBox::from_id_salt("toolbar_brush_shape")
                        .selected_text(format!("{:?}", self.toolbox.brush_shape))
                        .show_ui(ui, |ui| {
                            for shape in [
                                BrushShape::RoundSmall,
                                BrushShape::RoundMedium,
                                BrushShape::RoundLarge,
                                BrushShape::SquareSmall,
                                BrushShape::SquareMedium,
                                BrushShape::SquareLarge,
                                BrushShape::ForwardSlashSmall,
                                BrushShape::ForwardSlashMedium,
                                BrushShape::ForwardSlashLarge,
                                BrushShape::BackslashSmall,
                                BrushShape::BackslashMedium,
                                BrushShape::BackslashLarge,
                            ] {
                                ui.selectable_value(
                                    &mut self.toolbox.brush_shape,
                                    shape,
                                    format!("{:?}", shape),
                                );
                            }
                        });
                }
                ToolKind::Eraser => {
                    ui.label(egui::RichText::new("Size:").size(11.0));
                    for (sz, label) in [
                        (EraserSize::Small, "S"),
                        (EraserSize::Medium, "M"),
                        (EraserSize::Large, "L"),
                        (EraserSize::XLarge, "XL"),
                    ] {
                        if ui
                            .selectable_label(self.toolbox.eraser_size == sz, label)
                            .clicked()
                        {
                            self.toolbox.eraser_size = sz;
                        }
                    }
                    ui.separator();
                    ui.checkbox(&mut self.toolbox.erase_to_transparent, "Transparent");
                }
                ToolKind::Line | ToolKind::Curve => {
                    ui.label(egui::RichText::new("Width:").size(11.0));
                    let width_resp = ui.add(
                        egui::DragValue::new(&mut self.toolbox.line_width)
                            .range(1..=5)
                            .suffix(" px"),
                    );
                    handle_hover_scroll(ui, &width_resp, &mut self.toolbox.line_width, 1, 1, 5);
                    for w in 1..=5 {
                        if ui
                            .selectable_label(self.toolbox.line_width == w, format!("{}px", w))
                            .clicked()
                        {
                            self.toolbox.line_width = w;
                        }
                    }
                }
                ToolKind::Rectangle
                | ToolKind::Polygon
                | ToolKind::Ellipse
                | ToolKind::RoundedRectangle => {
                    ui.label(egui::RichText::new("Fill Style:").size(11.0));
                    for (st, label) in [
                        (FillStyle::OutlineOnly, "Outline"),
                        (FillStyle::FilledWithOutline, "Fill+Line"),
                        (FillStyle::FillOnly, "Fill Only"),
                    ] {
                        if ui
                            .selectable_label(self.toolbox.fill_style == st, label)
                            .clicked()
                        {
                            self.toolbox.fill_style = st;
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Line Width:").size(11.0));
                    let width_resp = ui.add(
                        egui::DragValue::new(&mut self.toolbox.line_width)
                            .range(1..=5)
                            .suffix(" px"),
                    );
                    handle_hover_scroll(ui, &width_resp, &mut self.toolbox.line_width, 1, 1, 5);
                    for w in 1..=5 {
                        if ui
                            .selectable_label(self.toolbox.line_width == w, format!("{}px", w))
                            .clicked()
                        {
                            self.toolbox.line_width = w;
                        }
                    }
                }
                ToolKind::Fill => {
                    ui.label(egui::RichText::new("Tolerance:").size(11.0));
                    let tol_resp =
                        ui.add(egui::Slider::new(&mut self.toolbox.fill_tolerance, 0..=100));
                    handle_hover_scroll(ui, &tol_resp, &mut self.toolbox.fill_tolerance, 1, 0, 100);
                    for &t in &[0, 15, 32, 64] {
                        if ui
                            .selectable_label(self.toolbox.fill_tolerance == t, format!("{}", t))
                            .clicked()
                        {
                            self.toolbox.fill_tolerance = t;
                        }
                    }
                }
                ToolKind::PatternFill => {
                    ui.label(egui::RichText::new("Pattern:").size(11.0));
                    egui::ComboBox::from_id_salt("toolbar_pattern_fill")
                        .selected_text(self.toolbox.fill_pattern.display_name())
                        .show_ui(ui, |ui| {
                            for pat in FillPattern::ALL {
                                ui.selectable_value(
                                    &mut self.toolbox.fill_pattern,
                                    pat,
                                    pat.display_name(),
                                );
                            }
                        });
                    ui.separator();
                    ui.label(egui::RichText::new("Tolerance:").size(11.0));
                    let tol_resp =
                        ui.add(egui::Slider::new(&mut self.toolbox.fill_tolerance, 0..=100));
                    handle_hover_scroll(ui, &tol_resp, &mut self.toolbox.fill_tolerance, 1, 0, 100);
                }
                ToolKind::Magnifier => {
                    ui.label(egui::RichText::new("Zoom:").size(11.0));
                    for &z in &[1.0, 2.0, 4.0, 8.0, 16.0] {
                        if ui
                            .selectable_label(
                                (self.canvas.zoom - z).abs() < 0.1,
                                format!("{}x", z as u32),
                            )
                            .clicked()
                        {
                            self.canvas.set_zoom(z);
                        }
                    }
                }
                _ => {}
            }
        });
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);

        match self.theme_preference {
            ThemePreference::System => {}
            ThemePreference::Light => ctx.set_visuals(egui::Visuals::light()),
            ThemePreference::Dark => ctx.set_visuals(egui::Visuals::dark()),
        }

        // Update Window Title with filename and dirty indicator
        let file_title = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("untitled");
        let dirty_marker = if self.is_dirty { " *" } else { "" };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "{}{} - Paint DX",
            file_title, dirty_marker
        )));

        // Animation Playback Engine
        if self.timeline.is_playing {
            let now = Instant::now();
            let delta_ms = if let Some(last) = self.timeline.last_tick {
                now.duration_since(last).as_secs_f64() * 1000.0
            } else {
                0.0
            };
            self.timeline.last_tick = Some(now);
            self.timeline.playback_accum_ms += delta_ms;

            let current_idx = self.doc.active_frame_index;
            let current_duration = self
                .doc
                .frames
                .get(current_idx)
                .map(|f| f.duration_ms)
                .unwrap_or(83) as f64;

            if self.timeline.playback_accum_ms >= current_duration {
                self.timeline.playback_accum_ms =
                    (self.timeline.playback_accum_ms - current_duration).max(0.0);

                let next_idx = if let Some(ref tag_name) = self.timeline.active_tag {
                    if let Some(tag) = self.doc.tags.iter().find(|t| &t.name == tag_name) {
                        if current_idx >= tag.to_frame || current_idx < tag.from_frame {
                            tag.from_frame
                        } else {
                            current_idx + 1
                        }
                    } else {
                        (current_idx + 1) % self.doc.frames.len().max(1)
                    }
                } else if self.timeline.loop_playback {
                    (current_idx + 1) % self.doc.frames.len().max(1)
                } else if current_idx + 1 < self.doc.frames.len() {
                    current_idx + 1
                } else {
                    self.timeline.is_playing = false;
                    current_idx
                };

                if next_idx != current_idx {
                    self.doc.active_frame_index = next_idx;
                    self.layer_stack = self.doc.to_layer_stack(next_idx);
                    self.canvas.mark_dirty();
                }
            }

            let remaining = (current_duration - self.timeline.playback_accum_ms).max(1.0);
            ctx.request_repaint_after(Duration::from_millis(remaining as u64));
        }

        // Autosave periodic check
        self.autosave
            .tick(&self.doc, &self.palette.swatches, self.file_path.as_deref());

        // Crash recovery prompt window
        if self.recovery_dialog_open {
            egui::Window::new("Paint DX - Crash Recovery")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Unsaved Project Detected");
                    ui.label("Paint DX detected an unsaved project from a previous session.");
                    ui.label("Would you like to restore your work?");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Restore Project").clicked() {
                            if let Some(proj) = self.recovery_project.take() {
                                self.doc = proj.doc;
                                self.layer_stack =
                                    self.doc.to_layer_stack(self.doc.active_frame_index);
                                if let Some(p) = proj.palette {
                                    self.palette.swatches = p;
                                }
                                self.mark_dirty();
                                self.canvas.mark_dirty();
                            }
                            self.recovery_dialog_open = false;
                        }
                        if ui.button("Discard").clicked() {
                            self.autosave.clear_recovery_files();
                            self.recovery_project = None;
                            self.recovery_dialog_open = false;
                        }
                    });
                });
        }

        // Unsaved changes confirmation dialog
        if self.confirm_close_dialog_open {
            egui::Window::new("Paint DX")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let doc_name = self
                        .file_path
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("untitled");
                    ui.label(format!("Save changes to {}?", doc_name));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.save_file();
                            if !self.is_dirty {
                                self.confirm_close_dialog_open = false;
                                if let Some(act) = self.pending_action.take() {
                                    self.execute_pending_action(ctx, act);
                                }
                            }
                        }
                        if ui.button("Don't Save").clicked() {
                            self.confirm_close_dialog_open = false;
                            self.mark_clean();
                            if let Some(act) = self.pending_action.take() {
                                self.execute_pending_action(ctx, act);
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_close_dialog_open = false;
                            self.pending_action = None;
                        }
                    });
                });
        }

        // View Bitmap fullscreen mode
        if self.is_view_bitmap_fullscreen {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
                .show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        self.canvas.ui(
                            ui,
                            &mut self.layer_stack,
                            &mut self.history,
                            &self.toolbox,
                            &mut self.palette.fg_color,
                            &mut self.palette.bg_color,
                            Some((&self.doc, &self.timeline.onion_skin)),
                        );
                    });
                    if ui.input(|i| i.pointer.any_click() || i.key_pressed(egui::Key::Escape)) {
                        self.is_view_bitmap_fullscreen = false;
                    }
                });
            return;
        }

        // Handle dialog logic
        if let Some(action) = self.flip_rotate_dlg.ui(ctx) {
            let before = self.layer_stack.clone();
            match action {
                FlipRotateAction::FlipHorizontal => {
                    self.canvas
                        .apply_flip_horizontal(&mut self.layer_stack, false);
                }
                FlipRotateAction::FlipVertical => {
                    self.canvas
                        .apply_flip_vertical(&mut self.layer_stack, false);
                }
                FlipRotateAction::Rotate90 => {
                    self.canvas.apply_rotate_90(&mut self.layer_stack, false);
                }
                FlipRotateAction::Rotate180 => {
                    self.canvas.apply_rotate_180(&mut self.layer_stack, false);
                }
                FlipRotateAction::Rotate270 => {
                    self.canvas.apply_rotate_270(&mut self.layer_stack, false);
                }
            }
            self.history
                .record_layer_stack_change(&before, &self.layer_stack);
            self.mark_dirty();
        }

        if let Some(params) = self.stretch_skew_dlg.ui(ctx) {
            let before = self.layer_stack.clone();
            if params.stretch_x_pct != 100 || params.stretch_y_pct != 100 {
                self.canvas.apply_stretch(
                    &mut self.layer_stack,
                    params.stretch_x_pct,
                    params.stretch_y_pct,
                    false,
                );
            }
            if params.skew_x_deg != 0 || params.skew_y_deg != 0 {
                self.canvas.apply_skew(
                    &mut self.layer_stack,
                    params.skew_x_deg,
                    params.skew_y_deg,
                    self.palette.bg_color,
                    false,
                );
            }
            self.history
                .record_layer_stack_change(&before, &self.layer_stack);
            self.mark_dirty();
        }

        if let Some(params) = self.attributes_dlg.ui(ctx) {
            let before = self.layer_stack.clone();
            if params.width != self.layer_stack.width || params.height != self.layer_stack.height {
                self.canvas.apply_resize_canvas(
                    &mut self.layer_stack,
                    params.width,
                    params.height,
                    self.palette.bg_color,
                );
            }
            if params.color_mode == ColorModeChoice::BlackAndWhite {
                self.canvas.apply_monochrome(&mut self.layer_stack, true);
            }
            self.history
                .record_layer_stack_change(&before, &self.layer_stack);
            self.mark_dirty();
        }

        if let Some(color) = self.edit_colors_dlg.ui(ctx) {
            if let Some(idx) = self.editing_swatch_idx.take() {
                self.palette.swatches[idx] = color;
                self.palette.active_preset = None;
                self.mark_dirty();
            }
            self.palette.fg_color = color;
        }

        if let Some(cfg) = self.new_project_dlg.ui(ctx) {
            self.create_new_project(cfg.width, cfg.height, cfg.fps, cfg.bg_color);
        }

        self.export_png_seq_dlg
            .ui(ctx, &self.doc, self.layer_stack.active_layer_id);
        self.export_sheet_dlg
            .ui(ctx, &self.doc, self.layer_stack.active_layer_id);
        self.export_gif_dlg
            .ui(ctx, &self.doc, self.layer_stack.active_layer_id);

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New...\tCtrl+N").clicked() {
                        self.request_new();
                        ui.close_menu();
                    }
                    if ui.button("Open...\tCtrl+O").clicked() {
                        self.request_open();
                        ui.close_menu();
                    }
                    if ui
                        .button("Import Image as Layer...\tCtrl+Shift+I")
                        .clicked()
                    {
                        self.import_image_as_layer_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Save\tCtrl+S").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_file_as_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Export Animated GIF...\tCtrl+Shift+G").clicked() {
                        self.doc
                            .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
                        self.export_gif_dlg
                            .open(self.file_path.as_deref(), &self.doc);
                        ui.close_menu();
                    }
                    if ui.button("Export PNG Sequence...\tCtrl+Shift+E").clicked() {
                        self.export_png_seq_dlg
                            .open(self.file_path.as_deref(), &self.doc);
                        ui.close_menu();
                    }
                    if ui.button("Export Sprite Sheet...\tCtrl+Shift+X").clicked() {
                        self.export_sheet_dlg
                            .open(self.file_path.as_deref(), &self.doc);
                        ui.close_menu();
                    }
                    ui.separator();

                    ui.add_enabled_ui(false, |ui| {
                        let _ = ui.button("From Scanner or Camera... (Deferred)");
                        let _ = ui.button("Print Preview (Deferred)");
                        let _ = ui.button("Page Setup... (Deferred)");
                        let _ = ui.button("Print...\tCtrl+P (Deferred)");
                        let _ = ui.button("Send... (Deferred)");
                    });

                    ui.separator();
                    if ui.button("Set As Wallpaper (Tiled)").clicked() {
                        if let Some(ref path) = self.file_path {
                            let _ = set_desktop_wallpaper(path, WallpaperStyle::Tiled);
                        } else {
                            self.save_file_as_dialog();
                            if let Some(ref path) = self.file_path {
                                let _ = set_desktop_wallpaper(path, WallpaperStyle::Tiled);
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Set As Wallpaper (Centered)").clicked() {
                        if let Some(ref path) = self.file_path {
                            let _ = set_desktop_wallpaper(path, WallpaperStyle::Centered);
                        } else {
                            self.save_file_as_dialog();
                            if let Some(ref path) = self.file_path {
                                let _ = set_desktop_wallpaper(path, WallpaperStyle::Centered);
                            }
                        }
                        ui.close_menu();
                    }

                    if !self.recent_files.is_empty() {
                        ui.separator();
                        let files = self.recent_files.clone();
                        for (i, file) in files.iter().enumerate() {
                            let filename =
                                file.file_name().and_then(|n| n.to_str()).unwrap_or("image");
                            if ui.button(format!("{} {}", i + 1, filename)).clicked() {
                                self.request_load_recent(i);
                                ui.close_menu();
                            }
                        }
                    }

                    ui.separator();
                    if ui.button("Exit").clicked() {
                        self.request_exit(ui.ctx());
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let can_undo = self.history.can_undo();
                    let can_redo = self.history.can_redo();

                    if ui
                        .add_enabled(can_undo, egui::Button::new("Undo\tCtrl+Z"))
                        .clicked()
                    {
                        if self.history.undo_layer_stack(&mut self.layer_stack) {
                            self.canvas.clear_selection();
                            self.mark_dirty();
                            self.canvas.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(can_redo, egui::Button::new("Repeat\tF4 / Ctrl+Y"))
                        .clicked()
                    {
                        if self.history.redo_layer_stack(&mut self.layer_stack) {
                            self.canvas.clear_selection();
                            self.mark_dirty();
                            self.canvas.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Cut\tCtrl+X").clicked() {
                        let before = self.layer_stack.clone();
                        let cut_buf = self
                            .canvas
                            .cut_selection_or_canvas(&mut self.layer_stack, self.palette.bg_color);
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                        let _ = clipboard::copy_to_clipboard(&cut_buf);
                        ui.close_menu();
                    }
                    if ui.button("Copy\tCtrl+C").clicked() {
                        let copy_buf = self.canvas.copy_selection_or_canvas(&self.layer_stack);
                        let _ = clipboard::copy_to_clipboard(&copy_buf);
                        ui.close_menu();
                    }
                    if ui.button("Paste\tCtrl+V").clicked() {
                        if let Ok(pasted_buf) = clipboard::paste_from_clipboard() {
                            let before = self.layer_stack.clone();
                            self.canvas.paste_as_selection(
                                &mut self.layer_stack,
                                pasted_buf,
                                &self.toolbox,
                                self.palette.bg_color,
                            );
                            self.history
                                .record_layer_stack_change(&before, &self.layer_stack);
                            self.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    if ui.button("Clear Selection\tDel").clicked() {
                        if self.canvas.has_floating_selection() {
                            let before = self.layer_stack.clone();
                            self.canvas.clear_selection();
                            self.history
                                .record_layer_stack_change(&before, &self.layer_stack);
                            self.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    if ui.button("Select All\tCtrl+A").clicked() {
                        let before = self.layer_stack.clone();
                        self.canvas
                            .select_all(&mut self.layer_stack, self.palette.bg_color);
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Copy To...").clicked() {
                        self.copy_to_file_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Paste From...").clicked() {
                        self.paste_from_file_dialog();
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_toolbar, "Tool Options Toolbar\tCtrl+T");
                    ui.checkbox(&mut self.show_toolbox, "Tool Box");
                    ui.checkbox(&mut self.show_palette, "Color Box\tCtrl+L");
                    ui.checkbox(&mut self.show_statusbar, "Status Bar");
                    ui.separator();

                    if ui.button("Center Canvas\tCtrl+0").clicked() {
                        self.canvas.zoom = 1.0;
                        self.canvas.mark_dirty();
                        ui.close_menu();
                    }

                    ui.menu_button("Zoom", |ui| {
                        if ui.button("50% (0.5x)").clicked() {
                            self.canvas.set_zoom(0.5);
                            ui.close_menu();
                        }
                        if ui.button("Normal Size (100%)\tCtrl+PgUp").clicked() {
                            self.canvas.set_zoom(1.0);
                            ui.close_menu();
                        }
                        if ui.button("200% (2x)").clicked() {
                            self.canvas.set_zoom(2.0);
                            ui.close_menu();
                        }
                        if ui.button("Large Size (400%)\tCtrl+PgDn").clicked() {
                            self.canvas.set_zoom(4.0);
                            ui.close_menu();
                        }
                        if ui.button("600% (6x)").clicked() {
                            self.canvas.set_zoom(6.0);
                            ui.close_menu();
                        }
                        if ui.button("800% (8x)").clicked() {
                            self.canvas.set_zoom(8.0);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("1200% (12x)").clicked() {
                            self.canvas.set_zoom(12.0);
                            ui.close_menu();
                        }
                        if ui.button("1600% (16x)").clicked() {
                            self.canvas.set_zoom(16.0);
                            ui.close_menu();
                        }
                        if ui.button("2400% (24x)").clicked() {
                            self.canvas.set_zoom(24.0);
                            ui.close_menu();
                        }
                        if ui.button("3200% (32x)").clicked() {
                            self.canvas.set_zoom(32.0);
                            ui.close_menu();
                        }
                        if ui.button("4800% (48x)").clicked() {
                            self.canvas.set_zoom(48.0);
                            ui.close_menu();
                        }
                        if ui.button("6400% (64x)").clicked() {
                            self.canvas.set_zoom(64.0);
                            ui.close_menu();
                        }
                    });

                    ui.checkbox(&mut self.canvas.show_grid, "Show Grid\tCtrl+G");
                    ui.checkbox(&mut self.thumbnail.is_open, "Show Thumbnail");
                    if ui.button("View Bitmap\tCtrl+F").clicked() {
                        self.is_view_bitmap_fullscreen = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.checkbox(&mut self.layers.is_open, "Layers Panel");
                    ui.checkbox(&mut self.timeline.is_open, "Timeline Panel");
                    ui.separator();
                    ui.menu_button("Theme", |ui| {
                        if ui
                            .selectable_label(
                                self.theme_preference == ThemePreference::System,
                                "Follow System",
                            )
                            .clicked()
                        {
                            self.theme_preference = ThemePreference::System;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.theme_preference == ThemePreference::Light,
                                "Light (Classic XP)",
                            )
                            .clicked()
                        {
                            self.theme_preference = ThemePreference::Light;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.theme_preference == ThemePreference::Dark,
                                "Dark Mode",
                            )
                            .clicked()
                        {
                            self.theme_preference = ThemePreference::Dark;
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Image", |ui| {
                    if ui.button("Flip/Rotate...\tCtrl+R").clicked() {
                        self.flip_rotate_dlg.open();
                        ui.close_menu();
                    }
                    if ui.button("Stretch/Skew...\tCtrl+W").clicked() {
                        self.stretch_skew_dlg.open();
                        ui.close_menu();
                    }
                    if ui.button("Invert Colors\tCtrl+I").clicked() {
                        let before = self.layer_stack.clone();
                        self.canvas
                            .apply_invert_colors(&mut self.layer_stack, false);
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                        ui.close_menu();
                    }
                    if ui.button("Attributes...\tCtrl+E").clicked() {
                        self.attributes_dlg
                            .open(self.layer_stack.width, self.layer_stack.height);
                        ui.close_menu();
                    }
                    if ui.button("Clear Image").clicked() {
                        let before = self.layer_stack.clone();
                        self.canvas.clear_selection();
                        if let Some(layer) = self.layer_stack.active_layer_mut() {
                            layer.buffer.clear(self.palette.bg_color);
                        }
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                        self.canvas.mark_dirty();
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.menu_button("Symmetry / Mirror Mode", |ui| {
                        if ui
                            .selectable_label(
                                self.canvas.symmetry_mode == SymmetryMode::None,
                                "Off",
                            )
                            .clicked()
                        {
                            self.canvas.symmetry_mode = SymmetryMode::None;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.canvas.symmetry_mode == SymmetryMode::Horizontal,
                                "Horizontal (Left / Right)",
                            )
                            .clicked()
                        {
                            self.canvas.symmetry_mode = SymmetryMode::Horizontal;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.canvas.symmetry_mode == SymmetryMode::Vertical,
                                "Vertical (Top / Bottom)",
                            )
                            .clicked()
                        {
                            self.canvas.symmetry_mode = SymmetryMode::Vertical;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.canvas.symmetry_mode == SymmetryMode::Quad,
                                "4-Way (Quad)",
                            )
                            .clicked()
                        {
                            self.canvas.symmetry_mode = SymmetryMode::Quad;
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                    let is_opaque = self.toolbox.selection_mode == SelectionMode::Opaque;
                    let mut opaque_check = is_opaque;
                    if ui.checkbox(&mut opaque_check, "Draw Opaque").clicked() {
                        self.toolbox.selection_mode = if opaque_check {
                            SelectionMode::Opaque
                        } else {
                            SelectionMode::Transparent
                        };
                    }
                });

                ui.menu_button("Tools", |ui| {
                    ui.menu_button("Tool Set", |ui| {
                        let is_classic = self.toolbox.mode == ToolboxMode::Classic;
                        if ui
                            .selectable_label(is_classic, "🎨 Classic Windows XP")
                            .clicked()
                        {
                            self.toolbox.mode = ToolboxMode::Classic;
                            ui.close_menu();
                        }
                        let is_kidpix = self.toolbox.mode == ToolboxMode::KidPix;
                        if ui
                            .selectable_label(is_kidpix, "🤪 Kid Pix / Wacky Tools")
                            .clicked()
                        {
                            self.toolbox.mode = ToolboxMode::KidPix;
                            if self.toolbox.active_tool == ToolKind::Brush
                                || self.toolbox.active_tool == ToolKind::Pencil
                            {
                                self.toolbox.active_tool = ToolKind::WackyBrush;
                            }
                            ui.close_menu();
                        }
                    });

                    ui.separator();

                    // 1. Classic XP Tools
                    ui.menu_button("🎨 Classic XP Tools", |ui| {
                        for tool in ToolKind::CLASSIC_TOOLS {
                            let is_current = self.toolbox.active_tool == tool
                                && self.toolbox.mode == ToolboxMode::Classic;
                            if ui
                                .selectable_label(is_current, tool.display_name())
                                .clicked()
                            {
                                self.toolbox.mode = ToolboxMode::Classic;
                                self.toolbox.active_tool = tool;
                                ui.close_menu();
                            }
                        }
                    });

                    // 2. Kid Pix Tools
                    ui.menu_button("🤪 Kid Pix Tools", |ui| {
                        if ui
                            .selectable_label(
                                self.toolbox.active_tool == ToolKind::WackyBrush,
                                "🪄 Wacky Brush",
                            )
                            .clicked()
                        {
                            self.toolbox.mode = ToolboxMode::KidPix;
                            self.toolbox.active_tool = ToolKind::WackyBrush;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.toolbox.active_tool == ToolKind::Stamp,
                                "🌟 Rubber Stamps",
                            )
                            .clicked()
                        {
                            self.toolbox.mode = ToolboxMode::KidPix;
                            self.toolbox.active_tool = ToolKind::Stamp;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.toolbox.active_tool == ToolKind::PatternFill,
                                "🪣 Pattern Fill",
                            )
                            .clicked()
                        {
                            self.toolbox.mode = ToolboxMode::KidPix;
                            self.toolbox.active_tool = ToolKind::PatternFill;
                            ui.close_menu();
                        }
                        if ui
                            .selectable_label(
                                self.toolbox.active_tool == ToolKind::ElectricMixer,
                                "⚡ Electric Mixer",
                            )
                            .clicked()
                        {
                            self.toolbox.mode = ToolboxMode::KidPix;
                            self.toolbox.active_tool = ToolKind::ElectricMixer;
                            ui.close_menu();
                        }

                        ui.separator();
                        ui.menu_button("Wacky Brush Tips", |ui| {
                            for shape in WackyBrushShape::ALL {
                                let is_current = self.toolbox.wacky_brush_shape == shape
                                    && self.toolbox.active_tool == ToolKind::WackyBrush;
                                let text = format!("{} {}", shape.icon(), shape.display_name());
                                if ui.selectable_label(is_current, text).clicked() {
                                    self.toolbox.mode = ToolboxMode::KidPix;
                                    self.toolbox.active_tool = ToolKind::WackyBrush;
                                    self.toolbox.wacky_brush_shape = shape;
                                    ui.close_menu();
                                }
                            }
                        });
                    });

                    // 3. Electric Mixer
                    ui.menu_button("⚡ Electric Mixer", |ui| {
                        if ui
                            .selectable_label(
                                self.toolbox.active_tool == ToolKind::ElectricMixer,
                                "⚡ Select Electric Mixer Tool",
                            )
                            .clicked()
                        {
                            self.toolbox.mode = ToolboxMode::KidPix;
                            self.toolbox.active_tool = ToolKind::ElectricMixer;
                            ui.close_menu();
                        }
                        ui.separator();

                        ui.menu_button("Tray 1 (Classic Mixers)", |ui| {
                            for effect in MixerEffect::TRAY_1 {
                                let is_active = self.toolbox.active_mixer_effect == effect
                                    && self.toolbox.active_tool == ToolKind::ElectricMixer;
                                let label = format!("{} {}", effect.icon(), effect.display_name());
                                if ui
                                    .selectable_label(is_active, label)
                                    .on_hover_text(effect.tooltip())
                                    .clicked()
                                {
                                    self.toolbox.mode = ToolboxMode::KidPix;
                                    self.toolbox.active_tool = ToolKind::ElectricMixer;
                                    self.toolbox.active_mixer_effect = effect;
                                    ui.close_menu();
                                }
                            }
                        });

                        ui.menu_button("Tray 2 (Wacky & Party)", |ui| {
                            for effect in MixerEffect::TRAY_2 {
                                let is_active = self.toolbox.active_mixer_effect == effect
                                    && self.toolbox.active_tool == ToolKind::ElectricMixer;
                                let label = format!("{} {}", effect.icon(), effect.display_name());
                                if ui
                                    .selectable_label(is_active, label)
                                    .on_hover_text(effect.tooltip())
                                    .clicked()
                                {
                                    self.toolbox.mode = ToolboxMode::KidPix;
                                    self.toolbox.active_tool = ToolKind::ElectricMixer;
                                    self.toolbox.active_mixer_effect = effect;
                                    ui.close_menu();
                                }
                            }
                        });

                        ui.separator();
                        if ui
                            .button("🎲 Random Mixer Effect\tCtrl+M")
                            .on_hover_text("Apply a randomly chosen Electric Mixer effect")
                            .clicked()
                        {
                            let rand_idx = (self.mixer_seed as usize) % MixerEffect::ALL.len();
                            let effect = MixerEffect::ALL[rand_idx];
                            self.toolbox.mode = ToolboxMode::KidPix;
                            self.toolbox.active_tool = ToolKind::ElectricMixer;
                            self.toolbox.active_mixer_effect = effect;
                            self.apply_mixer_effect(effect);
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Layer", |ui| {
                    if ui.button("New Layer...\tCtrl+Shift+N").clicked() {
                        let before = self.layer_stack.clone();
                        let count = self.layer_stack.layers.len() + 1;
                        self.layer_stack.add_layer(format!("Layer {}", count));
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                        self.canvas.mark_dirty();
                        ui.close_menu();
                    }
                    if ui
                        .button("Import Image as Layer...\tCtrl+Shift+I")
                        .clicked()
                    {
                        self.import_image_as_layer_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Duplicate Layer").clicked() {
                        let before = self.layer_stack.clone();
                        if self
                            .layer_stack
                            .duplicate_layer(self.layer_stack.active_layer_id)
                            .is_some()
                        {
                            self.history
                                .record_layer_stack_change(&before, &self.layer_stack);
                            self.mark_dirty();
                            self.canvas.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    let can_delete = self.layer_stack.layers.len() > 1;
                    if ui
                        .add_enabled(can_delete, egui::Button::new("Delete Layer"))
                        .clicked()
                    {
                        let before = self.layer_stack.clone();
                        if self.layer_stack.delete_active_layer() {
                            self.history
                                .record_layer_stack_change(&before, &self.layer_stack);
                            self.mark_dirty();
                            self.canvas.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    let active_idx = self.layer_stack.active_index().unwrap_or(0);
                    let can_up = active_idx + 1 < self.layer_stack.layers.len();
                    let can_down = active_idx > 0;
                    if ui
                        .add_enabled(can_up, egui::Button::new("Move Layer Up"))
                        .clicked()
                    {
                        let before = self.layer_stack.clone();
                        if self.layer_stack.move_up(self.layer_stack.active_layer_id) {
                            self.history
                                .record_layer_stack_change(&before, &self.layer_stack);
                            self.mark_dirty();
                            self.canvas.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(can_down, egui::Button::new("Move Layer Down"))
                        .clicked()
                    {
                        let before = self.layer_stack.clone();
                        if self.layer_stack.move_down(self.layer_stack.active_layer_id) {
                            self.history
                                .record_layer_stack_change(&before, &self.layer_stack);
                            self.mark_dirty();
                            self.canvas.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    let can_merge = active_idx > 0;
                    if ui
                        .add_enabled(can_merge, egui::Button::new("Merge Down\tCtrl+E"))
                        .clicked()
                    {
                        let before = self.layer_stack.clone();
                        if self
                            .layer_stack
                            .merge_down(self.layer_stack.active_layer_id)
                        {
                            self.history
                                .record_layer_stack_change(&before, &self.layer_stack);
                            self.mark_dirty();
                            self.canvas.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    let can_flatten = self.layer_stack.layers.len() > 1;
                    if ui
                        .add_enabled(can_flatten, egui::Button::new("Merge Visible"))
                        .clicked()
                    {
                        let before = self.layer_stack.clone();
                        if self.layer_stack.merge_visible() {
                            self.history
                                .record_layer_stack_change(&before, &self.layer_stack);
                            self.mark_dirty();
                            self.canvas.mark_dirty();
                        }
                        ui.close_menu();
                    }
                    if ui
                        .add_enabled(can_flatten, egui::Button::new("Flatten Image"))
                        .clicked()
                    {
                        let before = self.layer_stack.clone();
                        self.layer_stack.flatten(self.palette.bg_color);
                        self.history
                            .record_layer_stack_change(&before, &self.layer_stack);
                        self.mark_dirty();
                        self.canvas.mark_dirty();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Colors", |ui| {
                    if ui.button("Edit Colors...").clicked() {
                        self.editing_swatch_idx = None;
                        self.edit_colors_dlg.open(self.palette.fg_color);
                        ui.close_menu();
                    }

                    ui.separator();

                    ui.menu_button("Retro Palettes", |ui| {
                        for preset in paint_core::palette::RetroPalettePreset::ALL {
                            let is_current = self.palette.active_preset == Some(preset);
                            let text = if is_current {
                                format!("✓ {}", preset.name())
                            } else {
                                preset.name().to_string()
                            };
                            if ui.button(text).clicked() {
                                self.palette.apply_preset(preset);
                                self.mark_dirty();
                                ui.close_menu();
                            }
                        }
                    });
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Help Topics").clicked() {
                        self.help_open = true;
                        ui.close_menu();
                    }
                    if ui.button("Register .paintproj Associations").clicked() {
                        let _ = platform::windows::register_file_associations();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("About Paint DX").clicked() {
                        self.about_open = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Top tool options toolbar (Scale, Rotation, Tool-specific properties)
        if self.show_toolbar {
            egui::TopBottomPanel::top("tool_options_toolbar").show(ctx, |ui| {
                self.render_tool_options_toolbar(ui);
            });
        }

        // About & Help windows
        if self.about_open {
            egui::Window::new("About Paint DX")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Paint DX");
                    ui.label("Rust rewrite of classic Windows XP Paint for Windows 11.");
                    ui.label("Version 1.0.0 - Windows 11 Native Polish (Milestone 10 Complete)");
                    ui.label("Full layers, frames timeline, onion skin, .paintproj, PNG sequence and sprite sheet export.");
                    ui.add_space(8.0);
                    if ui.button("OK").clicked() {
                        self.about_open = false;
                    }
                });
        }

        if self.help_open {
            egui::Window::new("Help Topics")
                .resizable(true)
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    ui.label("Paint DX Help Topics:");
                    ui.label("• Draw with Left mouse button using Foreground Color.");
                    ui.label("• Draw with Right mouse button using Background Color.");
                    ui.label("• Hold Shift while drawing lines/shapes to constrain proportions.");
                    ui.label(
                        "• Save/Open .paintproj files to preserve full layer stacks and palettes.",
                    );
                    ui.label("• Automatic crash recovery protects unsaved drawings.");
                    ui.label("• Use Image menu to Flip, Rotate, Stretch, Skew, or Invert colors.");
                    ui.label(
                        "• Use Layer menu or Layers Panel to manage layers and adjust opacity.",
                    );
                    ui.label("• Double-click any palette color to open the Edit Colors dialog.");
                    if ui.button("Close").clicked() {
                        self.help_open = false;
                    }
                });
        }

        // Floating thumbnail window
        self.thumbnail.ui(
            ctx,
            &self.canvas.cached_composite,
            self.canvas.texture_version(),
        );

        // Bottom status bar
        if self.show_statusbar {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let tool_help = match self.toolbox.active_tool {
                        ToolKind::FreeformSelect => {
                            "Selects a free-form part of the picture to move, copy, or edit."
                        }
                        ToolKind::Select => {
                            "Selects a rectangular part of the picture to move, copy, or edit."
                        }
                        ToolKind::Eraser => {
                            "Erases a portion of the picture, using the selected eraser shape."
                        }
                        ToolKind::Fill => "Fills an area with the current drawing color.",
                        ToolKind::PickColor => "Picks up a color from the picture for drawing.",
                        ToolKind::Magnifier => "Changes the magnification.",
                        ToolKind::Pencil => "Draws a free-form line one pixel wide.",
                        ToolKind::Brush => "Draws using a brush with the selected shape and size.",
                        ToolKind::Airbrush => "Draws using an airbrush of the selected size.",
                        ToolKind::Text => "Inserts text into the picture.",
                        ToolKind::Line => "Draws a straight line with the selected line width.",
                        ToolKind::Curve => "Draws a curved line with the selected line width.",
                        ToolKind::Rectangle => "Draws a rectangle with the selected fill style.",
                        ToolKind::Polygon => "Draws a polygon with the selected fill style.",
                        ToolKind::Ellipse => "Draws an ellipse with the selected fill style.",
                        ToolKind::RoundedRectangle => {
                            "Draws a rounded rectangle with the selected fill style."
                        }
                        ToolKind::Move => "Moves the content of the active layer.",
                        ToolKind::WackyBrush => {
                            "Draws using creative wacky brushes (Pine Trees, Rainbow, Caterpillar, Pancakes, Fur, Patterns, Letters, etc.)."
                        }
                        ToolKind::Stamp => {
                            "Stamps a retro pixel-art rubber stamp onto the active layer."
                        }
                        ToolKind::PatternFill => {
                            "Fills an area with a retro pixel pattern."
                        }
                        ToolKind::ElectricMixer => {
                            "Electric Mixer: applies wild Kid Pix transformations and special effects (Tray 1 & Tray 2)."
                        }
                    };
                    ui.label(tool_help);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!(
                            "{}x{}px",
                            self.layer_stack.width, self.layer_stack.height
                        ));

                        if let Some(active) = self.layer_stack.active_layer() {
                            ui.separator();
                            let lock_str = if active.locked { " [Locked]" } else { "" };
                            let hide_str = if !active.visible { " [Hidden]" } else { "" };
                            ui.label(format!("Active: {}{}{}", active.name, lock_str, hide_str));
                        }

                        if self.canvas.symmetry_mode != SymmetryMode::None {
                            ui.separator();
                            ui.label(self.canvas.symmetry_mode.label());
                        }

                        if let Some(pos) = self.canvas.cursor_canvas_pos {
                            ui.separator();
                            ui.label(format!("{}, {}px", pos.x, pos.y));
                        }
                    });
                });
            });
        }

        // Bottom palette bar
        if self.show_palette {
            egui::TopBottomPanel::bottom("palette_panel").show(ctx, |ui| {
                if let Some(swatch_idx) = self.palette.ui(ui) {
                    self.editing_swatch_idx = Some(swatch_idx);
                    self.edit_colors_dlg.open(self.palette.swatches[swatch_idx]);
                }
            });
        }

        // Optional bottom timeline panel
        if self.timeline.is_open {
            egui::TopBottomPanel::bottom("timeline_panel")
                .resizable(true)
                .default_height(140.0)
                .show(ctx, |ui| {
                    let actions = self.timeline.ui(ui, &self.doc);
                    for action in actions {
                        self.handle_timeline_action(action);
                    }
                });
        }

        // Left toolbox panel
        if self.show_toolbox {
            egui::SidePanel::left("toolbox_panel")
                .resizable(false)
                .default_width(120.0)
                .show(ctx, |ui| {
                    self.toolbox.ui(
                        ui,
                        &mut self.canvas.symmetry_mode,
                        &mut self.canvas.show_grid,
                    );
                });
        }

        // Optional right layers panel
        if self.layers.is_open {
            egui::SidePanel::right("layers_panel")
                .resizable(true)
                .default_width(200.0)
                .show(ctx, |ui| {
                    if let Some(action) = self.layers.ui(
                        ui,
                        &mut self.layer_stack,
                        &mut self.history,
                        self.canvas.texture_version(),
                        self.palette.bg_color,
                    ) {
                        match action {
                            LayerAction::Modified | LayerAction::StackChanged => {
                                self.doc.sync_from_layer_stack(
                                    self.doc.active_frame_index,
                                    &self.layer_stack,
                                );
                                self.mark_dirty();
                                self.canvas.mark_dirty();
                            }
                            LayerAction::ImportImageRequested => {
                                self.import_image_as_layer_dialog();
                            }
                        }
                    }
                });
        }

        // Central canvas panel
        let undo_count_before = self.history.undo_count();
        egui::CentralPanel::default().show(ctx, |ui| {
            self.canvas.ui(
                ui,
                &mut self.layer_stack,
                &mut self.history,
                &self.toolbox,
                &mut self.palette.fg_color,
                &mut self.palette.bg_color,
                Some((&self.doc, &self.timeline.onion_skin)),
            );
        });

        if self.history.undo_count() != undo_count_before {
            self.doc
                .sync_from_layer_stack(self.doc.active_frame_index, &self.layer_stack);
            self.mark_dirty();
        }

        // Hide HTML loading splash on web as soon as egui starts updating
        #[cfg(target_arch = "wasm32")]
        {
            static HIDDEN_LOADER: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !HIDDEN_LOADER.swap(true, std::sync::atomic::Ordering::Relaxed) {
                platform::web::hide_loading_screen();
            }
        }

        // Render splash screen overlay if active
        self.render_splash_screen(ctx);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if args[1] == "--register-associations" {
            match platform::windows::register_file_associations() {
                Ok(_) => {
                    println!("Successfully registered .paintproj file associations.");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Failed to register file associations: {}", e);
                    return Ok(());
                }
            }
        } else if args[1] == "--unregister-associations" {
            match platform::windows::unregister_file_associations() {
                Ok(_) => {
                    println!("Successfully unregistered .paintproj file associations.");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Failed to unregister file associations: {}", e);
                    return Ok(());
                }
            }
        }
    }

    let initial_file = args.get(1).map(PathBuf::from).filter(|p| p.exists());

    let icon_data = image::load_from_memory(include_bytes!("../assets/icone.png"))
        .ok()
        .map(|img| {
            let rgba = img.into_rgba8();
            let (w, h) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width: w,
                height: h,
            }
        });

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("untitled - Paint DX")
        .with_inner_size([960.0, 680.0])
        .with_min_inner_size([400.0, 300.0]);

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Paint DX",
        native_options,
        Box::new(move |_cc| Ok(Box::new(PaintApp::with_initial_file(initial_file)))),
    )
}

#[cfg(target_arch = "wasm32")]
pub fn download_bytes(filename: &str, data: &[u8]) {
    use wasm_bindgen::JsCast;
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let uint8_array = js_sys::Uint8Array::new_with_length(data.len() as u32);
            uint8_array.copy_from(data);
            let array = js_sys::Array::new();
            array.push(&uint8_array.buffer());
            if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence(&array) {
                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                    if let Ok(a) = document.create_element("a") {
                        let _ = a.set_attribute("href", &url);
                        let _ = a.set_attribute("download", filename);
                        if let Ok(html_a) = a.dyn_into::<web_sys::HtmlElement>() {
                            html_a.click();
                        }
                    }
                    let _ = web_sys::Url::revoke_object_url(&url);
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }

    console_error_panic_hook::set_once();
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            web_sys::console::error_1(&"No global window found".into());
            return;
        }
    };
    let document = match window.document() {
        Some(d) => d,
        None => {
            web_sys::console::error_1(&"No document found on window".into());
            return;
        }
    };
    let canvas = match document.get_element_by_id("paint_canvas") {
        Some(elem) => match elem.dyn_into::<web_sys::HtmlCanvasElement>() {
            Ok(c) => c,
            Err(_) => {
                web_sys::console::error_1(&"Element 'paint_canvas' is not a canvas".into());
                return;
            }
        },
        None => {
            web_sys::console::error_1(&"Canvas element 'paint_canvas' not found in DOM".into());
            return;
        }
    };

    wasm_bindgen_futures::spawn_local(async move {
        let web_options = eframe::WebOptions::default();
        let runner = eframe::WebRunner::new();
        if let Err(e) = runner
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(PaintApp::default()))),
            )
            .await
        {
            web_sys::console::error_1(&format!("eframe::WebRunner error: {:?}", e).into());
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty on wasm32: #[wasm_bindgen(start)] automatically invokes start() on module init.
}
