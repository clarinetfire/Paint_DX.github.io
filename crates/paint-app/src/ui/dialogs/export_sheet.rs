//! Export Sprite Sheet Dialog.
//!
//! Provides interactive UI controls for sprite sheet layouts (Strip H/V, Grid, Packed),
//! cell sizing, spacing, border padding, edge pixel extrusion, Power-of-Two constraints,
//! transparent RGBA vs solid flattening, scale multipliers, real-time visual preview,
//! and versioned JSON / CSV metadata generation.

use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, Context, TextureOptions, Window};
use paint_core::export::png_seq::{
    FrameRangeChoice, LayerExportScope, TransparencyOption, TrimMode,
};
use paint_core::export::sheet::{
    export_sprite_sheet_to_disk, generate_sprite_sheet, FrameOrdering, MetadataFormat, SheetLayout,
    SheetSizeMode, SpriteSheetOptions, SpriteSheetResult,
};
use paint_core::frames::AnimationDocument;
use paint_core::layers::LayerId;
use paint_core::palette::{ColorRgba, WHITE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetFrameRangeMode {
    All,
    ActiveFrame,
    Tag,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetTransparencyChoice {
    KeepTransparency,
    FlattenToSolidColor,
}

pub struct ExportSheetDialog {
    pub is_open: bool,
    pub output_path: PathBuf,
    pub layout: SheetLayout,
    pub columns: u32,
    pub spacing: u32,
    pub border: u32,
    pub extrusion: u32,
    pub is_power_of_two: bool,
    pub frame_range_mode: SheetFrameRangeMode,
    pub selected_tag: String,
    pub custom_start: usize,
    pub custom_end: usize,
    pub ordering: FrameOrdering,
    pub transparency_mode: SheetTransparencyChoice,
    pub flatten_color: ColorRgba,
    pub scale: u32,
    pub trim_mode: TrimMode,
    pub padding: u32,
    pub metadata_format: MetadataFormat,

    // Status / Notification
    pub status_message: Option<String>,
    pub status_is_error: bool,

    // Preview cache
    preview_texture: Option<egui::TextureHandle>,
    last_preview_dims: (u32, u32),
    last_preview_frame_count: usize,
}

impl std::fmt::Debug for ExportSheetDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportSheetDialog")
            .field("is_open", &self.is_open)
            .field("output_path", &self.output_path)
            .field("layout", &self.layout)
            .field("columns", &self.columns)
            .field("spacing", &self.spacing)
            .field("border", &self.border)
            .field("extrusion", &self.extrusion)
            .field("is_power_of_two", &self.is_power_of_two)
            .field("scale", &self.scale)
            .field("trim_mode", &self.trim_mode)
            .finish()
    }
}

impl Default for ExportSheetDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            output_path: PathBuf::from("export/spritesheet.png"),
            layout: SheetLayout::Grid,
            columns: 4,
            spacing: 0,
            border: 0,
            extrusion: 0,
            is_power_of_two: false,
            frame_range_mode: SheetFrameRangeMode::All,
            selected_tag: String::new(),
            custom_start: 0,
            custom_end: 0,
            ordering: FrameOrdering::ByFrameIndex,
            transparency_mode: SheetTransparencyChoice::KeepTransparency,
            flatten_color: WHITE,
            scale: 1,
            trim_mode: TrimMode::None,
            padding: 0,
            metadata_format: MetadataFormat::Json,
            status_message: None,
            status_is_error: false,
            preview_texture: None,
            last_preview_dims: (0, 0),
            last_preview_frame_count: 0,
        }
    }
}

impl ExportSheetDialog {
    pub fn open(&mut self, current_project_path: Option<&Path>, doc: &AnimationDocument) {
        self.is_open = true;
        self.status_message = None;
        self.status_is_error = false;
        self.preview_texture = None;

        if let Some(path) = current_project_path {
            if let Some(parent) = path.parent() {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("spritesheet");
                self.output_path = parent.join(format!("{}_sheet.png", stem));
            }
        }

        self.custom_start = 0;
        self.custom_end = doc.frames.len().saturating_sub(1);
        if let Some(first_tag) = doc.tags.first() {
            self.selected_tag = first_tag.name.clone();
        }
    }

