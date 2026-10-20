//! Export Animated GIF Dialog.
//!
//! Provides UI controls for frame ranges, layer scopes, transparent vs solid background,
//! framerate / FPS override, loop count (infinite / once / custom), and nearest-neighbor scaling.

use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, Context, Window};
use paint_core::export::gif::{
    export_gif_to_disk, GifExportOptions, GifExportResult, GifRepeatOption,
};
use paint_core::export::png_seq::{FrameRangeChoice, LayerExportScope, TransparencyOption};
use paint_core::frames::AnimationDocument;
use paint_core::layers::LayerId;
use paint_core::palette::{ColorRgba, WHITE};

use super::export_png_seq::{FrameRangeMode, LayerScopeChoice, TransparencyChoice};

#[derive(Debug)]
pub struct ExportGifDialog {
    pub is_open: bool,
    pub output_path: PathBuf,
    pub frame_range_mode: FrameRangeMode,
    pub selected_tag: String,
    pub custom_start: usize,
    pub custom_end: usize,
    pub layer_scope: LayerScopeChoice,
    pub transparency_mode: TransparencyChoice,
    pub flatten_color: ColorRgba,
    pub scale: u32,
    pub use_custom_fps: bool,
    pub custom_fps: u32,
    pub repeat_mode: GifRepeatOption,

    // Status / Notification
    pub status_message: Option<String>,
    pub status_is_error: bool,
}

impl Default for ExportGifDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            output_path: PathBuf::from("export/animation.gif"),
            frame_range_mode: FrameRangeMode::All,
            selected_tag: String::new(),
            custom_start: 0,
            custom_end: 0,
            layer_scope: LayerScopeChoice::VisibleComposite,
            transparency_mode: TransparencyChoice::KeepTransparency,
            flatten_color: WHITE,
            scale: 1,
            use_custom_fps: false,
            custom_fps: 12,
            repeat_mode: GifRepeatOption::Infinite,
            status_message: None,
            status_is_error: false,
        }
    }
}

impl ExportGifDialog {
    pub fn open(&mut self, current_project_path: Option<&Path>, doc: &AnimationDocument) {
        self.is_open = true;
        self.status_message = None;
        self.status_is_error = false;
        self.custom_fps = doc.fps.max(1);

        if let Some(path) = current_project_path {
            if let Some(parent) = path.parent() {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("animation");
                self.output_path = parent.join(format!("{}.gif", stem));
            }
        }

        self.custom_start = 0;
        self.custom_end = doc.frames.len().saturating_sub(1);
        if let Some(first_tag) = doc.tags.first() {
            self.selected_tag = first_tag.name.clone();
        }
    }

