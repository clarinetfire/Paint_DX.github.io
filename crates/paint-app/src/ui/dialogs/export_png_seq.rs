//! Export PNG Sequence Dialog.
//!
//! Provides UI controls for frame ranges, layer scopes, transparent RGBA vs flattened color,
//! nearest-neighbor scale multipliers (1x..=8x), trim modes (None, Union, Per-frame),
//! outer padding, zero-padded index naming, and collision policies.

use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, Context, Window};
use paint_core::export::png_seq::{
    export_png_sequence, ExistingFilesPolicy, FrameRangeChoice, LayerExportScope,
    PngSequenceExportOptions, PngSequenceExportResult, TransparencyOption, TrimMode,
};
use paint_core::frames::AnimationDocument;
use paint_core::layers::LayerId;
use paint_core::palette::{ColorRgba, WHITE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameRangeMode {
    All,
    ActiveFrame,
    Tag,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerScopeChoice {
    VisibleComposite,
    ActiveLayerOnly,
    OneSequencePerLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparencyChoice {
    KeepTransparency,
    FlattenToSolidColor,
}

#[derive(Debug)]
pub struct ExportPngSeqDialog {
    pub is_open: bool,
    pub output_dir: PathBuf,
    pub pattern: String,
    pub start_index: u32,
    pub padding_digits: usize,
    pub frame_range_mode: FrameRangeMode,
    pub selected_tag: String,
    pub custom_start: usize,
    pub custom_end: usize,
    pub layer_scope: LayerScopeChoice,
    pub transparency_mode: TransparencyChoice,
    pub flatten_color: ColorRgba,
    pub scale: u32,
    pub trim_mode: TrimMode,
    pub padding: u32,
    pub existing_files_policy: ExistingFilesPolicy,

    // Status / Notification
    pub status_message: Option<String>,
    pub status_is_error: bool,
}

impl Default for ExportPngSeqDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            output_dir: PathBuf::from("export"),
            pattern: "frame_".to_string(),
            start_index: 0,
            padding_digits: 4,
            frame_range_mode: FrameRangeMode::All,
            selected_tag: String::new(),
            custom_start: 0,
            custom_end: 0,
            layer_scope: LayerScopeChoice::VisibleComposite,
            transparency_mode: TransparencyChoice::KeepTransparency,
            flatten_color: WHITE,
            scale: 1,
            trim_mode: TrimMode::None,
            padding: 0,
            existing_files_policy: ExistingFilesPolicy::Overwrite,
            status_message: None,
            status_is_error: false,
        }
    }
}

impl ExportPngSeqDialog {
    pub fn open(&mut self, current_project_path: Option<&Path>, doc: &AnimationDocument) {
        self.is_open = true;
        self.status_message = None;
        self.status_is_error = false;

        // Auto-configure output folder based on current project or default
        if let Some(path) = current_project_path {
            if let Some(parent) = path.parent() {
                self.output_dir = parent.join("export");
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                self.pattern = format!("{}_", stem);
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
    ) -> Option<PngSequenceExportResult> {
        if !self.is_open {
            return None;
        }

        let mut export_triggered = false;
        let mut export_result = None;

        let mut is_open = self.is_open;
        Window::new("Export PNG Sequence")
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .default_width(460.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);

                // 1. Output Destination Folder
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Destination Directory:").strong());
                    ui.horizontal(|ui| {
                        let path_str = self.output_dir.to_string_lossy();
                        ui.add(
                            egui::TextEdit::singleline(&mut path_str.to_string())
                                .desired_width(320.0)
                                .interactive(false),
                        );
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Browse...").clicked() {
                            if let Some(folder) = rfd::FileDialog::new()
                                .set_directory(&self.output_dir)
                                .pick_folder()
                            {
                                self.output_dir = folder;
                            }
                        }
                    });
                });

                // 2. Filename Pattern & Numbering
                ui.group(|ui| {
                    ui.label(egui::RichText::new("File Naming:").strong());
                    ui.horizontal(|ui| {
                        ui.label("Prefix:");
                        ui.add(egui::TextEdit::singleline(&mut self.pattern).desired_width(120.0));

                        ui.label("Start Index:");
                        ui.add(egui::DragValue::new(&mut self.start_index).range(0..=999999));

                        ui.label("Padding:");
                        ui.add(egui::DragValue::new(&mut self.padding_digits).range(1..=8));
                    });

                    // Filename sample preview
                    let digits = self.padding_digits.clamp(1, 8);
                    let sample_num = format!("{:0width$}", self.start_index, width = digits);
                    let sample_name = format!("{}{}.png", self.pattern, sample_num);
                    ui.label(
                        egui::RichText::new(format!("Preview: {}", sample_name))
                            .color(Color32::from_rgb(0, 102, 204))
                            .small(),
                    );
                });