    fn build_options(&self, _active_layer_id: LayerId) -> SpriteSheetOptions {
        let frame_range = match self.frame_range_mode {
            SheetFrameRangeMode::All => FrameRangeChoice::All,
            SheetFrameRangeMode::ActiveFrame => FrameRangeChoice::ActiveFrame,
            SheetFrameRangeMode::Tag => FrameRangeChoice::Tag(self.selected_tag.clone()),
            SheetFrameRangeMode::Custom => FrameRangeChoice::CustomRange {
                start: self.custom_start,
                end: self.custom_end,
            },
        };

        let transparency = match self.transparency_mode {
            SheetTransparencyChoice::KeepTransparency => TransparencyOption::KeepTransparency,
            SheetTransparencyChoice::FlattenToSolidColor => {
                TransparencyOption::Flatten(self.flatten_color)
            }
        };

        let size_mode = if self.is_power_of_two {
            SheetSizeMode::PowerOfTwo
        } else {
            SheetSizeMode::Auto
        };

        SpriteSheetOptions {
            layout: self.layout,
            columns: self.columns.max(1),
            spacing: self.spacing,
            border: self.border,
            extrusion: self.extrusion,
            size_mode,
            frame_range,
            layers: LayerExportScope::VisibleComposite,
            transparency,
            scale: self.scale.max(1),
            trim: self.trim_mode,
            padding: self.padding,
            metadata_format: self.metadata_format,
            ordering: self.ordering,
        }
    }