    /// Renders the dialog window and executes export when triggered.
    pub fn ui(
        &mut self,
        ctx: &Context,
        doc: &AnimationDocument,
        active_layer_id: LayerId,
    ) -> Option<GifExportResult> {
        if !self.is_open {
            return None;
        }

        let mut export_triggered = false;
        let mut export_result = None;

        let mut is_open = self.is_open;
        Window::new("Export Animated GIF")
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .default_width(440.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);

                // 1. Output Destination File
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Destination File:").strong());
                    ui.horizontal(|ui| {
                        let path_str = self.output_path.to_string_lossy();
                        ui.add(
                            egui::TextEdit::singleline(&mut path_str.to_string())
                                .desired_width(300.0)
                                .interactive(false),
                        );
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("GIF Image (*.gif)", &["gif"])
                                .set_file_name(
                                    self.output_path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("animation.gif"),
                                )
                                .save_file()
                            {
                                self.output_path = path;
                            }
                        }
                    });
                });

                // 2. Frame Range Selection
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Frame Range:").strong());
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.frame_range_mode, FrameRangeMode::All, "All");
                        ui.radio_value(
                            &mut self.frame_range_mode,
                            FrameRangeMode::ActiveFrame,
                            "Active Frame",
                        );
                        if !doc.tags.is_empty() {
                            ui.radio_value(&mut self.frame_range_mode, FrameRangeMode::Tag, "Tag");
                        }
                        ui.radio_value(
                            &mut self.frame_range_mode,
                            FrameRangeMode::Custom,
                            "Custom",
                        );
                    });

                    if self.frame_range_mode == FrameRangeMode::Tag && !doc.tags.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Select Tag:");
                            egui::ComboBox::from_id_salt("gif_export_tag_select")
                                .selected_text(&self.selected_tag)
                                .show_ui(ui, |ui| {
                                    for tag in &doc.tags {
                                        ui.selectable_value(
                                            &mut self.selected_tag,
                                            tag.name.clone(),
                                            &tag.name,
                                        );
                                    }
                                });
                        });
                    }

                    if self.frame_range_mode == FrameRangeMode::Custom {
                        ui.horizontal(|ui| {
                            let max_frame = doc.frames.len().saturating_sub(1);
                            ui.label("From Frame:");
                            ui.add(
                                egui::DragValue::new(&mut self.custom_start)
                                    .range(0..=max_frame)
                                    .speed(0.1),
                            );
                            ui.label("To Frame:");
                            ui.add(
                                egui::DragValue::new(&mut self.custom_end)
                                    .range(self.custom_start..=max_frame)
                                    .speed(0.1),
                            );
                        });
                    }
                });

                // 3. Animation & Timing Settings
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Animation Timing & Looping:").strong());
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.use_custom_fps, "Override Framerate (FPS):");
                        if self.use_custom_fps {
                            ui.add(
                                egui::DragValue::new(&mut self.custom_fps)
                                    .range(1..=60)
                                    .speed(0.2)
                                    .suffix(" fps"),
                            );
                        } else {
                            ui.label(format!("(Using Document: {} fps)", doc.fps));
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Loop Repeat:");
                        ui.selectable_value(
                            &mut self.repeat_mode,
                            GifRepeatOption::Infinite,
                            "Infinite Loop",
                        );
                        ui.selectable_value(
                            &mut self.repeat_mode,
                            GifRepeatOption::Once,
                            "Play Once",
                        );
                    });
                });

                // 4. Scaling & Appearance
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Appearance & Scaling:").strong());
                    ui.horizontal(|ui| {
                        ui.label("Scale Multiplier:");
                        egui::ComboBox::from_id_salt("gif_export_scale_select")
                            .selected_text(format!("{}x", self.scale))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.scale, 1, "1x (Original)");
                                ui.selectable_value(&mut self.scale, 2, "2x (Pixel Perfect)");
                                ui.selectable_value(&mut self.scale, 4, "4x");
                                ui.selectable_value(&mut self.scale, 8, "8x");
                            });

                        let orig_w = doc.width;
                        let orig_h = doc.height;
                        ui.label(format!(
                            "→ Size: {}x{} px",
                            orig_w * self.scale,
                            orig_h * self.scale
                        ));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Transparency:");
                        ui.radio_value(
                            &mut self.transparency_mode,
                            TransparencyChoice::KeepTransparency,
                            "Transparent",
                        );
                        ui.radio_value(
                            &mut self.transparency_mode,
                            TransparencyChoice::FlattenToSolidColor,
                            "Solid Background",
                        );
                    });
                });

                // Status message banner if any
                if let Some(ref msg) = self.status_message {
                    let color = if self.status_is_error {
                        Color32::from_rgb(204, 0, 0)
                    } else {
                        Color32::from_rgb(0, 153, 51)
                    };
                    ui.label(egui::RichText::new(msg).color(color).strong());
                }

                // Action Buttons
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new("Export Animated GIF")
                                .min_size(egui::vec2(150.0, 24.0)),
                        )
                        .clicked()
                    {
                        export_triggered = true;
                    }
                });
            });

        self.is_open = is_open;

        if export_triggered {
            let frame_range = match self.frame_range_mode {
                FrameRangeMode::All => FrameRangeChoice::All,
                FrameRangeMode::ActiveFrame => FrameRangeChoice::ActiveFrame,
                FrameRangeMode::Tag => FrameRangeChoice::Tag(self.selected_tag.clone()),
                FrameRangeMode::Custom => FrameRangeChoice::CustomRange {
                    start: self.custom_start,
                    end: self.custom_end,
                },
            };

            let layers = match self.layer_scope {
                LayerScopeChoice::VisibleComposite | LayerScopeChoice::OneSequencePerLayer => {
                    LayerExportScope::VisibleComposite
                }
                LayerScopeChoice::ActiveLayerOnly => LayerExportScope::SingleLayer(active_layer_id),
            };

            let transparency = match self.transparency_mode {
                TransparencyChoice::KeepTransparency => TransparencyOption::KeepTransparency,
                TransparencyChoice::FlattenToSolidColor => {
                    TransparencyOption::Flatten(self.flatten_color)
                }
            };

            let opts = GifExportOptions {
                frame_range,
                layers,
                transparency,
                fps_override: if self.use_custom_fps {
                    Some(self.custom_fps)
                } else {
                    None
                },
                scale: self.scale,
                repeat: self.repeat_mode,
            };

            match export_gif_to_disk(doc, &opts, &self.output_path) {
                Ok(res) => {
                    self.status_is_error = false;
                    self.status_message = Some(format!(
                        "Successfully exported animated GIF ({} frames, {}x{} px) to {}",
                        res.frame_count,
                        res.width,
                        res.height,
                        self.output_path.display()
                    ));
                    export_result = Some(res);
                }
                Err(err) => {
                    self.status_is_error = true;
                    self.status_message = Some(format!("GIF Export failed: {}", err));
                }
            }
        }

        export_result
    }
}