                // 3. Frame Range Selection
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Frame Range:").strong());
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.frame_range_mode,
                            FrameRangeMode::All,
                            format!("All Frames ({})", doc.frames.len()),
                        );
                        ui.radio_value(
                            &mut self.frame_range_mode,
                            FrameRangeMode::ActiveFrame,
                            format!("Active ({})", doc.active_frame_index + 1),
                        );
                    });

                    ui.horizontal(|ui| {
                        let has_tags = !doc.tags.is_empty();
                        ui.add_enabled_ui(has_tags, |ui| {
                            ui.radio_value(&mut self.frame_range_mode, FrameRangeMode::Tag, "Tag:");
                            if has_tags {
                                egui::ComboBox::from_id_salt("tag_picker")
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
                            }
                        });

                        ui.radio_value(
                            &mut self.frame_range_mode,
                            FrameRangeMode::Custom,
                            "Custom:",
                        );
                        ui.add_enabled_ui(self.frame_range_mode == FrameRangeMode::Custom, |ui| {
                            let max_idx = doc.frames.len().saturating_sub(1);
                            ui.add(
                                egui::DragValue::new(&mut self.custom_start)
                                    .range(0..=max_idx)
                                    .prefix("From: "),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.custom_end)
                                    .range(0..=max_idx)
                                    .prefix("To: "),
                            );
                        });
                    });
                });

                // 4. Layers Scope
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Layers:").strong());
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.layer_scope,
                            LayerScopeChoice::VisibleComposite,
                            "Visible Composite",
                        );
                        ui.radio_value(
                            &mut self.layer_scope,
                            LayerScopeChoice::ActiveLayerOnly,
                            "Active Layer Only",
                        );
                        ui.radio_value(
                            &mut self.layer_scope,
                            LayerScopeChoice::OneSequencePerLayer,
                            "One Sequence Per Layer",
                        );
                    });
                });

                // 5. Transparency & Flattening
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Transparency:").strong());
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.transparency_mode,
                            TransparencyChoice::KeepTransparency,
                            "Keep Transparency (RGBA)",
                        );
                        ui.radio_value(
                            &mut self.transparency_mode,
                            TransparencyChoice::FlattenToSolidColor,
                            "Flatten to Solid Color:",
                        );

                        if self.transparency_mode == TransparencyChoice::FlattenToSolidColor {
                            let mut rgb = [
                                self.flatten_color.r,
                                self.flatten_color.g,
                                self.flatten_color.b,
                            ];
                            if ui.color_edit_button_srgb(&mut rgb).changed() {
                                self.flatten_color = ColorRgba::rgb(rgb[0], rgb[1], rgb[2]);
                            }
                        }
                    });
                });

                // 6. Scale, Trim, Padding, Existing Files
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Transform & Output Options:").strong());

                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        for s in 1..=8 {
                            if ui
                                .selectable_label(self.scale == s, format!("{}x", s))
                                .clicked()
                            {
                                self.scale = s;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Trim:");
                        ui.radio_value(&mut self.trim_mode, TrimMode::None, "None");
                        ui.radio_value(&mut self.trim_mode, TrimMode::Union, "Union (All Frames)");
                        ui.radio_value(&mut self.trim_mode, TrimMode::PerFrame, "Per-frame");

                        ui.add_space(10.0);
                        ui.label("Padding:");
                        ui.add(
                            egui::DragValue::new(&mut self.padding)
                                .range(0..=64)
                                .suffix(" px"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Existing files:");
                        ui.radio_value(
                            &mut self.existing_files_policy,
                            ExistingFilesPolicy::Overwrite,
                            "Overwrite",
                        );
                        ui.radio_value(
                            &mut self.existing_files_policy,
                            ExistingFilesPolicy::Skip,
                            "Skip",
                        );
                        ui.radio_value(
                            &mut self.existing_files_policy,
                            ExistingFilesPolicy::AutoRename,
                            "Auto-rename",
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
                        .add(egui::Button::new("Export Sequence").min_size(egui::vec2(120.0, 24.0)))
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
                LayerScopeChoice::VisibleComposite => LayerExportScope::VisibleComposite,
                LayerScopeChoice::ActiveLayerOnly => LayerExportScope::SingleLayer(active_layer_id),
                LayerScopeChoice::OneSequencePerLayer => LayerExportScope::OneSequencePerLayer,
            };

            let transparency = match self.transparency_mode {
                TransparencyChoice::KeepTransparency => TransparencyOption::KeepTransparency,
                TransparencyChoice::FlattenToSolidColor => {
                    TransparencyOption::Flatten(self.flatten_color)
                }
            };

            let opts = PngSequenceExportOptions {
                output_dir: self.output_dir.clone(),
                pattern: self.pattern.clone(),
                start_index: self.start_index,
                padding_digits: self.padding_digits,
                frame_range,
                layers,
                transparency,
                scale: self.scale,
                trim: self.trim_mode,
                padding: self.padding,
                existing_files: self.existing_files_policy,
            };

            match export_png_sequence(doc, &opts, None, |_, _| true) {
                Ok(res) => {
                    self.status_message = Some(format!(
                        "Successfully exported {} frame(s) to {}",
                        res.exported_files.len(),
                        self.output_dir.display()
                    ));
                    self.status_is_error = false;
                    export_result = Some(res);
                }
                Err(err) => {
                    self.status_message = Some(format!("Export failed: {}", err));
                    self.status_is_error = true;
                }
            }
        }

        export_result
    }
}