    /// Renders the dialog window and executes export when triggered.
    pub fn ui(
        &mut self,
        ctx: &Context,
        doc: &AnimationDocument,
        active_layer_id: LayerId,
    ) -> Option<SpriteSheetResult> {
        if !self.is_open {
            return None;
        }

        let mut export_triggered = false;
        let mut export_result = None;

        let mut is_open = self.is_open;
        Window::new("Export Sprite Sheet")
            .open(&mut is_open)
            .resizable(true)
            .collapsible(false)
            .default_width(620.0)
            .default_height(540.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);

                // Top: Output Destination Path
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Destination PNG File:").strong());
                    ui.horizontal(|ui| {
                        let path_str = self.output_path.to_string_lossy();
                        ui.add(
                            egui::TextEdit::singleline(&mut path_str.to_string())
                                .desired_width(450.0)
                                .interactive(false),
                        );
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Browse...").clicked() {
                            if let Some(file) = rfd::FileDialog::new()
                                .add_filter("PNG Image (*.png)", &["png"])
                                .set_file_name(
                                    self.output_path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("sheet.png"),
                                )
                                .save_file()
                            {
                                self.output_path = file;
                            }
                        }
                    });
                });

                // Main two-column split: Settings on left, Preview on right
                ui.horizontal(|ui| {
                    // Left Column: Configuration options
                    ui.vertical(|ui| {
                        ui.set_width(320.0);

                        // 1. Layout & Grid
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Layout & Arrangement:").strong());
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut self.layout, SheetLayout::Grid, "Grid");
                                ui.radio_value(&mut self.layout, SheetLayout::HorizontalStrip, "H-Strip");
                                ui.radio_value(&mut self.layout, SheetLayout::VerticalStrip, "V-Strip");
                                ui.radio_value(&mut self.layout, SheetLayout::Packed, "Packed");
                            });

                            if self.layout == SheetLayout::Grid {
                                ui.horizontal(|ui| {
                                    ui.label("Columns:");
                                    ui.add(egui::DragValue::new(&mut self.columns).range(1..=64));
                                });
                            }

                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.is_power_of_two, "Power of Two (POT)");
                            });
                        });

                        // 2. Spacing, Border, Extrusion
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Spacing & Extrusion:").strong());
                            ui.horizontal(|ui| {
                                ui.label("Spacing:");
                                ui.add(egui::DragValue::new(&mut self.spacing).range(0..=64).suffix(" px"));

                                ui.label("Border:");
                                ui.add(egui::DragValue::new(&mut self.border).range(0..=64).suffix(" px"));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Edge Extrusion:");
                                ui.add(egui::DragValue::new(&mut self.extrusion).range(0..=8).suffix(" px"))
                                    .on_hover_text("Extrudes border pixels outward to prevent texture filtering bleed");
                            });
                        });

                        // 3. Frame Selection & Ordering
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Frame Selection:").strong());
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut self.frame_range_mode,
                                    SheetFrameRangeMode::All,
                                    format!("All ({})", doc.frames.len()),
                                );
                                ui.radio_value(
                                    &mut self.frame_range_mode,
                                    SheetFrameRangeMode::ActiveFrame,
                                    "Active",
                                );
                            });

                            ui.horizontal(|ui| {
                                let has_tags = !doc.tags.is_empty();
                                ui.add_enabled_ui(has_tags, |ui| {
                                    ui.radio_value(&mut self.frame_range_mode, SheetFrameRangeMode::Tag, "Tag:");
                                    if has_tags {
                                        egui::ComboBox::from_id_salt("sheet_tag_picker")
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
                                    SheetFrameRangeMode::Custom,
                                    "Custom:",
                                );
                                ui.add_enabled_ui(self.frame_range_mode == SheetFrameRangeMode::Custom, |ui| {
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

                            ui.horizontal(|ui| {
                                ui.label("Order:");
                                ui.radio_value(&mut self.ordering, FrameOrdering::ByFrameIndex, "By Index");
                                ui.radio_value(&mut self.ordering, FrameOrdering::ByTag, "By Tag");
                            });
                        });

                        // 4. Transform, Trim & Scaling
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Transform & Trim:").strong());
                            ui.horizontal(|ui| {
                                ui.label("Scale:");
                                for s in 1..=4 {
                                    if ui.selectable_label(self.scale == s, format!("{}x", s)).clicked() {
                                        self.scale = s;
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Trim:");
                                ui.radio_value(&mut self.trim_mode, TrimMode::None, "None");
                                ui.radio_value(&mut self.trim_mode, TrimMode::Union, "Union");
                                ui.radio_value(&mut self.trim_mode, TrimMode::PerFrame, "Per-frame");
                            });
                        });

                        // 5. Transparency & Metadata
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Transparency & Metadata:").strong());
                            ui.horizontal(|ui| {
                                ui.radio_value(
                                    &mut self.transparency_mode,
                                    SheetTransparencyChoice::KeepTransparency,
                                    "RGBA (Alpha)",
                                );
                                ui.radio_value(
                                    &mut self.transparency_mode,
                                    SheetTransparencyChoice::FlattenToSolidColor,
                                    "Solid Color:",
                                );

                                if self.transparency_mode == SheetTransparencyChoice::FlattenToSolidColor {
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

                            ui.horizontal(|ui| {
                                ui.label("Metadata:");
                                ui.radio_value(&mut self.metadata_format, MetadataFormat::Json, "JSON");
                                ui.radio_value(&mut self.metadata_format, MetadataFormat::Csv, "CSV");
                                ui.radio_value(&mut self.metadata_format, MetadataFormat::None, "None");
                            });
                        });
                    });

                    ui.separator();

                    // Right Column: Live Sheet Visual Preview
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Sprite Sheet Preview:").strong());

                        // Generate preview
                        let filename = self
                            .output_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("sheet.png");
                        let opts = self.build_options(active_layer_id);

                        if let Ok(sheet_res) = generate_sprite_sheet(doc, &opts, filename) {
                            self.last_preview_dims = (sheet_res.sheet_width, sheet_res.sheet_height);
                            self.last_preview_frame_count = sheet_res.frames.len();

                            // Update GPU texture for preview
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                [sheet_res.sheet_width as usize, sheet_res.sheet_height as usize],
                                sheet_res.sheet_buffer.as_raw(),
                            );
                            self.preview_texture = Some(ctx.load_texture(
                                "sheet_preview",
                                color_image,
                                TextureOptions::NEAREST,
                            ));
                        }

                        ui.label(
                            egui::RichText::new(format!(
                                "Dimensions: {} x {} px | Frames: {}",
                                self.last_preview_dims.0,
                                self.last_preview_dims.1,
                                self.last_preview_frame_count
                            ))
                            .small()
                            .color(Color32::from_rgb(0, 102, 204)),
                        );

                        // Draw preview container with dark checkerboard aesthetic
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            let available_size = ui.available_size();
                            let max_preview_w = available_size.x.min(260.0);
                            let max_preview_h = 240.0;

                            if let Some(ref tex) = self.preview_texture {
                                let tex_size = tex.size_vec2();
                                if tex_size.x > 0.0 && tex_size.y > 0.0 {
                                    let scale_factor = (max_preview_w / tex_size.x)
                                        .min(max_preview_h / tex_size.y)
                                        .min(4.0);
                                    let display_size = tex_size * scale_factor;

                                    let (rect, _response) =
                                        ui.allocate_exact_size(display_size, egui::Sense::hover());
                                    ui.painter().image(
                                        tex.id(),
                                        rect,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        Color32::WHITE,
                                    );
                                }
                            }
                        });
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

                // Bottom Action Buttons
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::Button::new("Export Sprite Sheet").min_size(egui::vec2(140.0, 24.0)))
                        .clicked()
                    {
                        export_triggered = true;
                    }
                });
            });

        self.is_open = is_open;

        if export_triggered {
            let opts = self.build_options(active_layer_id);
            match export_sprite_sheet_to_disk(doc, &opts, &self.output_path) {
                Ok(res) => {
                    self.status_message = Some(format!(
                        "Successfully exported {} frame(s) sprite sheet ({}x{}) to {}",
                        res.frames.len(),
                        res.sheet_width,
                        res.sheet_height,
                        self.output_path.display()
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
