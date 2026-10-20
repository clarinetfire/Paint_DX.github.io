//! New Project / New Canvas Dialog.
//!
//! Prompts user for project dimensions (width, height), framerate (FPS),
//! presets (Full HD, HD, Classic, Pixel Art, Sprite), and background color.

use eframe::egui::{self, Context, Window};
use paint_core::palette::{ColorRgba, WHITE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasPreset {
    Custom,
    FullHd,     // 1920 x 1080
    Hd,         // 1280 x 720
    Classic800, // 800 x 600
    Classic640, // 640 x 480
    Retro320,   // 320 x 240
    Sprite64,   // 64 x 64
    Icon32,     // 32 x 32
}

impl CanvasPreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Custom => "Custom Dimensions",
            Self::FullHd => "1920 × 1080 (Full HD)",
            Self::Hd => "1280 × 720 (HD)",
            Self::Classic800 => "800 × 600 (Classic)",
            Self::Classic640 => "640 × 480 (VGA)",
            Self::Retro320 => "320 × 240 (Retro Pixel Art)",
            Self::Sprite64 => "64 × 64 (Sprite / Tile)",
            Self::Icon32 => "32 × 32 (Icon)",
        }
    }

    pub fn dimensions(&self) -> Option<(u32, u32)> {
        match self {
            Self::Custom => None,
            Self::FullHd => Some((1920, 1080)),
            Self::Hd => Some((1280, 720)),
            Self::Classic800 => Some((800, 600)),
            Self::Classic640 => Some((640, 480)),
            Self::Retro320 => Some((320, 240)),
            Self::Sprite64 => Some((64, 64)),
            Self::Icon32 => Some((32, 32)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgChoice {
    White,
    Transparent,
}

#[derive(Debug, Clone)]
pub struct NewProjectConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bg_color: ColorRgba,
}

#[derive(Debug)]
pub struct NewProjectDialog {
    pub is_open: bool,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub preset: CanvasPreset,
    pub bg_choice: BgChoice,
}

impl Default for NewProjectDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            width: 800,
            height: 600,
            fps: 12,
            preset: CanvasPreset::Classic800,
            bg_choice: BgChoice::White,
        }
    }
}

impl NewProjectDialog {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn ui(&mut self, ctx: &Context) -> Option<NewProjectConfig> {
        if !self.is_open {
            return None;
        }

        let mut confirmed = false;
        let mut is_open = self.is_open;

        Window::new("New Project / Canvas Parameters")
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .default_width(360.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                // Presets
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Preset:").strong());
                    egui::ComboBox::from_id_salt("new_project_preset_combo")
                        .selected_text(self.preset.label())
                        .show_ui(ui, |ui| {
                            let all_presets = [
                                CanvasPreset::Classic800,
                                CanvasPreset::Classic640,
                                CanvasPreset::FullHd,
                                CanvasPreset::Hd,
                                CanvasPreset::Retro320,
                                CanvasPreset::Sprite64,
                                CanvasPreset::Icon32,
                                CanvasPreset::Custom,
                            ];
                            for p in all_presets {
                                if ui
                                    .selectable_value(&mut self.preset, p, p.label())
                                    .clicked()
                                {
                                    if let Some((w, h)) = p.dimensions() {
                                        self.width = w;
                                        self.height = h;
                                    }
                                }
                            }
                        });
                });

                // Dimensions
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Canvas Dimensions (Pixels):").strong());
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.width)
                                    .range(1..=8192)
                                    .speed(1.0)
                                    .suffix(" px"),
                            )
                            .changed()
                        {
                            self.preset = CanvasPreset::Custom;
                        }

                        ui.label("Height:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.height)
                                    .range(1..=8192)
                                    .speed(1.0)
                                    .suffix(" px"),
                            )
                            .changed()
                        {
                            self.preset = CanvasPreset::Custom;
                        }
                    });
                });

                // Animation Framerate
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Animation Framerate:").strong());
                    ui.horizontal(|ui| {
                        ui.label("Framerate (FPS):");
                        ui.add(
                            egui::DragValue::new(&mut self.fps)
                                .range(1..=60)
                                .speed(0.2)
                                .suffix(" fps"),
                        );
                    });
                });

                // Background
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Initial Background:").strong());
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.bg_choice, BgChoice::White, "White");
                        ui.radio_value(&mut self.bg_choice, BgChoice::Transparent, "Transparent");
                    });
                });

                // Action Buttons
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::Button::new("Create Canvas").min_size(egui::vec2(120.0, 24.0)))
                        .clicked()
                    {
                        confirmed = true;
                    }

                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        self.is_open = is_open;

        if confirmed {
            self.is_open = false;
            let bg_color = match self.bg_choice {
                BgChoice::White => WHITE,
                BgChoice::Transparent => ColorRgba::rgba(0, 0, 0, 0),
            };
            Some(NewProjectConfig {
                width: self.width.clamp(1, 8192),
                height: self.height.clamp(1, 8192),
                fps: self.fps.clamp(1, 60),
                bg_color,
            })
        } else {
            None
        }
    }
}
